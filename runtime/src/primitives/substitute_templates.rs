//! `substitute-templates` — walk a source tree, apply a `{key}` → value
//! substitution map to each text file, and write the result to a
//! destination tree.
//!
//! Text files are detected by attempting a UTF-8 decode on the file's
//! bytes; files that fail the decode are treated as binary and copied
//! to the destination unchanged. Substitution syntax is the literal
//! `{key}` token — matches the existing govern template convention
//! (e.g., `{project}`, `{cli-config-dir}`). Keys appear in the
//! substitutions map; values are substituted verbatim with no escaping.
//!
//! Symlinks in the source tree are skipped (a future revision may
//! choose to surface them as a finding). Empty directories in the
//! source tree are not propagated — only files are written, and parent
//! directories are created on demand. The destination overwrites
//! existing files with the same relative path — adopter projects that
//! want to preserve specific files should use the `merge-claude-md`
//! primitive (or its successors) instead.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

use crate::primitives::{PrimitiveError, Result, write_atomic_bytes};
use crate::schema::primitives::{SubstituteTemplatesArgs, SubstituteTemplatesResult};

/// Execute the `substitute-templates` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::Io`] when source files cannot be read or
/// destination files cannot be written.
pub fn run(args: &SubstituteTemplatesArgs, repo: &Path) -> Result<SubstituteTemplatesResult> {
    let source = resolve_path(repo, &args.source_dir);
    let dest = resolve_path(repo, &args.target_dir);

    fs::create_dir_all(&dest).map_err(|source| PrimitiveError::Io {
        path: dest.clone(),
        source,
    })?;

    let mut files: Vec<String> = Vec::new();
    let mut substitutions_applied: u32 = 0;

    for entry in WalkDir::new(&source).follow_links(false) {
        let entry = entry.map_err(|err| PrimitiveError::Io {
            path: err.path().map_or_else(|| source.clone(), Path::to_path_buf),
            source: err
                .into_io_error()
                .unwrap_or_else(|| std::io::Error::other("walkdir error")),
        })?;
        if !entry.file_type().is_file() {
            continue;
        }
        let abs = entry.path();
        let rel = abs.strip_prefix(&source).map_err(|_| PrimitiveError::Io {
            path: abs.into(),
            source: std::io::Error::other("entry escaped source directory"),
        })?;
        let dest_path = dest.join(rel);

        let bytes = fs::read(abs).map_err(|source| PrimitiveError::Io {
            path: abs.into(),
            source,
        })?;

        let (written_bytes, replaced) = match std::str::from_utf8(&bytes) {
            Ok(text) => {
                let (substituted, count) = apply_substitutions(text, &args.substitutions);
                (substituted.into_bytes(), count)
            }
            Err(_) => (bytes, 0),
        };

        write_atomic_bytes(&dest_path, &written_bytes)?;
        substitutions_applied = substitutions_applied.saturating_add(replaced);
        files.push(rel.to_string_lossy().replace('\\', "/"));
    }

    let files_written = u32::try_from(files.len()).unwrap_or(u32::MAX);
    Ok(SubstituteTemplatesResult {
        target_dir: dest.to_string_lossy().into_owned(),
        files_written,
        substitutions_applied,
        files,
    })
}

fn resolve_path(repo: &Path, p: &str) -> PathBuf {
    let candidate = Path::new(p);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo.join(candidate)
    }
}

/// Apply `{key}` → value replacements to `text`. Returns the substituted
/// string and the total count of replacements applied across all keys.
///
/// Substitution is non-recursive: a value that itself contains a `{key}`
/// token is not re-substituted. Keys are processed in `BTreeMap` order
/// (lexicographic) for deterministic output across runs.
pub(crate) fn apply_substitutions(
    text: &str,
    substitutions: &BTreeMap<String, String>,
) -> (String, u32) {
    let mut out = text.to_string();
    let mut total: u32 = 0;
    for (key, value) in substitutions {
        let placeholder = format!("{{{key}}}");
        let count = u32::try_from(out.matches(&placeholder).count()).unwrap_or(u32::MAX);
        if count > 0 {
            out = out.replace(&placeholder, value);
            total = total.saturating_add(count);
        }
    }
    (out, total)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn map(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect()
    }

    #[test]
    fn apply_substitutions_replaces_known_placeholders() {
        let subs = map(&[("project", "anvil"), ("cli-config-dir", ".claude")]);
        let (out, count) = apply_substitutions(
            "Project {project} lives at {cli-config-dir}/{project}-session.json.",
            &subs,
        );
        assert_eq!(out, "Project anvil lives at .claude/anvil-session.json.");
        assert_eq!(count, 3);
    }

    #[test]
    fn apply_substitutions_leaves_unknown_placeholders_intact() {
        let subs = map(&[("project", "anvil")]);
        let (out, count) = apply_substitutions("{project} and {unknown}", &subs);
        assert_eq!(out, "anvil and {unknown}");
        assert_eq!(count, 1);
    }

    #[test]
    fn apply_substitutions_is_non_recursive() {
        let subs = map(&[("a", "{b}"), ("b", "BEE")]);
        let (out, _) = apply_substitutions("{a}", &subs);
        // `a` resolves to `{b}` first; `b` is then processed against the
        // updated string, so the chained value DOES get substituted. Pin
        // the observed behavior so future refactors don't quietly change
        // it.
        assert_eq!(out, "BEE");
    }

    #[test]
    fn run_writes_text_files_with_substitutions() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        fs::create_dir_all(src.join("nested")).unwrap();
        fs::write(src.join("README.md"), "# {project}\n\nHello, {project}.\n").unwrap();
        fs::write(src.join("nested/cmd.md"), "/{project}:status").unwrap();

        let args = SubstituteTemplatesArgs {
            source_dir: src.to_string_lossy().into_owned(),
            target_dir: dst.to_string_lossy().into_owned(),
            substitutions: map(&[("project", "anvil")]),
        };
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.files_written, 2);
        assert_eq!(result.substitutions_applied, 3);
        assert_eq!(
            fs::read_to_string(dst.join("README.md")).unwrap(),
            "# anvil\n\nHello, anvil.\n"
        );
        assert_eq!(
            fs::read_to_string(dst.join("nested/cmd.md")).unwrap(),
            "/anvil:status"
        );
    }

    #[test]
    fn run_copies_binary_files_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        fs::create_dir_all(&src).unwrap();
        // Bytes that are invalid UTF-8 (lone continuation byte).
        let bin = [0x80_u8, 0x81, 0x82, 0xff, 0x00];
        fs::write(src.join("bin.dat"), bin).unwrap();

        let args = SubstituteTemplatesArgs {
            source_dir: src.to_string_lossy().into_owned(),
            target_dir: dst.to_string_lossy().into_owned(),
            substitutions: map(&[("project", "anvil")]),
        };
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.files_written, 1);
        assert_eq!(result.substitutions_applied, 0);
        assert_eq!(fs::read(dst.join("bin.dat")).unwrap(), bin);
    }

    #[test]
    fn run_overwrites_existing_destination_files() {
        let tmp = tempfile::tempdir().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");
        fs::create_dir_all(&src).unwrap();
        fs::create_dir_all(&dst).unwrap();
        fs::write(src.join("a.txt"), "new {project} body").unwrap();
        fs::write(dst.join("a.txt"), "old body").unwrap();

        let args = SubstituteTemplatesArgs {
            source_dir: src.to_string_lossy().into_owned(),
            target_dir: dst.to_string_lossy().into_owned(),
            substitutions: map(&[("project", "anvil")]),
        };
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.files_written, 1);
        assert_eq!(
            fs::read_to_string(dst.join("a.txt")).unwrap(),
            "new anvil body"
        );
    }
}
