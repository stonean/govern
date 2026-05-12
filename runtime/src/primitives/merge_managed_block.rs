//! `merge-managed-block` — idempotently install or update a
//! framework-managed block in an arbitrary text file using a
//! configurable marker shape.
//!
//! Generalizes [`crate::primitives::merge_claude_md`] to support two
//! marker styles:
//!
//! - **`html-comment`** (default, the legacy `merge-claude-md`
//!   behavior): paired `<!-- BEGIN {marker} -->` / `<!-- END {marker}
//!   -->` markers; body between them is the managed region.
//! - **`line-prefix`**: a single `# {marker}` line preamble followed
//!   by the block, terminated by the next blank line or EOF. Matches
//!   `.gitignore` and `.gitattributes` conventions where a `#` line
//!   serves as both a comment and an inline section header.
//!
//! In either style the primitive chooses one of four actions:
//!
//! - **created**: the file did not exist; a fresh file is written
//!   containing only the marker(s) and the supplied block.
//! - **inserted**: the file existed but no managed block was present;
//!   the marker(s) plus block are appended after a blank-line
//!   separator.
//! - **updated**: a managed block was present and its body differed
//!   from the supplied block; the body is replaced. Content outside
//!   the managed region is preserved byte-for-byte (subject to the
//!   trailing-newline normalization noted below).
//! - **unchanged**: a managed block was present and its body matched;
//!   the file is not rewritten (preserves mtime, idempotent for build
//!   tools).
//!
//! Trailing-newline normalization: when the primitive writes, the file
//! ends with exactly one trailing newline regardless of marker style.
//! For `line-prefix`, the block is followed by a single blank line if
//! subsequent content exists, or nothing if the block is at EOF — so
//! `.gitignore` callers don't need to pre-pad their block.
//!
//! `html-comment` markers must be balanced: a BEGIN without an END
//! yields [`PrimitiveError::MalformedMarkers`] — that's an
//! adopter-edit error the primitive refuses to silently repair.

use std::path::{Path, PathBuf};

use crate::primitives::{PrimitiveError, Result, read_text, write_atomic};
use crate::schema::primitives::{MergeManagedBlockArgs, MergeManagedBlockResult};

const DEFAULT_MARKER: &str = "govern-managed";
const STYLE_HTML_COMMENT: &str = "html-comment";
const STYLE_LINE_PREFIX: &str = "line-prefix";

/// Execute the `merge-managed-block` primitive.
///
/// # Errors
///
/// - [`PrimitiveError::Io`] on local filesystem failures.
/// - [`PrimitiveError::MalformedMarkers`] when an `html-comment` style
///   file has a BEGIN marker without an END (or vice versa).
/// - [`PrimitiveError::UnknownManifestStrategy`] when `marker-style`
///   is not one of `html-comment` or `line-prefix`. The error type is
///   reused (the field name is generic enough — "unknown strategy" —
///   to cover both manifest strategies and marker styles); the message
///   text makes the surface explicit.
pub fn run(args: &MergeManagedBlockArgs, repo: &Path) -> Result<MergeManagedBlockResult> {
    let path = resolve_path(repo, &args.path);
    let marker = args
        .marker
        .as_deref()
        .map_or_else(|| DEFAULT_MARKER.to_string(), str::to_string);
    let style_str = args
        .marker_style
        .as_deref()
        .unwrap_or(STYLE_HTML_COMMENT)
        .to_string();
    let style = parse_style(&style_str)?;
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

    let (new_content, action) = compute_merge(
        existing.as_deref(),
        style,
        &marker,
        &normalized_block,
        &path,
    )?;
    if let Some(content) = new_content {
        write_atomic(&path, &content)?;
    }

    Ok(MergeManagedBlockResult {
        path: path.to_string_lossy().into_owned(),
        action: action.into(),
        marker,
        marker_style: style_str,
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

#[derive(Clone, Copy, Debug)]
enum MarkerStyle {
    HtmlComment,
    LinePrefix,
}

fn parse_style(s: &str) -> Result<MarkerStyle> {
    match s {
        STYLE_HTML_COMMENT => Ok(MarkerStyle::HtmlComment),
        STYLE_LINE_PREFIX => Ok(MarkerStyle::LinePrefix),
        other => Err(PrimitiveError::UnknownManifestStrategy {
            strategy: format!("marker-style '{other}' (expected 'html-comment' or 'line-prefix')"),
        }),
    }
}

/// Compute the post-merge file content and the action label. Returns
/// `(Some(content), action)` when the file should be written, or
/// `(None, "unchanged")` when no write is needed.
fn compute_merge(
    existing: Option<&str>,
    style: MarkerStyle,
    marker: &str,
    block: &str,
    path: &Path,
) -> Result<(Option<String>, &'static str)> {
    match existing {
        None => Ok((Some(format_fresh(style, marker, block)), "created")),
        Some(text) => match style {
            MarkerStyle::HtmlComment => merge_html_comment(text, marker, block, path),
            MarkerStyle::LinePrefix => Ok(merge_line_prefix(text, marker, block)),
        },
    }
}

fn format_fresh(style: MarkerStyle, marker: &str, block: &str) -> String {
    match style {
        MarkerStyle::HtmlComment => {
            format!("<!-- BEGIN {marker} -->\n{block}\n<!-- END {marker} -->\n")
        }
        MarkerStyle::LinePrefix => format!("# {marker}\n{block}\n"),
    }
}

fn merge_html_comment(
    text: &str,
    marker: &str,
    block: &str,
    path: &Path,
) -> Result<(Option<String>, &'static str)> {
    let begin = format!("<!-- BEGIN {marker} -->");
    let end = format!("<!-- END {marker} -->");
    match (text.find(&begin), text.find(&end)) {
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
    }
}

/// Locate a `# {marker}` line in `text` and the body that follows it
/// (up to the next blank line or EOF). Returns `Some((line_start,
/// body_end, body))` where `line_start` is the byte offset of the
/// `#` character, `body_end` is the byte offset of the blank-line
/// terminator (or `text.len()` at EOF), and `body` is the body trimmed
/// of leading/trailing newlines.
fn find_line_prefix_block<'a>(text: &'a str, marker: &str) -> Option<(usize, usize, &'a str)> {
    let header = format!("# {marker}");
    let mut offset = 0;
    while offset < text.len() {
        let rest = &text[offset..];
        let line_end = rest.find('\n').map_or(rest.len(), |i| i);
        let raw_line = &rest[..line_end];
        let line = raw_line.trim_end_matches('\r');
        if line == header {
            let line_start = offset;
            let body_start = offset + line_end + usize::from(line_end < rest.len());
            let body_end = find_blank_line(text, body_start);
            let body = text[body_start..body_end].trim_matches('\n');
            return Some((line_start, body_end, body));
        }
        offset += line_end + usize::from(line_end < rest.len());
    }
    None
}

/// From `start`, find the byte offset of the first blank line, or
/// `text.len()` if none is found before EOF. A blank line is a `\n` or
/// `\r\n` not preceded by any other content on that line.
fn find_blank_line(text: &str, start: usize) -> usize {
    let mut offset = start;
    while offset < text.len() {
        let rest = &text[offset..];
        let line_end = rest.find('\n').map_or(rest.len(), |i| i);
        let raw_line = &rest[..line_end];
        let line = raw_line.trim_end_matches('\r');
        if line.is_empty() {
            return offset;
        }
        offset += line_end + usize::from(line_end < rest.len());
    }
    text.len()
}

fn merge_line_prefix(text: &str, marker: &str, block: &str) -> (Option<String>, &'static str) {
    let header = format!("# {marker}");
    match find_line_prefix_block(text, marker) {
        None => {
            // Pad so the appended block is separated by exactly one
            // blank line. Empty file or file already ending in a
            // blank line: no padding. Single trailing newline: one
            // more. No trailing newline: two more.
            let separator = if text.is_empty() || text.ends_with("\n\n") {
                ""
            } else if text.ends_with('\n') {
                "\n"
            } else {
                "\n\n"
            };
            let combined = format!("{text}{separator}{header}\n{block}\n");
            (Some(combined), "inserted")
        }
        Some((line_start, body_end, body)) => {
            if body == block {
                return (None, "unchanged");
            }
            let before = &text[..line_start];
            let after = &text[body_end..];
            // Ensure exactly one blank line separates the block from
            // subsequent content; the file always ends with exactly
            // one trailing newline.
            let after_normalized = if after.is_empty() {
                String::new()
            } else if after.starts_with('\n') {
                after.to_string()
            } else {
                format!("\n{after}")
            };
            let combined = format!("{before}{header}\n{block}\n{after_normalized}");
            (Some(combined), "updated")
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;

    fn args(path: &Path, block: &str, style: Option<&str>) -> MergeManagedBlockArgs {
        MergeManagedBlockArgs {
            path: path.to_string_lossy().into_owned(),
            block: block.into(),
            marker: None,
            marker_style: style.map(str::to_string),
        }
    }

    // -- html-comment style — reproduces merge_claude_md behavior -----------

    #[test]
    fn html_first_run_creates_file_with_block() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        let result = run(
            &args(&path, "framework section\nline two", None),
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.action, "created");
        assert_eq!(result.marker, "govern-managed");
        assert_eq!(result.marker_style, "html-comment");
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("<!-- BEGIN govern-managed -->"));
        assert!(body.contains("framework section\nline two"));
        assert!(body.contains("<!-- END govern-managed -->"));
    }

    #[test]
    fn html_rerun_unchanged_preserves_mtime() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(
            &path,
            "<!-- BEGIN govern-managed -->\nblock body\n<!-- END govern-managed -->\n",
        )
        .unwrap();
        let mtime_before = fs::metadata(&path).unwrap().modified().unwrap();
        let result = run(&args(&path, "block body", None), tmp.path()).unwrap();
        assert_eq!(result.action, "unchanged");
        assert_eq!(
            fs::metadata(&path).unwrap().modified().unwrap(),
            mtime_before
        );
    }

    #[test]
    fn html_malformed_markers_error() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        fs::write(&path, "<!-- BEGIN govern-managed -->\nopen\n").unwrap();
        let err = run(&args(&path, "x", None), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::MalformedMarkers { .. }));
    }

    // -- line-prefix style — .gitignore / .gitattributes conventions --------

    #[test]
    fn line_prefix_first_run_creates_gitignore_shaped_file() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        let block = ".claude/\nspecs/.cache/";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "created");
        assert_eq!(result.marker_style, "line-prefix");
        let body = fs::read_to_string(&path).unwrap();
        assert_eq!(body, "# govern-managed\n.claude/\nspecs/.cache/\n");
    }

    #[test]
    fn line_prefix_appends_to_existing_gitignore_without_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        fs::write(&path, "node_modules/\n*.log\n").unwrap();
        let block = ".claude/\nspecs/.cache/";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "inserted");
        let body = fs::read_to_string(&path).unwrap();
        assert_eq!(
            body,
            "node_modules/\n*.log\n\n# govern-managed\n.claude/\nspecs/.cache/\n"
        );
    }

    #[test]
    fn line_prefix_rerun_with_matching_body_is_unchanged_and_preserves_mtime() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        fs::write(
            &path,
            "node_modules/\n\n# govern-managed\n.claude/\nspecs/.cache/\n",
        )
        .unwrap();
        let mtime_before = fs::metadata(&path).unwrap().modified().unwrap();
        let block = ".claude/\nspecs/.cache/";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "unchanged");
        let mtime_after = fs::metadata(&path).unwrap().modified().unwrap();
        assert_eq!(mtime_before, mtime_after, "unchanged must not rewrite");
    }

    #[test]
    fn line_prefix_updates_in_place_preserving_surrounding_content() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        fs::write(
            &path,
            "node_modules/\n\n# govern-managed\n.old/\n\n# user-tail-section\n*.tmp\n",
        )
        .unwrap();
        let block = ".claude/\nspecs/.cache/";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "updated");
        let body = fs::read_to_string(&path).unwrap();
        // Block updated; user-tail-section after the blank-line terminator is preserved verbatim.
        assert_eq!(
            body,
            "node_modules/\n\n# govern-managed\n.claude/\nspecs/.cache/\n\n# user-tail-section\n*.tmp\n"
        );
    }

    #[test]
    fn line_prefix_block_at_eof_has_exactly_one_trailing_newline() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        fs::write(&path, "user-pre\n\n# govern-managed\n.old/").unwrap();
        let block = ".claude/";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "updated");
        let body = fs::read_to_string(&path).unwrap();
        assert_eq!(body, "user-pre\n\n# govern-managed\n.claude/\n");
        assert!(
            body.ends_with('\n') && !body.ends_with("\n\n"),
            "must end with exactly one trailing newline: {body:?}"
        );
    }

    #[test]
    fn line_prefix_existing_file_no_trailing_newline_gets_padded() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        // No trailing newline on the existing content — the primitive
        // pads with one blank line before appending the managed block.
        fs::write(&path, "node_modules/").unwrap();
        let block = ".claude/";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "inserted");
        let body = fs::read_to_string(&path).unwrap();
        assert_eq!(body, "node_modules/\n\n# govern-managed\n.claude/\n");
    }

    #[test]
    fn line_prefix_custom_marker_is_honored() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        let result = run(
            &MergeManagedBlockArgs {
                path: path.to_string_lossy().into_owned(),
                block: ".tmp/".into(),
                marker: Some("anvil".into()),
                marker_style: Some("line-prefix".into()),
            },
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.action, "created");
        assert_eq!(result.marker, "anvil");
        let body = fs::read_to_string(&path).unwrap();
        assert_eq!(body, "# anvil\n.tmp/\n");
    }

    #[test]
    fn line_prefix_does_not_confuse_inline_hash_with_marker_line() {
        // A line like `foo # govern-managed` (the marker as a tail
        // comment) is NOT the marker — only a line that exactly equals
        // `# {marker}` counts. The primitive must append a fresh block.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        fs::write(&path, "user-ignore # govern-managed\n").unwrap();
        let result = run(&args(&path, ".claude/", Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "inserted");
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.starts_with("user-ignore # govern-managed\n"));
        assert!(body.contains("# govern-managed\n.claude/\n"));
    }

    #[test]
    fn unknown_marker_style_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("x.md");
        let err = run(&args(&path, "x", Some("yaml-block")), tmp.path()).unwrap_err();
        match err {
            PrimitiveError::UnknownManifestStrategy { strategy } => {
                assert!(strategy.contains("yaml-block"));
            }
            other => panic!("expected UnknownManifestStrategy, got {other:?}"),
        }
    }

    #[test]
    fn line_prefix_block_with_crlf_line_endings_in_existing_file() {
        // CRLF in the existing file — the marker detector strips the
        // trailing \r before equality, so the marker line `# govern-managed\r\n`
        // is recognized correctly.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        fs::write(
            &path,
            "node\r\n\r\n# govern-managed\r\n.old/\r\n\r\nuser-tail\r\n",
        )
        .unwrap();
        let block = ".claude/";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "updated");
        // Body is rewritten with LF newlines (the primitive normalizes
        // the managed region; the surrounding content is preserved
        // byte-for-byte, including its original CRLF endings).
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("# govern-managed\n.claude/\n"));
        // Surrounding lines kept verbatim.
        assert!(body.starts_with("node\r\n\r\n# govern-managed"));
        assert!(body.contains("user-tail\r\n"));
    }
}
