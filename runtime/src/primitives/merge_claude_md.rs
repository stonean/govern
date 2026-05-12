//! `merge-claude-md` — idempotently install or update a
//! framework-managed block in the adopter's `CLAUDE.md`.
//!
//! The managed region is delimited by paired HTML-comment markers
//! `<!-- BEGIN {marker} -->` and `<!-- END {marker} -->` (marker name
//! defaults to `govern-managed`). On each invocation the primitive
//! chooses one of four actions:
//!
//! - **created**: the file did not exist; a new file containing only
//!   the marker pair plus the supplied block is written.
//! - **inserted**: the file existed but contained no markers; the
//!   marker pair plus block is appended to the end after a blank-line
//!   separator (with a single trailing newline).
//! - **updated**: markers were present; the body between them differed
//!   from the supplied block, so the markers' contents are replaced.
//!   Content outside the markers is preserved byte-for-byte.
//! - **unchanged**: markers were present and the body matched; the
//!   file is not rewritten (preserves mtime, idempotent for tools).
//!
//! Markers must be balanced: a BEGIN without an END (or vice versa)
//! yields [`PrimitiveError::MalformedMarkers`] — that's an adopter-edit
//! error the primitive refuses to silently repair.

use std::path::{Path, PathBuf};

use crate::primitives::{PrimitiveError, Result, read_text, write_atomic};
use crate::schema::primitives::{MergeClaudeMdArgs, MergeClaudeMdResult};

const DEFAULT_MARKER: &str = "govern-managed";

/// Execute the `merge-claude-md` primitive.
///
/// # Errors
///
/// - [`PrimitiveError::Io`] on local filesystem failures.
/// - [`PrimitiveError::MalformedMarkers`] when the file contains a BEGIN marker
///   without an END (or vice versa).
pub fn run(args: &MergeClaudeMdArgs, repo: &Path) -> Result<MergeClaudeMdResult> {
    let path = resolve_path(repo, &args.path);
    let marker = args
        .marker
        .as_deref()
        .map_or_else(|| DEFAULT_MARKER.to_string(), str::to_string);
    let begin = format!("<!-- BEGIN {marker} -->");
    let end = format!("<!-- END {marker} -->");
    let normalized_block = normalize_block(&args.block);

    let existing = match path.try_exists() {
        Ok(true) => Some(read_text(&path)?),
        Ok(false) => None,
        Err(source) => {
            return Err(PrimitiveError::Io {
                path: path.clone(),
                source,
            });
        }
    };

    let (new_content, action) =
        compute_merge(existing.as_deref(), &begin, &end, &normalized_block, &path)?;
    if let Some(content) = new_content {
        write_atomic(&path, &content)?;
    }

    Ok(MergeClaudeMdResult {
        path: path.to_string_lossy().into_owned(),
        action: action.into(),
        marker,
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

fn normalize_block(block: &str) -> String {
    block.trim_matches('\n').to_string()
}

/// Compute the post-merge file content and the action label. Returns
/// `(Some(content), action)` when the file should be written, or
/// `(None, "unchanged")` when no write is needed.
fn compute_merge(
    existing: Option<&str>,
    begin: &str,
    end: &str,
    block: &str,
    path: &Path,
) -> Result<(Option<String>, &'static str)> {
    match existing {
        None => Ok((Some(format!("{begin}\n{block}\n{end}\n")), "created")),
        Some(text) => match (text.find(begin), text.find(end)) {
            (None, None) => {
                let separator = if text.ends_with('\n') { "\n" } else { "\n\n" };
                let combined = format!("{text}{separator}{begin}\n{block}\n{end}\n");
                Ok((Some(combined), "inserted"))
            }
            (Some(b_idx), Some(e_idx)) if b_idx < e_idx => {
                let before = &text[..b_idx];
                let inner_start = b_idx + begin.len();
                let inner = text[inner_start..e_idx].trim_matches('\n');
                let after = &text[e_idx + end.len()..];
                if inner == block {
                    return Ok((None, "unchanged"));
                }
                Ok((
                    Some(format!("{before}{begin}\n{block}\n{end}{after}")),
                    "updated",
                ))
            }
            (Some(_), Some(_)) => Err(PrimitiveError::MalformedMarkers {
                path: path.into(),
                reason: "END marker appears before BEGIN marker".into(),
            }),
            (Some(_), None) => Err(PrimitiveError::MalformedMarkers {
                path: path.into(),
                reason: "BEGIN marker present without matching END".into(),
            }),
            (None, Some(_)) => Err(PrimitiveError::MalformedMarkers {
                path: path.into(),
                reason: "END marker present without matching BEGIN".into(),
            }),
        },
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;

    fn args(path: &Path, block: &str) -> MergeClaudeMdArgs {
        MergeClaudeMdArgs {
            path: path.to_string_lossy().into_owned(),
            block: block.into(),
            marker: None,
        }
    }

    #[test]
    fn first_run_creates_file_with_block() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        let result = run(&args(&path, "framework section\nline two"), tmp.path()).unwrap();
        assert_eq!(result.action, "created");
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("<!-- BEGIN govern-managed -->"));
        assert!(body.contains("framework section\nline two"));
        assert!(body.contains("<!-- END govern-managed -->"));
    }

    #[test]
    fn first_run_existing_file_appends_block() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(&path, "# Existing\n\nUser content.\n").unwrap();
        let result = run(&args(&path, "framework section"), tmp.path()).unwrap();
        assert_eq!(result.action, "inserted");
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.starts_with("# Existing\n\nUser content.\n"));
        assert!(body.contains(
            "<!-- BEGIN govern-managed -->\nframework section\n<!-- END govern-managed -->"
        ));
    }

    #[test]
    fn rerun_with_same_block_is_unchanged() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(
            &path,
            "# user\n\n<!-- BEGIN govern-managed -->\nblock body\n<!-- END govern-managed -->\n",
        )
        .unwrap();
        let mtime_before = fs::metadata(&path).unwrap().modified().unwrap();
        let result = run(&args(&path, "block body"), tmp.path()).unwrap();
        assert_eq!(result.action, "unchanged");
        let mtime_after = fs::metadata(&path).unwrap().modified().unwrap();
        assert_eq!(mtime_before, mtime_after, "unchanged must not rewrite");
    }

    #[test]
    fn rerun_with_new_block_updates_in_place() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(
            &path,
            "# user\n\n<!-- BEGIN govern-managed -->\nold body\n<!-- END govern-managed -->\n\nuser footer.\n",
        )
        .unwrap();
        let result = run(&args(&path, "new body"), tmp.path()).unwrap();
        assert_eq!(result.action, "updated");
        let body = fs::read_to_string(&path).unwrap();
        assert!(
            body.contains("<!-- BEGIN govern-managed -->\nnew body\n<!-- END govern-managed -->")
        );
        assert!(body.starts_with("# user\n"));
        assert!(body.contains("user footer."));
    }

    #[test]
    fn malformed_begin_without_end_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(&path, "<!-- BEGIN govern-managed -->\nstuck open\n").unwrap();
        let err = run(&args(&path, "anything"), tmp.path()).unwrap_err();
        match err {
            PrimitiveError::MalformedMarkers { reason, .. } => {
                assert!(reason.contains("BEGIN marker present"), "got: {reason}");
            }
            other => panic!("expected MalformedMarkers, got {other:?}"),
        }
    }

    #[test]
    fn custom_marker_name_is_honored() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        let result = run(
            &MergeClaudeMdArgs {
                path: path.to_string_lossy().into_owned(),
                block: "other framework".into(),
                marker: Some("other-managed".into()),
            },
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.action, "created");
        assert_eq!(result.marker, "other-managed");
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("<!-- BEGIN other-managed -->"));
    }

    #[test]
    fn end_before_begin_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(
            &path,
            "<!-- END govern-managed -->\nsomething\n<!-- BEGIN govern-managed -->\n",
        )
        .unwrap();
        let err = run(&args(&path, "anything"), tmp.path()).unwrap_err();
        matches!(err, PrimitiveError::MalformedMarkers { .. });
    }
}
