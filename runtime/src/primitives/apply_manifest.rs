//! `apply-manifest` — strategy-aware bulk substitute + write.
//!
//! Walks a typed manifest of `{ source, dest, strategy, keep-literals }`
//! entries, applies the entry's strategy against the destination tree,
//! and returns per-entry actions plus aggregate counts. Designed for
//! the `/govern` bootstrap, where the same primitive call replaces a
//! host-generated bash walker and three distinct write strategies.
//!
//! Strategies:
//!
//! - **update**: substitute placeholders, compare against the existing
//!   destination, write only when the result differs. Actions:
//!   `created` (dest absent) / `updated` (dest exists & differs) /
//!   `unchanged` (dest exists & matches — mtime preserved by not
//!   touching the file).
//! - **create**: substitute placeholders, write only when the
//!   destination is absent. Actions: `created` / `skipped-exists`.
//! - **skip-if-conflict**: write only when the destination is absent;
//!   substitution is NOT applied (these are adopter-owned templates the
//!   framework seeds but never edits afterward). Actions: `created` /
//!   `skipped-exists`.
//!
//! Cross-cutting concerns:
//!
//! - **Pinned exemption**: the args' `pinned` list short-circuits before
//!   any read or write — pinned entries record `skipped-pinned`
//!   regardless of strategy. Pinned is checked first because it's the
//!   absolute "do not touch" signal.
//! - **`keep-literals`**: per-entry list of substitution keys to
//!   exclude. Used to keep `{project}` and `{cli-config-dir}` literal
//!   in a self-installed `govern.md` so the next adopter's bootstrap
//!   substitutes them per their project, not this one's.
//! - **Source missing**: when the staging tree doesn't contain an
//!   entry's source, the action is `source-missing` (not an
//!   operational error — the host surfaces it so the operator can
//!   diagnose the upstream archive).
//!
//! Cross-platform: paths in the manifest use forward slashes; the
//! primitive joins them with the host OS's separator at write time.
//! Pinned matching is case-sensitive on Unix and case-insensitive on
//! Windows (matching NTFS semantics).

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::primitives::substitute_templates::apply_substitutions;
use crate::primitives::{PrimitiveError, Result, validate_no_traversal, write_atomic_bytes};
use crate::schema::primitives::{
    ApplyManifestArgs, ApplyManifestResult, ManifestEntry, ManifestEntryResult,
};

const ACTION_CREATED: &str = "created";
const ACTION_UPDATED: &str = "updated";
const ACTION_UNCHANGED: &str = "unchanged";
const ACTION_SKIPPED_EXISTS: &str = "skipped-exists";
const ACTION_SKIPPED_PINNED: &str = "skipped-pinned";
const ACTION_SOURCE_MISSING: &str = "source-missing";

/// Execute the `apply-manifest` primitive.
///
/// # Errors
///
/// - [`PrimitiveError::InvalidPath`] when any entry's `source` or `dest`
///   is absolute, empty, or contains a parent-directory component. The
///   manifest is fetched-artifact-class input (BE-INPUT-004): a `..`
///   entry would write outside the target root. Every entry is validated
///   before any filesystem operation so a bad entry halts the whole walk
///   with zero writes. The two *roots* stay absolute-capable as before.
/// - [`PrimitiveError::Io`] on local filesystem failures while reading
///   sources or writing destinations.
/// - [`PrimitiveError::UnknownManifestStrategy`] when an entry's
///   `strategy` field is not one of `update`, `create`, or
///   `skip-if-conflict`.
pub fn run(args: &ApplyManifestArgs, repo: &Path) -> Result<ApplyManifestResult> {
    for entry in &args.entries {
        validate_no_traversal(&entry.source)?;
        validate_no_traversal(&entry.dest)?;
    }
    let source_root = resolve_path(repo, &args.source_root);
    let target_root = resolve_path(repo, &args.target_root);

    let pinned: Vec<String> = args.pinned.iter().map(|p| normalize_dest_path(p)).collect();

    let mut entries_out = Vec::with_capacity(args.entries.len());
    let mut result = ApplyManifestResult::default();

    for entry in &args.entries {
        let action = process_entry(
            entry,
            &source_root,
            &target_root,
            &pinned,
            &args.substitutions,
        )?;
        match action {
            ACTION_CREATED => result.created = result.created.saturating_add(1),
            ACTION_UPDATED => result.updated = result.updated.saturating_add(1),
            ACTION_UNCHANGED => result.unchanged = result.unchanged.saturating_add(1),
            ACTION_SKIPPED_EXISTS => {
                result.skipped_exists = result.skipped_exists.saturating_add(1);
            }
            ACTION_SKIPPED_PINNED => {
                result.skipped_pinned = result.skipped_pinned.saturating_add(1);
            }
            ACTION_SOURCE_MISSING => {
                result.source_missing = result.source_missing.saturating_add(1);
            }
            _ => {}
        }
        entries_out.push(ManifestEntryResult {
            source: entry.source.clone(),
            dest: entry.dest.clone(),
            action: action.to_string(),
        });
    }

    result.entries = entries_out;
    Ok(result)
}

fn resolve_path(repo: &Path, p: &str) -> PathBuf {
    let candidate = Path::new(p);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo.join(candidate)
    }
}

fn normalize_dest_path(p: &str) -> String {
    p.replace('\\', "/")
}

fn pinned_match(pinned: &[String], dest: &str) -> bool {
    let normalized = normalize_dest_path(dest);
    if cfg!(windows) {
        pinned.iter().any(|p| p.eq_ignore_ascii_case(&normalized))
    } else {
        pinned.iter().any(|p| p == &normalized)
    }
}

fn process_entry(
    entry: &ManifestEntry,
    source_root: &Path,
    target_root: &Path,
    pinned: &[String],
    substitutions: &BTreeMap<String, String>,
) -> Result<&'static str> {
    // 1. Pinned short-circuit: do not read, do not write.
    if pinned_match(pinned, &entry.dest) {
        return Ok(ACTION_SKIPPED_PINNED);
    }

    // 2. Resolve and probe the source.
    let source_path = source_root.join(Path::new(&entry.source));
    let source_exists = source_path
        .try_exists()
        .map_err(|source| PrimitiveError::Io {
            path: source_path.clone(),
            source,
        })?;
    if !source_exists {
        return Ok(ACTION_SOURCE_MISSING);
    }

    // 3. Resolve and probe the destination.
    let dest_path = target_root.join(Path::new(&entry.dest));
    let dest_exists = dest_path
        .try_exists()
        .map_err(|source| PrimitiveError::Io {
            path: dest_path.clone(),
            source,
        })?;

    // 4. Dispatch by strategy.
    match entry.strategy.as_str() {
        "update" => apply_update(&source_path, &dest_path, dest_exists, entry, substitutions),
        "create" => apply_create(&source_path, &dest_path, dest_exists, entry, substitutions),
        "skip-if-conflict" => apply_skip_if_conflict(&source_path, &dest_path, dest_exists),
        other => Err(PrimitiveError::UnknownManifestStrategy {
            strategy: other.to_string(),
        }),
    }
}

fn apply_update(
    source: &Path,
    dest: &Path,
    dest_exists: bool,
    entry: &ManifestEntry,
    substitutions: &BTreeMap<String, String>,
) -> Result<&'static str> {
    let new_bytes = read_and_substitute(source, entry.keep_literals.as_deref(), substitutions)?;

    if dest_exists {
        let existing = fs::read(dest).map_err(|source| PrimitiveError::Io {
            path: dest.into(),
            source,
        })?;
        if existing == new_bytes {
            return Ok(ACTION_UNCHANGED);
        }
        write_atomic_bytes(dest, &new_bytes)?;
        mirror_source_mode(source, dest)?;
        Ok(ACTION_UPDATED)
    } else {
        write_atomic_bytes(dest, &new_bytes)?;
        mirror_source_mode(source, dest)?;
        Ok(ACTION_CREATED)
    }
}

fn apply_create(
    source: &Path,
    dest: &Path,
    dest_exists: bool,
    entry: &ManifestEntry,
    substitutions: &BTreeMap<String, String>,
) -> Result<&'static str> {
    if dest_exists {
        return Ok(ACTION_SKIPPED_EXISTS);
    }
    let new_bytes = read_and_substitute(source, entry.keep_literals.as_deref(), substitutions)?;
    write_atomic_bytes(dest, &new_bytes)?;
    mirror_source_mode(source, dest)?;
    Ok(ACTION_CREATED)
}

fn apply_skip_if_conflict(source: &Path, dest: &Path, dest_exists: bool) -> Result<&'static str> {
    if dest_exists {
        return Ok(ACTION_SKIPPED_EXISTS);
    }
    let bytes = fs::read(source).map_err(|src| PrimitiveError::Io {
        path: source.into(),
        source: src,
    })?;
    write_atomic_bytes(dest, &bytes)?;
    mirror_source_mode(source, dest)?;
    Ok(ACTION_CREATED)
}

/// Mirror the source file's permission bits onto a freshly-written
/// destination. `write_atomic_bytes` materializes the destination from a
/// new tempfile (mode `0600` on Unix), which strips the source's
/// executable bit. That is fatal for the generator scripts shipped via
/// the Shared Files manifest (`scripts/gen-*.sh`, `update` strategy): the
/// `govern-pre-commit` hook must be able to exec them, and on a fresh
/// adopter the `created` action would otherwise emit a non-executable
/// generator. Copying the source's `Permissions` (the readonly flag on
/// Windows, the full mode on Unix) restores `cp -p`-style fidelity.
///
/// Only called on the write paths (`created` / `updated`); the
/// `unchanged` path never touches the destination, so its mode — already
/// correct from the write that created it — is preserved untouched.
///
/// `pub(crate)` because `substitute-templates` shares the same
/// tempfile-mode problem and mirrors modes through this one helper.
pub(crate) fn mirror_source_mode(source: &Path, dest: &Path) -> Result<()> {
    let perms = fs::metadata(source)
        .map_err(|src| PrimitiveError::Io {
            path: source.into(),
            source: src,
        })?
        .permissions();
    fs::set_permissions(dest, perms).map_err(|src| PrimitiveError::Io {
        path: dest.into(),
        source: src,
    })
}

/// Read `source` and, if its bytes decode as UTF-8, apply the
/// substitution map (with `keep_literals` masking specific keys). Binary
/// files are passed through unchanged.
fn read_and_substitute(
    source: &Path,
    keep_literals: Option<&[String]>,
    substitutions: &BTreeMap<String, String>,
) -> Result<Vec<u8>> {
    let bytes = fs::read(source).map_err(|src| PrimitiveError::Io {
        path: source.into(),
        source: src,
    })?;
    match std::str::from_utf8(&bytes) {
        Ok(text) => {
            let effective = effective_substitutions(substitutions, keep_literals);
            let (substituted, _count) = apply_substitutions(text, &effective);
            Ok(substituted.into_bytes())
        }
        Err(_) => Ok(bytes),
    }
}

/// Build the per-entry substitution map by removing every key listed in
/// `keep_literals`. Listed keys absent from the input map are no-ops.
/// When `keep_literals` is `None` or empty, the input map is cloned
/// unchanged.
fn effective_substitutions(
    substitutions: &BTreeMap<String, String>,
    keep_literals: Option<&[String]>,
) -> BTreeMap<String, String> {
    match keep_literals {
        Some(keys) if !keys.is_empty() => {
            let mut filtered = substitutions.clone();
            for key in keys {
                filtered.remove(key);
            }
            filtered
        }
        _ => substitutions.clone(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::collections::BTreeMap;

    fn subs(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    fn entry(source: &str, dest: &str, strategy: &str) -> ManifestEntry {
        ManifestEntry {
            source: source.into(),
            dest: dest.into(),
            strategy: strategy.into(),
            keep_literals: None,
        }
    }

    fn args_for(
        src_root: &Path,
        dst_root: &Path,
        entries: Vec<ManifestEntry>,
        pinned: Vec<String>,
        substitutions: BTreeMap<String, String>,
    ) -> ApplyManifestArgs {
        ApplyManifestArgs {
            source_root: src_root.to_string_lossy().into_owned(),
            target_root: dst_root.to_string_lossy().into_owned(),
            entries,
            pinned,
            substitutions,
        }
    }

    /// Build a minimal source tree under `tmp/src/` and return its path.
    fn write_source(tmp: &Path, files: &[(&str, &str)]) -> PathBuf {
        let src = tmp.join("src");
        for (rel, content) in files {
            let path = src.join(rel);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::write(&path, content).unwrap();
        }
        src
    }

    #[test]
    fn update_creates_when_dest_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let src = write_source(tmp.path(), &[("a.md", "Hello {project}\n")]);
        let dst = tmp.path().join("dst");

        let args = args_for(
            &src,
            &dst,
            vec![entry("a.md", "a.md", "update")],
            vec![],
            subs(&[("project", "anvil")]),
        );
        let result = run(&args, tmp.path()).unwrap();

        assert_eq!(result.created, 1);
        assert_eq!(result.updated, 0);
        assert_eq!(result.unchanged, 0);
        assert_eq!(result.entries[0].action, "created");
        assert_eq!(
            fs::read_to_string(dst.join("a.md")).unwrap(),
            "Hello anvil\n"
        );
    }

    #[test]
    fn update_updates_when_dest_differs() {
        let tmp = tempfile::tempdir().unwrap();
        let src = write_source(tmp.path(), &[("a.md", "new {project}\n")]);
        let dst = tmp.path().join("dst");
        fs::create_dir_all(&dst).unwrap();
        fs::write(dst.join("a.md"), "stale body\n").unwrap();

        let args = args_for(
            &src,
            &dst,
            vec![entry("a.md", "a.md", "update")],
            vec![],
            subs(&[("project", "anvil")]),
        );
        let result = run(&args, tmp.path()).unwrap();

        assert_eq!(result.updated, 1);
        assert_eq!(result.entries[0].action, "updated");
        assert_eq!(fs::read_to_string(dst.join("a.md")).unwrap(), "new anvil\n");
    }

    #[test]
    fn update_is_unchanged_and_preserves_mtime_when_matched() {
        let tmp = tempfile::tempdir().unwrap();
        let src = write_source(tmp.path(), &[("a.md", "Hello {project}\n")]);
        let dst = tmp.path().join("dst");
        fs::create_dir_all(&dst).unwrap();
        // Pre-seed the destination with the post-substitution content so
        // `update` sees a match.
        fs::write(dst.join("a.md"), "Hello anvil\n").unwrap();
        let mtime_before = fs::metadata(dst.join("a.md")).unwrap().modified().unwrap();

        let args = args_for(
            &src,
            &dst,
            vec![entry("a.md", "a.md", "update")],
            vec![],
            subs(&[("project", "anvil")]),
        );
        let result = run(&args, tmp.path()).unwrap();

        assert_eq!(result.unchanged, 1);
        assert_eq!(result.entries[0].action, "unchanged");
        let mtime_after = fs::metadata(dst.join("a.md")).unwrap().modified().unwrap();
        assert_eq!(mtime_before, mtime_after, "unchanged must not rewrite");
    }

    #[test]
    fn create_writes_only_when_dest_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let src = write_source(
            tmp.path(),
            &[
                ("new.md", "new {project}\n"),
                ("existing.md", "{project} A\n"),
            ],
        );
        let dst = tmp.path().join("dst");
        fs::create_dir_all(&dst).unwrap();
        fs::write(dst.join("existing.md"), "ADOPTER content\n").unwrap();

        let args = args_for(
            &src,
            &dst,
            vec![
                entry("new.md", "new.md", "create"),
                entry("existing.md", "existing.md", "create"),
            ],
            vec![],
            subs(&[("project", "anvil")]),
        );
        let result = run(&args, tmp.path()).unwrap();

        assert_eq!(result.created, 1);
        assert_eq!(result.skipped_exists, 1);
        assert_eq!(result.entries[0].action, "created");
        assert_eq!(result.entries[1].action, "skipped-exists");
        assert_eq!(
            fs::read_to_string(dst.join("new.md")).unwrap(),
            "new anvil\n"
        );
        // Existing destination is preserved verbatim.
        assert_eq!(
            fs::read_to_string(dst.join("existing.md")).unwrap(),
            "ADOPTER content\n"
        );
    }

    #[test]
    fn skip_if_conflict_does_not_apply_substitutions() {
        let tmp = tempfile::tempdir().unwrap();
        // Template for an adopter-owned file containing `{project}`; the
        // primitive must seed it without substituting so the adopter can
        // edit the placeholder themselves.
        let src = write_source(
            tmp.path(),
            &[(
                "AGENTS.md",
                "# {project} Agents\n\nAdopter customizes here.\n",
            )],
        );
        let dst = tmp.path().join("dst");

        let args = args_for(
            &src,
            &dst,
            vec![entry("AGENTS.md", "AGENTS.md", "skip-if-conflict")],
            vec![],
            subs(&[("project", "anvil")]),
        );
        let result = run(&args, tmp.path()).unwrap();

        assert_eq!(result.created, 1);
        assert_eq!(result.entries[0].action, "created");
        let body = fs::read_to_string(dst.join("AGENTS.md")).unwrap();
        assert!(
            body.starts_with("# {project} Agents"),
            "skip-if-conflict must NOT apply substitution: got {body:?}"
        );

        // Second run with the dest in place: skipped-exists; existing
        // adopter content stays intact.
        fs::write(dst.join("AGENTS.md"), "ADOPTER edit\n").unwrap();
        let result2 = run(&args, tmp.path()).unwrap();
        assert_eq!(result2.skipped_exists, 1);
        assert_eq!(result2.entries[0].action, "skipped-exists");
        assert_eq!(
            fs::read_to_string(dst.join("AGENTS.md")).unwrap(),
            "ADOPTER edit\n"
        );
    }

    #[test]
    fn pinned_short_circuits_before_any_io_regardless_of_strategy() {
        let tmp = tempfile::tempdir().unwrap();
        let src = write_source(tmp.path(), &[("constitution.md", "new body\n")]);
        let dst = tmp.path().join("dst");
        fs::create_dir_all(&dst).unwrap();
        fs::write(dst.join("constitution.md"), "ADOPTER pinned body\n").unwrap();

        let args = args_for(
            &src,
            &dst,
            vec![entry("constitution.md", "constitution.md", "update")],
            vec!["constitution.md".to_string()],
            BTreeMap::new(),
        );
        let result = run(&args, tmp.path()).unwrap();

        assert_eq!(result.skipped_pinned, 1);
        assert_eq!(result.updated, 0);
        assert_eq!(result.entries[0].action, "skipped-pinned");
        // File untouched.
        assert_eq!(
            fs::read_to_string(dst.join("constitution.md")).unwrap(),
            "ADOPTER pinned body\n"
        );
    }

    #[test]
    fn pinned_wins_over_skip_if_conflict_label() {
        // Both pinned-match and dest-exists are true; the result label must
        // reflect pinned (the absolute "do not touch" signal) per the
        // scenario's tie-break rule.
        let tmp = tempfile::tempdir().unwrap();
        let src = write_source(tmp.path(), &[("AGENTS.md", "framework seed\n")]);
        let dst = tmp.path().join("dst");
        fs::create_dir_all(&dst).unwrap();
        fs::write(dst.join("AGENTS.md"), "adopter\n").unwrap();

        let args = args_for(
            &src,
            &dst,
            vec![entry("AGENTS.md", "AGENTS.md", "skip-if-conflict")],
            vec!["AGENTS.md".to_string()],
            BTreeMap::new(),
        );
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.entries[0].action, "skipped-pinned");
        assert_eq!(result.skipped_pinned, 1);
        assert_eq!(result.skipped_exists, 0);
    }

    #[test]
    fn pinned_wins_over_source_missing() {
        let tmp = tempfile::tempdir().unwrap();
        // No source file written — source-missing would normally fire.
        let src = write_source(tmp.path(), &[]);
        let dst = tmp.path().join("dst");

        let args = args_for(
            &src,
            &dst,
            vec![entry("gone.md", "gone.md", "update")],
            vec!["gone.md".to_string()],
            BTreeMap::new(),
        );
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.entries[0].action, "skipped-pinned");
    }

    #[test]
    fn source_missing_returns_action_not_error() {
        let tmp = tempfile::tempdir().unwrap();
        // Build a source root but omit the requested file. The primitive
        // must report `source-missing` rather than erroring the whole walk.
        let src = write_source(tmp.path(), &[("present.md", "ok\n")]);
        let dst = tmp.path().join("dst");

        let args = args_for(
            &src,
            &dst,
            vec![
                entry("present.md", "present.md", "update"),
                entry("absent.md", "absent.md", "update"),
            ],
            vec![],
            BTreeMap::new(),
        );
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.entries[1].action, "source-missing");
        assert_eq!(result.source_missing, 1);
        assert_eq!(result.created, 1);
    }

    #[test]
    fn keep_literals_preserves_named_placeholders_for_that_entry_only() {
        // Mirrors the bootstrap's `govern.md` self-install: the framework
        // installs the file with `{project}` and `{cli-config-dir}` kept
        // literal so the NEXT adopter's `/govern` substitutes them per
        // that adopter — not per this one.
        let tmp = tempfile::tempdir().unwrap();
        let src = write_source(
            tmp.path(),
            &[
                (
                    "govern.md",
                    "Project {project} writes to {cli-config-dir}/{project}-session.json (version {version}).\n",
                ),
                ("README.md", "{project} {version}\n"),
            ],
        );
        let dst = tmp.path().join("dst");

        let args = args_for(
            &src,
            &dst,
            vec![
                ManifestEntry {
                    source: "govern.md".into(),
                    dest: "govern.md".into(),
                    strategy: "update".into(),
                    keep_literals: Some(vec!["project".into(), "cli-config-dir".into()]),
                },
                entry("README.md", "README.md", "update"),
            ],
            vec![],
            subs(&[
                ("project", "anvil"),
                ("cli-config-dir", ".claude"),
                ("version", "0.3.0"),
            ]),
        );
        let result = run(&args, tmp.path()).unwrap();

        assert_eq!(result.created, 2);
        // govern.md keeps {project} and {cli-config-dir} literal but
        // still substitutes {version}.
        let installed = fs::read_to_string(dst.join("govern.md")).unwrap();
        assert!(
            installed.contains("{project}") && installed.contains("{cli-config-dir}"),
            "keep-literals must preserve named placeholders: got {installed:?}"
        );
        assert!(
            installed.contains("0.3.0"),
            "unlisted keys must still substitute: got {installed:?}"
        );
        // README has no keep-literals — every placeholder substituted.
        assert_eq!(
            fs::read_to_string(dst.join("README.md")).unwrap(),
            "anvil 0.3.0\n"
        );
    }

    #[test]
    fn keep_literals_for_unknown_keys_is_a_noop() {
        // Listing a key absent from the substitutions map is allowed and
        // produces no error.
        let tmp = tempfile::tempdir().unwrap();
        let src = write_source(tmp.path(), &[("a.md", "{project}\n")]);
        let dst = tmp.path().join("dst");
        let args = args_for(
            &src,
            &dst,
            vec![ManifestEntry {
                source: "a.md".into(),
                dest: "a.md".into(),
                strategy: "update".into(),
                keep_literals: Some(vec!["nonexistent-key".into()]),
            }],
            vec![],
            subs(&[("project", "anvil")]),
        );
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.created, 1);
        assert_eq!(fs::read_to_string(dst.join("a.md")).unwrap(), "anvil\n");
    }

    #[test]
    fn traversal_in_entry_dest_halts_before_any_write() {
        let tmp = tempfile::tempdir().unwrap();
        let src = write_source(tmp.path(), &[("ok.md", "fine\n"), ("evil.md", "payload\n")]);
        let dst = tmp.path().join("dst");

        // The good entry comes FIRST; validation must still reject the
        // whole manifest before writing anything.
        let args = args_for(
            &src,
            &dst,
            vec![
                entry("ok.md", "ok.md", "update"),
                entry("evil.md", "../outside.md", "update"),
            ],
            vec![],
            BTreeMap::new(),
        );
        let err = run(&args, tmp.path()).unwrap_err();
        assert!(
            matches!(err, PrimitiveError::InvalidPath { .. }),
            "expected InvalidPath, got {err:?}"
        );
        assert!(!dst.join("ok.md").exists(), "no entry may write");
        assert!(!tmp.path().join("outside.md").exists());
    }

    #[test]
    fn traversal_in_entry_source_is_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let src = write_source(tmp.path(), &[]);
        // A secret outside the source root that a `..` source would leak
        // into the target tree.
        fs::write(tmp.path().join("secret.md"), "secret\n").unwrap();
        let dst = tmp.path().join("dst");

        let args = args_for(
            &src,
            &dst,
            vec![entry("../secret.md", "leaked.md", "update")],
            vec![],
            BTreeMap::new(),
        );
        let err = run(&args, tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidPath { .. }));
        assert!(!dst.join("leaked.md").exists());
    }

    #[test]
    fn absolute_entry_paths_are_rejected() {
        let tmp = tempfile::tempdir().unwrap();
        let src = write_source(tmp.path(), &[("a.md", "x\n")]);
        let dst = tmp.path().join("dst");
        let abs_dest = tmp.path().join("elsewhere.md");

        let args = args_for(
            &src,
            &dst,
            vec![entry("a.md", abs_dest.to_string_lossy().as_ref(), "update")],
            vec![],
            BTreeMap::new(),
        );
        let err = run(&args, tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidPath { .. }));
        assert!(!abs_dest.exists());
    }

    #[test]
    fn unknown_strategy_errors_the_walk() {
        let tmp = tempfile::tempdir().unwrap();
        let src = write_source(tmp.path(), &[("a.md", "x\n")]);
        let dst = tmp.path().join("dst");
        let args = args_for(
            &src,
            &dst,
            vec![entry("a.md", "a.md", "replace-everywhere")],
            vec![],
            BTreeMap::new(),
        );
        let err = run(&args, tmp.path()).unwrap_err();
        match err {
            PrimitiveError::UnknownManifestStrategy { strategy } => {
                assert_eq!(strategy, "replace-everywhere");
            }
            other => panic!("expected UnknownManifestStrategy, got {other:?}"),
        }
    }

    #[test]
    fn full_re_run_is_idempotent_with_unchanged_actions() {
        // After a first-run that creates every dest, the second run with
        // identical inputs records `unchanged` (update strategy) and
        // `skipped-exists` (create / skip-if-conflict strategies) — no
        // file is rewritten.
        let tmp = tempfile::tempdir().unwrap();
        let src = write_source(
            tmp.path(),
            &[
                ("u.md", "Hello {project}\n"),
                ("c.md", "Init {project}\n"),
                ("s.md", "Adopter {project}\n"),
            ],
        );
        let dst = tmp.path().join("dst");

        let args = args_for(
            &src,
            &dst,
            vec![
                entry("u.md", "u.md", "update"),
                entry("c.md", "c.md", "create"),
                entry("s.md", "s.md", "skip-if-conflict"),
            ],
            vec![],
            subs(&[("project", "anvil")]),
        );

        let first = run(&args, tmp.path()).unwrap();
        assert_eq!(first.created, 3);
        // Sample mtimes after the first write.
        let mtime_u = fs::metadata(dst.join("u.md")).unwrap().modified().unwrap();
        let mtime_c = fs::metadata(dst.join("c.md")).unwrap().modified().unwrap();
        let mtime_s = fs::metadata(dst.join("s.md")).unwrap().modified().unwrap();

        let second = run(&args, tmp.path()).unwrap();
        assert_eq!(second.unchanged, 1);
        assert_eq!(second.skipped_exists, 2);
        assert_eq!(second.created, 0);
        // mtimes preserved on the idempotent re-run.
        assert_eq!(
            fs::metadata(dst.join("u.md")).unwrap().modified().unwrap(),
            mtime_u
        );
        assert_eq!(
            fs::metadata(dst.join("c.md")).unwrap().modified().unwrap(),
            mtime_c
        );
        assert_eq!(
            fs::metadata(dst.join("s.md")).unwrap().modified().unwrap(),
            mtime_s
        );
    }

    #[test]
    fn binary_source_files_are_copied_unchanged_in_update_strategy() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        fs::create_dir_all(&src).unwrap();
        let bin = [0x80_u8, 0x81, 0xff, 0x00, 0x42];
        fs::write(src.join("logo.png"), bin).unwrap();
        let dst = tmp.path().join("dst");

        let args = args_for(
            &src,
            &dst,
            vec![entry("logo.png", "logo.png", "update")],
            vec![],
            subs(&[("project", "anvil")]),
        );
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.created, 1);
        assert_eq!(fs::read(dst.join("logo.png")).unwrap(), bin);
    }

    #[cfg(unix)]
    #[test]
    fn source_executable_bit_propagates_to_dest_on_create_and_update() {
        // Regression: `write_atomic_bytes` lands every write as mode 0600,
        // stripping +x. The Shared Files manifest ships `scripts/gen-*.sh`
        // with `update` strategy; the dest must stay executable so the
        // govern-pre-commit hook can run it.
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let src = write_source(tmp.path(), &[("gen.sh", "#!/bin/sh\necho {project}\n")]);
        fs::set_permissions(src.join("gen.sh"), fs::Permissions::from_mode(0o755)).unwrap();
        let dst = tmp.path().join("dst");

        let args = args_for(
            &src,
            &dst,
            vec![entry("gen.sh", "scripts/gen.sh", "update")],
            vec![],
            subs(&[("project", "anvil")]),
        );

        // create (dest absent) keeps the executable bit.
        let first = run(&args, tmp.path()).unwrap();
        assert_eq!(first.created, 1);
        let mode = fs::metadata(dst.join("scripts/gen.sh"))
            .unwrap()
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0, "created dest must keep +x, got {mode:o}");

        // update (dest exists, bytes differ) keeps the executable bit.
        fs::write(src.join("gen.sh"), "#!/bin/sh\necho {project} v2\n").unwrap();
        fs::set_permissions(src.join("gen.sh"), fs::Permissions::from_mode(0o755)).unwrap();
        let second = run(&args, tmp.path()).unwrap();
        assert_eq!(second.updated, 1);
        let mode = fs::metadata(dst.join("scripts/gen.sh"))
            .unwrap()
            .permissions()
            .mode();
        assert_ne!(mode & 0o111, 0, "updated dest must keep +x, got {mode:o}");
    }

    #[cfg(unix)]
    #[test]
    fn skip_if_conflict_propagates_source_executable_bit() {
        use std::os::unix::fs::PermissionsExt;

        let tmp = tempfile::tempdir().unwrap();
        let src = write_source(tmp.path(), &[("hook.sh", "#!/bin/sh\nexit 0\n")]);
        fs::set_permissions(src.join("hook.sh"), fs::Permissions::from_mode(0o755)).unwrap();
        let dst = tmp.path().join("dst");

        let args = args_for(
            &src,
            &dst,
            vec![entry("hook.sh", ".githooks/hook.sh", "skip-if-conflict")],
            vec![],
            BTreeMap::new(),
        );
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.created, 1);
        let mode = fs::metadata(dst.join(".githooks/hook.sh"))
            .unwrap()
            .permissions()
            .mode();
        assert_ne!(
            mode & 0o111,
            0,
            "seeded hook must be executable, got {mode:o}"
        );
    }

    #[test]
    fn nested_destination_directories_are_created_on_demand() {
        let tmp = tempfile::tempdir().unwrap();
        let src = write_source(
            tmp.path(),
            &[("framework/commands/status.md", "/{project}:status\n")],
        );
        let dst = tmp.path().join("dst");

        let args = args_for(
            &src,
            &dst,
            vec![entry(
                "framework/commands/status.md",
                "framework/commands/status.md",
                "update",
            )],
            vec![],
            subs(&[("project", "anvil")]),
        );
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.created, 1);
        assert_eq!(
            fs::read_to_string(dst.join("framework/commands/status.md")).unwrap(),
            "/anvil:status\n"
        );
    }
}
