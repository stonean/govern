//! `enforce-manifest` — directory cleanup against an expected file list.
//!
//! Walks `directory`, removes files matching `glob-include` whose
//! relative path is neither in `expected` nor `pinned`, and returns the
//! per-file outcome split across `removed` / `kept` / `pinned-kept`.
//! The bootstrap's sole caller is the per-agent slash-command manifest
//! enforcement loop; adopter cleanup of historical conventions (the
//! legacy `skills/` directory, legacy workflow filenames, and so on) is
//! owned by the registry-driven `## Pre-run Migrations` loop and the
//! per-entry procedure files at `framework/migrations/{id}.md` (spec
//! 027). The primitive itself is generic — it removes whatever files
//! the caller's `expected` list omits — but the legacy adopter-cleanup
//! responsibility has moved out of its contract.
//!
//! Companion to `apply-manifest`: `apply-manifest` writes the files
//! callers want present; `enforce-manifest` deletes the files no longer
//! on the list. Splitting the two keeps each primitive's failure modes
//! narrow and lets callers compose them (apply-then-enforce for
//! first-run; just-apply for incremental scaffolding).
//!
//! Semantics:
//!
//! - `recursive: false` (default): top-level only. Subdirectories are
//!   skipped entirely. Matches the bootstrap's slash-command cleanup.
//! - `recursive: true`: walks every file under `directory`. Relative
//!   paths use forward slashes regardless of host OS so `expected` and
//!   `pinned` match identically across platforms.
//! - `glob-include` (default `*.md`): only files whose basename matches
//!   the glob are considered. Files outside the glob are left
//!   untouched. Glob syntax: `*` matches any run of characters, `?`
//!   matches one character; every other character is literal. Globs
//!   never cross directory separators because they're matched against
//!   the basename.
//! - Missing or empty `directory`: success with zero removals. The
//!   primitive does NOT create `directory`; `apply-manifest` is
//!   responsible for materializing destination paths.
//! - Pinned matching is case-sensitive on Unix and case-insensitive on
//!   Windows (matching NTFS semantics), parallel to `apply-manifest`.

use std::fs;
use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::WalkDir;

use crate::primitives::{PrimitiveError, Result, validate_no_traversal};
use crate::schema::primitives::{EnforceManifestArgs, EnforceManifestResult};

const DEFAULT_GLOB: &str = "*.md";

/// Execute the `enforce-manifest` primitive.
///
/// # Errors
///
/// - [`PrimitiveError::InvalidPath`] when `directory` resolves outside
///   the repo root (parent-directory components, or an absolute path
///   that does not sit under `repo`). This primitive deletes files, so
///   the containment check runs before any filesystem operation
///   (BE-INPUT-004 defense-in-depth). Legitimate absolute directories
///   inside the repo keep working.
/// - [`PrimitiveError::Io`] on local filesystem failures while walking
///   the directory or removing files.
pub fn run(args: &EnforceManifestArgs, repo: &Path) -> Result<EnforceManifestResult> {
    let directory = resolve_contained_dir(repo, &args.directory)?;
    let mut result = EnforceManifestResult::default();

    // Missing directory: zero-removal success. The primitive does not
    // create the directory — that's `apply-manifest`'s job.
    match directory.try_exists() {
        Ok(false) => return Ok(result),
        Err(source) => {
            return Err(PrimitiveError::Io {
                path: directory.clone(),
                source,
            });
        }
        Ok(true) => {}
    }
    if !directory.is_dir() {
        return Err(PrimitiveError::Io {
            path: directory.clone(),
            source: std::io::Error::other("enforce-manifest target is not a directory"),
        });
    }

    let glob = args.glob_include.as_deref().unwrap_or(DEFAULT_GLOB);
    let glob_regex = compile_glob(glob);

    let expected: Vec<String> = args.expected.iter().map(|p| normalize(p)).collect();
    let pinned: Vec<String> = args.pinned.iter().map(|p| normalize(p)).collect();

    let entries = collect_entries(&directory, args.recursive)?;
    for entry in entries {
        let Ok(rel) = entry.strip_prefix(&directory) else {
            continue;
        };
        let rel_str = path_to_forward_slash(rel);
        let basename = entry
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        if !glob_regex.is_match(&basename) {
            // Outside the glob — do not touch.
            continue;
        }

        if list_contains(&expected, &rel_str) {
            result.kept.push(rel_str);
        } else if list_contains(&pinned, &rel_str) {
            result.pinned_kept.push(rel_str);
        } else {
            fs::remove_file(&entry).map_err(|source| PrimitiveError::Io {
                path: entry.clone(),
                source,
            })?;
            result.removed.push(rel_str);
        }
    }

    Ok(result)
}

/// Resolve `directory` against `repo`, guaranteeing the result sits under
/// the repo root. Relative paths go through [`validate_no_traversal`]
/// (no `..`, non-empty); absolute paths are allowed only when they are
/// `..`-free and prefixed by `repo` — the destructive cleanup loop must
/// never walk an arbitrary filesystem location.
fn resolve_contained_dir(repo: &Path, p: &str) -> Result<PathBuf> {
    let candidate = Path::new(p);
    if !candidate.is_absolute() {
        validate_no_traversal(p)?;
        return Ok(repo.join(candidate));
    }
    if candidate
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir))
    {
        return Err(PrimitiveError::InvalidPath {
            path: p.into(),
            reason: "parent-directory component ('..') not permitted".into(),
        });
    }
    if !candidate.starts_with(repo) {
        return Err(PrimitiveError::InvalidPath {
            path: p.into(),
            reason: format!(
                "absolute directory must sit under the repo root {}",
                repo.display()
            ),
        });
    }
    Ok(candidate.to_path_buf())
}

fn normalize(p: &str) -> String {
    p.replace('\\', "/")
}

fn path_to_forward_slash(p: &Path) -> String {
    p.to_string_lossy().replace('\\', "/")
}

fn list_contains(haystack: &[String], needle: &str) -> bool {
    if cfg!(windows) {
        haystack.iter().any(|h| h.eq_ignore_ascii_case(needle))
    } else {
        haystack.iter().any(|h| h == needle)
    }
}

/// Collect every regular file under `directory`. When `recursive` is
/// false, only the immediate children are returned. Symlinks are
/// followed for `is_file()` purposes but not traversed as directories.
fn collect_entries(directory: &Path, recursive: bool) -> Result<Vec<PathBuf>> {
    let mut out = Vec::new();
    if recursive {
        for entry in WalkDir::new(directory).min_depth(1).follow_links(false) {
            let entry = entry.map_err(|err| PrimitiveError::Io {
                path: err
                    .path()
                    .map_or_else(|| directory.into(), Path::to_path_buf),
                source: err
                    .into_io_error()
                    .unwrap_or_else(|| std::io::Error::other("walkdir error")),
            })?;
            if entry.file_type().is_file() {
                out.push(entry.into_path());
            }
        }
    } else {
        let read = fs::read_dir(directory).map_err(|source| PrimitiveError::Io {
            path: directory.into(),
            source,
        })?;
        for entry in read {
            let entry = entry.map_err(|source| PrimitiveError::Io {
                path: directory.into(),
                source,
            })?;
            let path = entry.path();
            let ft = entry.file_type().map_err(|source| PrimitiveError::Io {
                path: path.clone(),
                source,
            })?;
            if ft.is_file() {
                out.push(path);
            }
        }
    }
    out.sort();
    Ok(out)
}

/// Compile an fnmatch-style glob to an anchored regex. Supports `*`
/// (zero or more characters) and `?` (exactly one character); every
/// other character is escaped to its literal regex form via
/// [`regex::escape`]. Globs never cross directory separators because
/// the caller matches them against a basename.
///
/// Every character either becomes `.*` / `.` or a `regex::escape`d
/// literal — the resulting regex is always valid, so this function is
/// infallible by construction.
fn compile_glob(pattern: &str) -> Regex {
    let mut re = String::with_capacity(pattern.len() * 2 + 2);
    re.push('^');
    let mut buf = [0u8; 4];
    for c in pattern.chars() {
        match c {
            '*' => re.push_str(".*"),
            '?' => re.push('.'),
            ch => re.push_str(&regex::escape(ch.encode_utf8(&mut buf))),
        }
    }
    re.push('$');
    Regex::new(&re).unwrap_or_else(|err| {
        unreachable!("compile_glob produced invalid regex {re:?} (pattern={pattern:?}): {err}")
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn args_for(
        directory: &Path,
        expected: &[&str],
        pinned: &[&str],
        recursive: bool,
        glob: Option<&str>,
    ) -> EnforceManifestArgs {
        EnforceManifestArgs {
            directory: directory.to_string_lossy().into_owned(),
            expected: expected.iter().map(|s| (*s).to_string()).collect(),
            pinned: pinned.iter().map(|s| (*s).to_string()).collect(),
            recursive,
            glob_include: glob.map(str::to_string),
        }
    }

    fn touch(path: &Path) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, "x\n").unwrap();
    }

    #[test]
    fn top_level_cleanup_removes_unlisted_md_files() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("cmds");
        for f in ["status.md", "target.md", "legacy-workflow.md"] {
            touch(&dir.join(f));
        }

        let args = args_for(
            &dir,
            &["status.md", "target.md"],
            &[],
            false,
            None, // default *.md
        );
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.removed, vec!["legacy-workflow.md"]);
        assert_eq!(
            result
                .kept
                .iter()
                .collect::<std::collections::BTreeSet<_>>(),
            ["status.md".to_string(), "target.md".to_string()]
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
        );
        assert!(result.pinned_kept.is_empty());
        assert!(!dir.join("legacy-workflow.md").exists());
        assert!(dir.join("status.md").exists());
    }

    #[test]
    fn pinned_files_are_kept_with_pinned_label() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("cmds");
        for f in ["status.md", "adopter-custom.md", "stale.md"] {
            touch(&dir.join(f));
        }

        let args = args_for(&dir, &["status.md"], &["adopter-custom.md"], false, None);
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.removed, vec!["stale.md"]);
        assert_eq!(result.kept, vec!["status.md"]);
        assert_eq!(result.pinned_kept, vec!["adopter-custom.md"]);
        assert!(dir.join("adopter-custom.md").exists());
        assert!(!dir.join("stale.md").exists());
    }

    #[test]
    fn missing_directory_is_zero_removal_success() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("never-existed");
        let args = args_for(&dir, &["x.md"], &[], false, None);
        let result = run(&args, tmp.path()).unwrap();
        assert!(result.removed.is_empty());
        assert!(result.kept.is_empty());
        assert!(result.pinned_kept.is_empty());
        // Crucially: the primitive does NOT create the directory.
        assert!(!dir.exists(), "enforce-manifest must not create the dir");
    }

    #[test]
    fn empty_directory_is_zero_removal_success() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("empty");
        fs::create_dir_all(&dir).unwrap();
        let args = args_for(&dir, &["x.md"], &[], false, None);
        let result = run(&args, tmp.path()).unwrap();
        assert!(result.removed.is_empty());
        assert!(result.kept.is_empty());
        assert!(result.pinned_kept.is_empty());
    }

    #[test]
    fn recursive_cleanup_walks_into_subdirectories() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("framework");
        for f in [
            "commands/status.md",
            "commands/legacy.md",
            "bootstrap/govern.md",
            "skills/old-skill.md",
        ] {
            touch(&dir.join(f));
        }

        let args = args_for(
            &dir,
            &["commands/status.md", "bootstrap/govern.md"],
            &[],
            true,
            None,
        );
        let result = run(&args, tmp.path()).unwrap();
        let mut removed_sorted = result.removed.clone();
        removed_sorted.sort();
        assert_eq!(
            removed_sorted,
            vec!["commands/legacy.md", "skills/old-skill.md"]
        );
        assert!(!dir.join("commands/legacy.md").exists());
        assert!(!dir.join("skills/old-skill.md").exists());
        assert!(dir.join("commands/status.md").exists());
        assert!(dir.join("bootstrap/govern.md").exists());
    }

    #[test]
    fn non_recursive_walk_ignores_subdirectories() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("framework");
        touch(&dir.join("top.md"));
        touch(&dir.join("stale.md"));
        touch(&dir.join("subdir/nested.md"));

        let args = args_for(&dir, &["top.md"], &[], false, None);
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.removed, vec!["stale.md"]);
        // The subdirectory file is invisible to a non-recursive walk.
        assert!(dir.join("subdir/nested.md").exists());
        assert!(!dir.join("stale.md").exists());
    }

    #[test]
    fn non_default_glob_only_touches_matching_files() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("workflows");
        for f in ["ci.yml", "legacy-ci.yml", "README.md"] {
            touch(&dir.join(f));
        }

        let args = args_for(&dir, &["ci.yml"], &[], false, Some("*.yml"));
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.removed, vec!["legacy-ci.yml"]);
        assert_eq!(result.kept, vec!["ci.yml"]);
        // README.md was outside the glob — untouched and unreported.
        assert!(dir.join("README.md").exists());
        assert!(!result.removed.contains(&"README.md".to_string()));
        assert!(!result.kept.contains(&"README.md".to_string()));
    }

    #[test]
    fn glob_escapes_regex_metacharacters() {
        // A literal `.` in the glob must not act as a regex
        // metacharacter — `legacy.md` must NOT match `legacyXmd`.
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("d");
        touch(&dir.join("legacy.md"));
        touch(&dir.join("legacyXmd")); // would match a buggy regex `legacy.md`

        let args = args_for(&dir, &[], &[], false, Some("legacy.md"));
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.removed, vec!["legacy.md"]);
        // The misleadingly-named file is outside the glob, kept untouched.
        assert!(dir.join("legacyXmd").exists());
    }

    #[test]
    fn question_mark_glob_matches_single_character() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("d");
        touch(&dir.join("v1.md"));
        touch(&dir.join("v22.md"));

        let args = args_for(&dir, &[], &[], false, Some("v?.md"));
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.removed, vec!["v1.md"]);
        assert!(dir.join("v22.md").exists());
    }

    #[test]
    fn star_glob_matches_everything() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("d");
        for f in ["a.md", "b.yml", "c.json", "no-extension"] {
            touch(&dir.join(f));
        }

        let args = args_for(&dir, &["a.md"], &[], false, Some("*"));
        let result = run(&args, tmp.path()).unwrap();
        let mut removed_sorted = result.removed.clone();
        removed_sorted.sort();
        assert_eq!(removed_sorted, vec!["b.yml", "c.json", "no-extension"]);
        assert_eq!(result.kept, vec!["a.md"]);
    }

    #[test]
    fn idempotent_rerun_records_zero_removals() {
        // After the first run prunes everything stale, a second run with
        // the same inputs records zero removals (the assumption being
        // that no upstream rename happened between invocations).
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("cmds");
        for f in ["status.md", "stale.md"] {
            touch(&dir.join(f));
        }

        let args = args_for(&dir, &["status.md"], &[], false, None);
        let first = run(&args, tmp.path()).unwrap();
        assert_eq!(first.removed, vec!["stale.md"]);

        let second = run(&args, tmp.path()).unwrap();
        assert!(second.removed.is_empty());
        assert_eq!(second.kept, vec!["status.md"]);
    }

    #[test]
    fn pinned_relative_path_works_in_recursive_walk() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("framework");
        touch(&dir.join("rules/security-backend.md"));
        touch(&dir.join("rules/adopter-overrides.md"));
        touch(&dir.join("rules/legacy-rule.md"));

        let args = args_for(
            &dir,
            &["rules/security-backend.md"],
            &["rules/adopter-overrides.md"],
            true,
            None,
        );
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.removed, vec!["rules/legacy-rule.md"]);
        assert_eq!(result.kept, vec!["rules/security-backend.md"]);
        assert_eq!(result.pinned_kept, vec!["rules/adopter-overrides.md"]);
    }

    #[test]
    fn target_that_is_a_file_errors_out() {
        let tmp = tempfile::tempdir().unwrap();
        let file = tmp.path().join("not-a-dir.md");
        fs::write(&file, "x\n").unwrap();
        let args = args_for(&file, &[], &[], false, None);
        let err = run(&args, tmp.path()).unwrap_err();
        match err {
            PrimitiveError::Io { source, .. } => {
                assert!(source.to_string().contains("not a directory"));
            }
            other => panic!("expected Io error, got {other:?}"),
        }
    }

    #[test]
    fn relative_directory_resolves_under_repo_root() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("cmds");
        for f in ["status.md", "stale.md"] {
            touch(&dir.join(f));
        }
        // Pass the directory as a repo-relative path, the bootstrap's shape.
        let args = EnforceManifestArgs {
            directory: "cmds".into(),
            expected: vec!["status.md".into()],
            pinned: vec![],
            recursive: false,
            glob_include: None,
        };
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.removed, vec!["stale.md"]);
        assert!(dir.join("status.md").exists());
    }

    #[test]
    fn absolute_directory_outside_repo_is_rejected_before_any_removal() {
        let tmp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let victim = outside.path().join("innocent.md");
        touch(&victim);

        let args = args_for(outside.path(), &[], &[], true, Some("*"));
        let err = run(&args, tmp.path()).unwrap_err();
        assert!(
            matches!(err, PrimitiveError::InvalidPath { .. }),
            "expected InvalidPath, got {err:?}"
        );
        assert!(victim.exists(), "no file outside the repo may be removed");
    }

    #[test]
    fn relative_directory_with_parent_component_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let args = EnforceManifestArgs {
            directory: "../escape".into(),
            expected: vec![],
            pinned: vec![],
            recursive: false,
            glob_include: None,
        };
        let err = run(&args, tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidPath { .. }));
    }

    #[test]
    fn absolute_directory_with_parent_component_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        // Absolute, `..`-laden path that would resolve back inside the
        // repo after normalization — still rejected (no `..` anywhere).
        let sneaky = tmp.path().join("cmds/../cmds");
        touch(&tmp.path().join("cmds/stale.md"));
        let args = args_for(&sneaky, &[], &[], false, None);
        let err = run(&args, tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidPath { .. }));
        assert!(tmp.path().join("cmds/stale.md").exists());
    }

    #[test]
    fn regex_metacharacter_inside_glob_is_treated_as_a_literal() {
        // A `[` in the glob is escaped, not interpreted as a regex
        // character class. The primitive must not panic and must not
        // reject the input.
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("d");
        touch(&dir.join("[bracket].md"));
        let args = args_for(&dir, &[], &[], false, Some("[bracket].md"));
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.removed, vec!["[bracket].md"]);
    }
}
