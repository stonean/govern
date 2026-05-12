//! `merge-claude-md` — compat shim that delegates to
//! [`crate::primitives::merge_managed_block`] with
//! `marker-style: "html-comment"` and `marker: "govern-managed"`.
//!
//! The managed region is delimited by paired HTML-comment markers
//! `<!-- BEGIN govern-managed -->` and `<!-- END govern-managed -->`.
//! All four actions — `created`, `inserted`, `updated`, `unchanged` —
//! are inherited from `merge-managed-block`.
//!
//! Retained as a thin wrapper so existing callers (the bootstrap
//! fixture, parity goldens, and any host scripts) keep working
//! unchanged. Slated for removal in the next major `gvrn` release;
//! new callers should reach for `merge-managed-block` directly so
//! they can opt into other marker styles.

use std::path::Path;

use crate::primitives::Result;
use crate::primitives::merge_managed_block;
use crate::schema::primitives::{MergeClaudeMdArgs, MergeClaudeMdResult, MergeManagedBlockArgs};

/// Execute the `merge-claude-md` primitive.
///
/// # Errors
///
/// Forwarded verbatim from
/// [`crate::primitives::merge_managed_block::run`]:
///
/// - [`crate::primitives::PrimitiveError::Io`] on local filesystem failures.
/// - [`crate::primitives::PrimitiveError::MalformedMarkers`] when the
///   file contains a BEGIN marker without an END (or vice versa).
pub fn run(args: &MergeClaudeMdArgs, repo: &Path) -> Result<MergeClaudeMdResult> {
    let delegated = MergeManagedBlockArgs {
        path: args.path.clone(),
        block: args.block.clone(),
        marker: args.marker.clone(),
        marker_style: None, // defaults to html-comment in merge-managed-block
    };
    let result = merge_managed_block::run(&delegated, repo)?;
    Ok(MergeClaudeMdResult {
        path: result.path,
        action: result.action,
        marker: result.marker,
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::primitives::PrimitiveError;
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
        assert!(matches!(err, PrimitiveError::MalformedMarkers { .. }));
    }
}
