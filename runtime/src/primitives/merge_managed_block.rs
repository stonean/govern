//! `merge-managed-block` — idempotently install or update a
//! framework-managed block in an arbitrary text file using a
//! configurable marker shape.
//!
//! Generalizes the retired `merge-claude-md` primitive (a thin wrapper
//! removed by scenario coverage-residue-cleanup once no command invoked
//! it) to support two marker styles:
//!
//! - **`html-comment`** (default, the legacy `merge-claude-md`
//!   behavior): paired `<!-- BEGIN {marker} -->` / `<!-- END {marker}
//!   -->` markers; body between them is the managed region.
//! - **`line-prefix`**: a single `# {marker}` line preamble followed
//!   by the block. The block's extent on disk is identified by aligning
//!   the on-disk blank-line-delimited subsections against the supplied
//!   block's subsections (see [`walk_body_extent`]); unmatched trailing
//!   canonical subsections are insertions, and the first unaligned
//!   on-disk group ends the block. This matches `.gitignore` /
//!   `.gitattributes` conventions where a `#` line serves as both a
//!   comment and an inline section header, and correctly handles blocks
//!   containing interior blank lines between subsections (the shipped
//!   `.gitignore` template is shaped this way).
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

use std::path::Path;

use crate::primitives::{PrimitiveError, Result, read_text, validate_no_traversal, write_atomic};
use crate::schema::primitives::{MergeManagedBlockArgs, MergeManagedBlockResult};

const DEFAULT_MARKER: &str = "govern-managed";
const STYLE_HTML_COMMENT: &str = "html-comment";
const STYLE_LINE_PREFIX: &str = "line-prefix";

/// Execute the `merge-managed-block` primitive.
///
/// # Errors
///
/// - [`PrimitiveError::InvalidPath`] when `path` is absolute, empty, or
///   contains a parent-directory component — the BE-INPUT-004
///   defense-in-depth check every path-taking primitive applies before
///   filesystem operations. `path` is repo-relative (`.gitignore`,
///   `CLAUDE.md`); no caller needs an out-of-repo destination.
/// - [`PrimitiveError::Io`] on local filesystem failures.
/// - [`PrimitiveError::MalformedMarkers`] when an `html-comment` style
///   file has a BEGIN marker without an END (or vice versa).
/// - [`PrimitiveError::UnknownManifestStrategy`] when `marker-style`
///   is not one of `html-comment` or `line-prefix`. The error type is
///   reused (the field name is generic enough — "unknown strategy" —
///   to cover both manifest strategies and marker styles); the message
///   text makes the surface explicit.
pub fn run(args: &MergeManagedBlockArgs, repo: &Path) -> Result<MergeManagedBlockResult> {
    validate_no_traversal(&args.path)?;
    let path = repo.join(&args.path);
    let marker = args
        .marker
        .as_deref()
        .map_or_else(|| DEFAULT_MARKER.to_string(), str::to_string);
    // The marker is interpolated into the managed-region delimiters
    // (`<!-- BEGIN {marker} -->` / `# {marker}`). A newline would split the
    // delimiter across lines, and a `-->` would close the HTML comment early —
    // either corrupts the region boundaries. Refuse both.
    super::validate_single_line("merge-managed-block", "marker", &marker)?;
    if marker.contains("-->") {
        return Err(PrimitiveError::InvalidArgument {
            primitive: "merge-managed-block".into(),
            argument: "marker".into(),
            reason: "must not contain `-->`, which would terminate the HTML-comment marker early"
                .into(),
        });
    }
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

    let (new_content, action_static, dedup_lines) = compute_merge(
        existing.as_deref(),
        style,
        &marker,
        &normalized_block,
        &path,
    )?;
    if let Some(content) = &new_content {
        write_atomic(&path, content)?;
    }

    let (dedup_removed, dedup_removed_lines) = match style {
        MarkerStyle::LinePrefix => {
            let count = u32::try_from(dedup_lines.len()).unwrap_or(u32::MAX);
            (Some(count), Some(dedup_lines))
        }
        MarkerStyle::HtmlComment => (None, None),
    };

    Ok(MergeManagedBlockResult {
        path: path.to_string_lossy().into_owned(),
        action: action_static.into(),
        marker,
        marker_style: style_str,
        dedup_removed,
        dedup_removed_lines,
    })
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

/// Compute the post-merge file content, the action label, and (for
/// `line-prefix` only) the list of adopter-area lines removed by the
/// cross-boundary dedup pass. Returns `(Some(content), action,
/// removed)` when the file should be written, or `(None, "unchanged",
/// removed)` when no write is needed. The `removed` vector is always
/// empty for the `html-comment` path (whose managed region is prose,
/// not a list).
fn compute_merge(
    existing: Option<&str>,
    style: MarkerStyle,
    marker: &str,
    block: &str,
    path: &Path,
) -> Result<(Option<String>, &'static str, Vec<String>)> {
    match existing {
        None => Ok((
            Some(format_fresh(style, marker, block)),
            "created",
            Vec::new(),
        )),
        Some(text) => match style {
            MarkerStyle::HtmlComment => {
                let (content, action) = merge_html_comment(text, marker, block, path)?;
                Ok((content, action, Vec::new()))
            }
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
            // Normalize CRLF before the unchanged-compare: on a CRLF checkout
            // `inner` retains a trailing `\r` per line, so a byte-for-byte
            // compare against the LF-normalized `block` would never match and
            // the region would be rewritten (with LF markers) on every run —
            // mtime thrash and CRLF→LF corruption. The `line-prefix` path was
            // hardened for CRLF the same way; this mirrors it.
            if inner.replace('\r', "") == block.replace('\r', "") {
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

/// Locate a `# {marker}` line in `text` and the body that follows it.
/// Returns `Some((line_start, body_end, body))` where `line_start` is
/// the byte offset of the `#` character, `body_end` is the byte offset
/// immediately past the on-disk canonical block's last line (including
/// its terminating newline), and `body` is the body trimmed of
/// leading/trailing newlines.
///
/// The body extent is determined by walking up to
/// `expected_block.lines().count()` lines from the position past the
/// marker line, using `expected_block` as a *structural* template:
/// expected blank lines (interior subsection separators) are matched
/// against on-disk blanks, and expected non-blank lines may match any
/// non-blank on-disk content. The walk terminates early when the
/// structure mismatches — specifically, when the expected line is
/// non-blank but the on-disk line is blank, signalling the end-of-block
/// blank-line terminator the previous run wrote. The content of each
/// on-disk line is not required to match `expected_block`; the caller
/// performs the byte-equality check against the returned `body`.
///
/// This replaces an earlier "next blank line is the terminator"
/// heuristic that mis-truncated multi-subsection canonicals (those with
/// interior blank lines between subsections), causing repeated runs to
/// leave orphan subsection headers below the managed region. See
/// `scenarios/merge-managed-block-multi-subsection-end.md`.
fn find_line_prefix_block<'a>(
    text: &'a str,
    marker: &str,
    expected_block: &str,
) -> Option<(usize, usize, &'a str)> {
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
            let body_end = walk_body_extent(text, body_start, expected_block);
            let body = text[body_start..body_end].trim_matches('\n');
            return Some((line_start, body_end, body));
        }
        offset += line_end + usize::from(line_end < rest.len());
    }
    None
}

/// One blank-line-delimited subsection of a managed block, reduced to the
/// parts that carry identity across framework releases.
#[derive(Debug, Default)]
struct Group<'a> {
    /// The subsection header — the group's first line, when that line is a
    /// comment (`# …`). Identity fallback for comment-only groups, which
    /// have no pattern lines to match on.
    heading: Option<&'a str>,
    /// Non-blank, non-comment lines — the stable identity. Comment wording
    /// and blank positions drift between framework releases (tweaks,
    /// reflow); pattern lines don't.
    patterns: Vec<&'a str>,
}

/// Whether an on-disk group and a canonical group refer to the same
/// subsection: they share a pattern line, or — when both are comment-only
/// and therefore have no pattern identity — their headers are identical.
fn groups_match(on_disk: &Group<'_>, canonical: &Group<'_>) -> bool {
    canonical
        .patterns
        .iter()
        .any(|p| on_disk.patterns.contains(p))
        || (on_disk.patterns.is_empty()
            && canonical.patterns.is_empty()
            && canonical.heading.is_some()
            && on_disk.heading == canonical.heading)
}

/// Split a managed block into its blank-line-delimited subsection [`Group`]s.
fn block_groups(block: &str) -> Vec<Group<'_>> {
    let mut groups: Vec<Group<'_>> = Vec::new();
    let mut current = Group::default();
    let mut started = false;
    for line in block.lines() {
        let l = line.trim_end_matches('\r');
        if l.is_empty() {
            if started {
                groups.push(std::mem::take(&mut current));
                started = false;
            }
        } else {
            if !started {
                started = true;
                if l.starts_with('#') {
                    current.heading = Some(l);
                }
            }
            if !l.starts_with('#') {
                current.patterns.push(l);
            }
        }
    }
    if started {
        groups.push(current);
    }
    groups
}

/// Read one blank-line-delimited group from `text` starting at `start`.
/// Returns the [`Group`], the byte offset just past the group's last line,
/// and the byte offset of the next group's first line. The two offsets
/// differ by the blank-line separator that terminates the group; at EOF they
/// coincide.
fn read_group(text: &str, start: usize) -> (Group<'_>, usize, usize) {
    let mut group = Group::default();
    let mut started = false;
    let mut scan = start;
    let mut content_end = start;
    while scan < text.len() {
        let rest = &text[scan..];
        let line_end = rest.find('\n').map_or(rest.len(), |i| i);
        let line = rest[..line_end].trim_end_matches('\r');
        let next = scan + line_end + usize::from(line_end < rest.len());
        if line.is_empty() {
            // Blank separator — not part of the group. Report the offset past
            // it as the next group's start.
            return (group, content_end, next);
        }
        if !started {
            started = true;
            if line.starts_with('#') {
                group.heading = Some(line);
            }
        }
        if !line.starts_with('#') {
            group.patterns.push(line);
        }
        scan = next;
        content_end = next;
    }
    (group, content_end, scan)
}

/// Find the byte offset where the on-disk managed block ends, given the new
/// canonical `expected_block`. The on-disk block is the *previous* canonical,
/// which may differ from the new one structurally — a subsection inserted or
/// removed, or a subsection's contents rewritten. A blank-line-only structural
/// walk can't tell an interior subsection separator from the end-of-block
/// terminator once the structures diverge, so this aligns on-disk groups
/// against canonical groups with a two-pointer walk:
///
/// - **matches the current canonical group** ([`groups_match`]) → structure-
///   preserving edit (e.g. a comment-wording tweak); consume it.
/// - **matches a *later* canonical group** → the canonical inserted one or
///   more groups not on disk (e.g. a new agent's subsection); skip past
///   them, then consume this group against the group it matches.
/// - **matches no remaining canonical group, and nothing has been consumed
///   yet** → a full-content rewrite of the block (the single-group
///   replacement path). The first group after the marker is always part of
///   the old managed block; consume it.
/// - **matches no remaining canonical group, after at least one group was
///   consumed** → the block has ended: the remaining canonical groups are
///   trailing insertions (subsections appended by a framework release) and
///   this group is the first adopter group past the block. Stop without
///   consuming it. Consuming here — the old behavior — swallowed the
///   adopter's first tail section as a "full rewrite" of the appended
///   canonical group, deleting adopter content. See
///   `scenarios/merge-managed-block-trailing-append.md`. (A framework group
///   removed or fully renamed mid-block also stops the walk now; its old
///   content survives below the merged block instead of being replaced —
///   preservation over consumption when identity is ambiguous.)
///
/// The block also ends at the first on-disk group reached after the
/// canonical's groups are exhausted — adopter territory, preserved. Returns
/// the byte offset immediately past the last in-block group's final line (or
/// `body_start` if the block is empty).
fn walk_body_extent(text: &str, body_start: usize, expected_block: &str) -> usize {
    let canon = block_groups(expected_block);
    let mut ci = 0;
    let mut offset = body_start;
    let mut block_end = body_start;
    let mut consumed_any = false;
    while offset < text.len() {
        // Canonical groups exhausted: everything from here is adopter content.
        if ci >= canon.len() {
            break;
        }
        let (group, content_end, next_start) = read_group(text, offset);
        if groups_match(&group, &canon[ci]) {
            ci += 1;
        } else if let Some(rel) = canon[ci + 1..].iter().position(|g| groups_match(&group, g)) {
            // Skip the inserted canonical group(s); re-match this on-disk group
            // against the later canonical group it matches without advancing
            // past it on disk.
            ci += rel + 1;
            continue;
        } else if consumed_any {
            // Unmatched after alignment began: trailing canonical groups are
            // pure insertions; this group is adopter territory.
            break;
        } else {
            // Full rewrite of the block, starting at its first group.
            ci += 1;
        }
        consumed_any = true;
        block_end = content_end;
        offset = next_start;
    }
    block_end
}

fn merge_line_prefix(
    text: &str,
    marker: &str,
    block: &str,
) -> (Option<String>, &'static str, Vec<String>) {
    let header = format!("# {marker}");
    // Length of `# {marker}\n{block}\n` — the exact byte span the
    // primitive writes for the managed region. Used to compute the
    // post-merge block bounds passed to the dedup phase below, so the
    // dedup pass doesn't re-derive bounds via `find_line_prefix_block`
    // (which stops at the first interior blank line and would leave
    // canonical content past the first subsection unprotected).
    let managed_region_len = header.len() + 1 + block.len() + 1;

    // Phase 1: install or update the managed block.
    let (post_merge, merge_action, block_start, block_end) =
        match find_line_prefix_block(text, marker, block) {
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
                let block_start = text.len() + separator.len();
                let block_end = block_start + managed_region_len;
                (
                    format!("{text}{separator}{header}\n{block}\n"),
                    "inserted",
                    block_start,
                    block_end,
                )
            }
            Some((line_start, body_end, body)) => {
                if body == block {
                    let block_end = body_end;
                    (text.to_string(), "unchanged", line_start, block_end)
                } else {
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
                    let block_start = before.len();
                    let block_end = block_start + managed_region_len;
                    (
                        format!("{before}{header}\n{block}\n{after_normalized}"),
                        "updated",
                        block_start,
                        block_end,
                    )
                }
            }
        };

    // Phase 2: cross-boundary dedup. Canonical-block wins — adopter-area
    // lines that string-equal a non-blank, non-comment line inside the
    // managed block are removed. Comments and blank lines in adopter
    // territory are preserved untouched.
    let canonical_lines: Vec<&str> = block
        .lines()
        .map(|l| l.trim_end_matches('\r'))
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    let (final_content, removed) =
        dedup_outside_block(&post_merge, block_start, block_end, &canonical_lines);

    // Decide what to return. If the net effect (merge + dedup) leaves
    // the file unchanged byte-for-byte, no write happens; otherwise
    // promote `unchanged` → `updated` when dedup removed something.
    if final_content == text {
        return (None, "unchanged", removed);
    }
    let action = if merge_action == "unchanged" && !removed.is_empty() {
        "updated"
    } else {
        merge_action
    };
    (Some(final_content), action, removed)
}

/// Walk `content` line by line and remove adopter-area lines that
/// string-equal a non-blank, non-comment line inside the managed
/// block. `block_start..block_end` is the byte range of the managed
/// region as computed by the merge phase (not re-derived here) —
/// callers pass the exact span they just wrote so canonical content
/// with interior blank lines is fully protected from dedup. Blank
/// lines and comment lines (`# foo`) outside the block are also
/// preserved untouched even when they happen to match a canonical
/// line. Returns the post-dedup content plus the verbatim list of
/// removed adopter-area lines in source order.
fn dedup_outside_block(
    content: &str,
    block_start: usize,
    block_end: usize,
    canonical_lines: &[&str],
) -> (String, Vec<String>) {
    let mut out = String::with_capacity(content.len());
    let mut removed: Vec<String> = Vec::new();
    let mut offset = 0;
    while offset < content.len() {
        let rest = &content[offset..];
        let line_end = rest.find('\n').map_or(rest.len(), |i| i);
        let raw_line = &rest[..line_end];
        let line = raw_line.trim_end_matches('\r');
        let has_newline = line_end < rest.len();
        let advance = line_end + usize::from(has_newline);

        let in_block = offset >= block_start && offset < block_end;
        let is_blank = line.is_empty();
        let is_comment = line.starts_with('#');
        let should_remove =
            !in_block && !is_blank && !is_comment && canonical_lines.contains(&line);

        if should_remove {
            removed.push(line.to_string());
        } else {
            out.push_str(raw_line);
            if has_newline {
                out.push('\n');
            }
        }
        offset += advance;
    }

    (out, removed)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;

    /// Build args from the test file's absolute path. `path` must be a
    /// direct child of the tempdir passed as `repo`; the args carry only
    /// the repo-relative basename (the primitive rejects absolute paths).
    fn args(path: &Path, block: &str, style: Option<&str>) -> MergeManagedBlockArgs {
        MergeManagedBlockArgs {
            path: path
                .file_name()
                .expect("test path has a basename")
                .to_string_lossy()
                .into_owned(),
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
                path: ".gitignore".into(),
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
    fn absolute_path_is_rejected_before_any_write() {
        let tmp = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let target = outside.path().join("victim.gitignore");
        let err = run(
            &MergeManagedBlockArgs {
                path: target.to_string_lossy().into_owned(),
                block: ".claude/".into(),
                marker: None,
                marker_style: Some("line-prefix".into()),
            },
            tmp.path(),
        )
        .unwrap_err();
        assert!(
            matches!(err, PrimitiveError::InvalidPath { .. }),
            "expected InvalidPath, got {err:?}"
        );
        assert!(!target.exists(), "nothing may be written outside the repo");
    }

    #[test]
    fn traversal_path_is_rejected_before_any_write() {
        let tmp = tempfile::tempdir().unwrap();
        let err = run(
            &MergeManagedBlockArgs {
                path: "../escape.gitignore".into(),
                block: ".claude/".into(),
                marker: None,
                marker_style: Some("line-prefix".into()),
            },
            tmp.path(),
        )
        .unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidPath { .. }));
    }

    #[test]
    fn nested_relative_path_still_works() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("sub")).unwrap();
        let result = run(
            &MergeManagedBlockArgs {
                path: "sub/.gitignore".into(),
                block: ".cache/".into(),
                marker: None,
                marker_style: Some("line-prefix".into()),
            },
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.action, "created");
        let body = fs::read_to_string(tmp.path().join("sub/.gitignore")).unwrap();
        assert_eq!(body, "# govern-managed\n.cache/\n");
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

    // -- cross-boundary dedup (line-prefix only) ----------------------------

    #[test]
    fn line_prefix_removes_duplicate_line_above_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        fs::write(
            &path,
            "node_modules/\n.claude/\nother/\n\n# govern-managed\n.claude/\nspecs/.cache/\n",
        )
        .unwrap();
        let block = ".claude/\nspecs/.cache/";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        // body matches canonical → merge says unchanged, but dedup
        // removes adopter-area `.claude/` → action upgrades to updated.
        assert_eq!(result.action, "updated");
        assert_eq!(result.dedup_removed, Some(1));
        assert_eq!(
            result.dedup_removed_lines.as_deref(),
            Some(&[".claude/".to_string()][..])
        );

        let body = fs::read_to_string(&path).unwrap();
        assert_eq!(
            body,
            "node_modules/\nother/\n\n# govern-managed\n.claude/\nspecs/.cache/\n"
        );
    }

    #[test]
    fn line_prefix_removes_duplicate_line_below_marker() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        fs::write(
            &path,
            "node_modules/\n\n# govern-managed\n.claude/\n\nother/\n.claude/\n",
        )
        .unwrap();
        let block = ".claude/";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "updated");
        assert_eq!(result.dedup_removed, Some(1));

        let body = fs::read_to_string(&path).unwrap();
        assert_eq!(
            body,
            "node_modules/\n\n# govern-managed\n.claude/\n\nother/\n"
        );
    }

    #[test]
    fn line_prefix_removes_all_adopter_area_duplicates() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        fs::write(
            &path,
            ".claude/\nfoo/\n.claude/\n\n# govern-managed\n.claude/\n\n.claude/\nbar/\n",
        )
        .unwrap();
        let block = ".claude/";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "updated");
        assert_eq!(result.dedup_removed, Some(3));
        assert_eq!(
            result.dedup_removed_lines.as_deref(),
            Some(
                &[
                    ".claude/".to_string(),
                    ".claude/".to_string(),
                    ".claude/".to_string()
                ][..]
            )
        );

        let body = fs::read_to_string(&path).unwrap();
        assert_eq!(body, "foo/\n\n# govern-managed\n.claude/\n\nbar/\n");
    }

    #[test]
    fn line_prefix_preserves_adopter_comments_even_when_matching_canonical() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        // An adopter comment line `# .claude/` happens to share text
        // with a canonical body line `.claude/` — but it starts with
        // `#`, so dedup leaves it alone.
        fs::write(
            &path,
            "# .claude/ (a note)\nnode_modules/\n\n# govern-managed\n.claude/\n",
        )
        .unwrap();
        let block = ".claude/";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "unchanged");
        assert_eq!(result.dedup_removed, Some(0));

        let body = fs::read_to_string(&path).unwrap();
        assert!(body.starts_with("# .claude/ (a note)"));
    }

    #[test]
    fn line_prefix_preserves_adopter_blank_lines() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        fs::write(&path, "foo/\n\n\nbar/\n\n# govern-managed\n.claude/\n").unwrap();
        let block = ".claude/";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "unchanged");
        assert_eq!(result.dedup_removed, Some(0));
    }

    #[test]
    fn line_prefix_unchanged_when_no_duplicates_and_block_matches() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        let original = "node_modules/\nother/\n\n# govern-managed\n.claude/\nspecs/.cache/\n";
        fs::write(&path, original).unwrap();
        let mtime_before = fs::metadata(&path).unwrap().modified().unwrap();
        let block = ".claude/\nspecs/.cache/";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "unchanged");
        assert_eq!(result.dedup_removed, Some(0));
        // mtime preserved — no write happened.
        assert_eq!(
            fs::metadata(&path).unwrap().modified().unwrap(),
            mtime_before
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
    }

    #[test]
    fn line_prefix_string_equality_is_exact() {
        // `.claude/` and `.claude/*` are distinct under string-equality.
        // Both are preserved if both are present outside the marker.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        fs::write(&path, ".claude/*\nother/\n\n# govern-managed\n.claude/\n").unwrap();
        let block = ".claude/";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "unchanged");
        assert_eq!(result.dedup_removed, Some(0));
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains(".claude/*"));
    }

    #[test]
    fn line_prefix_dedup_happens_on_insert_path() {
        // Adopter has a pre-existing `.claude/` line; the marker doesn't
        // exist yet. Insert path appends the marker + body; dedup then
        // removes the adopter's duplicate.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        fs::write(&path, ".claude/\nnode_modules/\n").unwrap();
        let block = ".claude/\nspecs/.cache/";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "inserted");
        assert_eq!(result.dedup_removed, Some(1));

        let body = fs::read_to_string(&path).unwrap();
        assert_eq!(
            body,
            "node_modules/\n\n# govern-managed\n.claude/\nspecs/.cache/\n"
        );
    }

    #[test]
    fn html_comment_path_never_sets_dedup_fields() {
        // The dedup contract gates on `marker-style: "line-prefix"`.
        // For `html-comment` invocations, the result's dedup fields
        // are `None` (the JSON shape elides them entirely).
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("CLAUDE.md");
        let result = run(&args(&path, "framework section", None), tmp.path()).unwrap();
        assert_eq!(result.marker_style, "html-comment");
        assert_eq!(result.dedup_removed, None);
        assert_eq!(result.dedup_removed_lines, None);
    }

    #[test]
    fn line_prefix_preserves_multi_subsection_block_with_interior_blank_lines() {
        // Regression: the canonical block may contain blank-line-separated
        // subsections (the .gitignore template shipped by /govern is shaped
        // this way). The dedup pass must not treat lines past the first
        // interior blank line as adopter territory — they're still inside
        // the managed region and must be preserved.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        // Adopter file has duplicates of several canonical lines above
        // the marker.
        fs::write(&path, ".claude/*\nnode_modules/\n.vscode/\n.DS_Store\n").unwrap();
        let block = "\
# Environment and secrets
.env
.env.*

# Claude Code local settings (keep commands tracked for project-wide access)
.claude/*
!.claude/commands/

# IDE
.vscode/
.idea/

# OS
.DS_Store
Thumbs.db";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "inserted");
        // .claude/*, .vscode/, .DS_Store are adopter-area dupes of canonical
        // lines spread across multiple subsections; all three get removed.
        assert_eq!(result.dedup_removed, Some(3));

        let body = fs::read_to_string(&path).unwrap();
        // The full canonical block — every subsection — must survive
        // inside the managed region.
        assert!(body.contains("# Environment and secrets\n.env\n.env.*"));
        assert!(
            body.contains(".claude/*\n!.claude/commands/"),
            "subsection past first interior blank must survive: got {body:?}"
        );
        assert!(
            body.contains(".vscode/\n.idea/"),
            "later subsection must survive: got {body:?}"
        );
        assert!(
            body.contains(".DS_Store\nThumbs.db"),
            "final subsection must survive: got {body:?}"
        );
        // Adopter-area dupes removed; non-canonical adopter line preserved.
        assert!(body.contains("node_modules/"));
        assert!(
            !body.starts_with(".claude/*\n"),
            "adopter-area .claude/* dupe must be removed: got {body:?}"
        );
    }

    #[test]
    fn line_prefix_multi_subsection_rerun_is_unchanged_and_preserves_mtime() {
        // Regression: a multi-subsection canonical (the shipped `.gitignore`
        // template shape) re-applied against a file that already contains
        // the same canonical must reach `unchanged` — not rewrite the file
        // every run leaving orphan subsection headers in its wake. See
        // scenarios/merge-managed-block-multi-subsection-end.md.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        let canonical = "\
# Environment and secrets
.env
.env.*

# Claude Code local settings (keep commands tracked for project-wide access)
.claude/*
!.claude/commands/

# IDE
.vscode/
.idea/

# OS
.DS_Store
Thumbs.db";
        let on_disk = format!("node_modules/\n\n# govern-managed\n{canonical}\n");
        fs::write(&path, &on_disk).unwrap();
        let mtime_before = fs::metadata(&path).unwrap().modified().unwrap();

        let result = run(&args(&path, canonical, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "unchanged");
        assert_eq!(result.dedup_removed, Some(0));
        assert_eq!(
            fs::metadata(&path).unwrap().modified().unwrap(),
            mtime_before,
            "unchanged must not rewrite the multi-subsection canonical"
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), on_disk);
    }

    #[test]
    fn line_prefix_multi_subsection_update_replaces_cleanly_without_duplicated_tail() {
        // Regression: when the multi-subsection canonical's content changes
        // while preserving its structure (same line count, same blank-line
        // positions), the update path must replace exactly the on-disk
        // block — not leave the tail subsections duplicated below the new
        // block as orphan headers after the dedup pass strips matching
        // body lines.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        let old_canonical = "\
# Environment and secrets
.env
.env.*

# Claude Code local settings (keep commands tracked for project-wide access)
.claude/*
!.claude/commands/

# IDE
.vscode/
.idea/

# OS
.DS_Store
Thumbs.db";
        // Same structure (4 subsections, identical blank positions) but
        // one comment-line wording tweaked. This is the realistic update
        // path: framework template wording evolves between releases.
        let new_canonical = "\
# Environment and secrets
.env
.env.*

# Claude Code local settings — commands stay tracked for project-wide access
.claude/*
!.claude/commands/

# IDE
.vscode/
.idea/

# OS
.DS_Store
Thumbs.db";
        let on_disk = format!("node_modules/\n\n# govern-managed\n{old_canonical}\n\nuser-tail/\n");
        fs::write(&path, &on_disk).unwrap();

        let result = run(&args(&path, new_canonical, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "updated");

        let body = fs::read_to_string(&path).unwrap();
        let expected =
            format!("node_modules/\n\n# govern-managed\n{new_canonical}\n\nuser-tail/\n");
        assert_eq!(
            body, expected,
            "multi-subsection update must replace cleanly with no orphan tail"
        );
        // Sanity-check: each subsection header appears exactly once (the
        // duplicated-tail symptom would show two copies of `# IDE` / `# OS`).
        assert_eq!(
            body.matches("# IDE\n").count(),
            1,
            "subsection header must appear exactly once: {body:?}"
        );
        assert_eq!(
            body.matches("# OS\n").count(),
            1,
            "subsection header must appear exactly once: {body:?}"
        );
    }

    #[test]
    fn line_prefix_multi_subsection_inserts_new_subsection_without_orphan_tail() {
        // Regression: a structure-CHANGING canonical edit — a new subsection
        // inserted in the middle (the realistic case: adding an agent's
        // gitignore block, e.g. Auggie's `.augment/*` between Claude and
        // Antigravity). The new canonical is longer than the on-disk block, so
        // the blank-line structural walk drifts and mis-bounds the old block,
        // leaving its tail subsections duplicated below the new block as orphan
        // comment headers after dedup strips their bodies. The merge must
        // instead bound the old block by group alignment and replace it
        // cleanly. Adopter content after the block is preserved.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        let old_canonical = "\
# Environment and secrets
.env

# Claude Code local settings (keep commands tracked for project-wide access)
.claude/*
!.claude/commands/

# Antigravity local settings (keep skills tracked for project-wide access)
.agents/*
!.agents/skills/

# IDE
.vscode/

# OS
.DS_Store
Thumbs.db";
        let new_canonical = "\
# Environment and secrets
.env

# Claude Code local settings (keep commands tracked for project-wide access)
.claude/*
!.claude/commands/

# Auggie local settings (keep commands tracked for project-wide access)
.augment/*
!.augment/commands/

# Antigravity local settings (keep skills tracked for project-wide access)
.agents/*
!.agents/skills/

# IDE
.vscode/

# OS
.DS_Store
Thumbs.db";
        let on_disk = format!("# govern-managed\n{old_canonical}\n\n# Rust\n/target\n");
        fs::write(&path, &on_disk).unwrap();

        let result = run(&args(&path, new_canonical, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "updated");

        let body = fs::read_to_string(&path).unwrap();
        let expected = format!("# govern-managed\n{new_canonical}\n\n# Rust\n/target\n");
        assert_eq!(
            body, expected,
            "inserted subsection must replace cleanly with no orphan tail"
        );
        // The Auggie subsection is present exactly once, and no subsection
        // header is duplicated (the orphan-tail symptom).
        assert_eq!(body.matches(".augment/*").count(), 1);
        for header in ["# IDE\n", "# OS\n", "# Antigravity"] {
            assert_eq!(
                body.matches(header).count(),
                1,
                "header must appear exactly once: {header:?} in {body:?}"
            );
        }
        // Adopter language section after the block survives verbatim.
        assert!(body.contains("# Rust\n/target\n"));

        // Idempotent: a second run with the same new canonical is a no-op.
        let rerun = run(&args(&path, new_canonical, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(rerun.action, "unchanged");
        assert_eq!(fs::read_to_string(&path).unwrap(), expected);
    }

    #[test]
    fn line_prefix_trailing_append_preserves_adopter_tail() {
        // Regression: the new canonical APPENDS a subsection at the end
        // (on-disk block {Env, IDE}, new canonical {Env, IDE, OS}) while
        // adopter content follows the managed block. The group-alignment
        // walk must treat the unmatched trailing canonical group as a pure
        // insertion — not consume the adopter's first tail section as a
        // "full rewrite" of it, which deleted `# Rust\n/target` outright.
        // See scenarios/merge-managed-block-trailing-append.md.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        let old_canonical = "\
# Environment and secrets
.env

# IDE
.vscode/";
        let new_canonical = "\
# Environment and secrets
.env

# IDE
.vscode/

# OS
.DS_Store
Thumbs.db";
        let on_disk = format!("# govern-managed\n{old_canonical}\n\n# Rust\n/target\n");
        fs::write(&path, &on_disk).unwrap();

        let result = run(&args(&path, new_canonical, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "updated");

        let body = fs::read_to_string(&path).unwrap();
        let expected = format!("# govern-managed\n{new_canonical}\n\n# Rust\n/target\n");
        assert_eq!(
            body, expected,
            "appended canonical group must insert; adopter tail must survive"
        );

        // Idempotent: a second run with the same canonical is a no-op.
        let rerun = run(&args(&path, new_canonical, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(rerun.action, "unchanged");
        assert_eq!(fs::read_to_string(&path).unwrap(), expected);
    }

    #[test]
    fn line_prefix_multiple_trailing_appends_preserve_adopter_tail() {
        // Edge case: several canonical groups appended at once are all
        // inserted at the end of the managed block; the adopter tail still
        // survives. See scenarios/merge-managed-block-trailing-append.md.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        let old_canonical = "\
# Environment and secrets
.env

# IDE
.vscode/";
        let new_canonical = "\
# Environment and secrets
.env

# IDE
.vscode/

# OS
.DS_Store
Thumbs.db

# Node
node_modules/";
        let on_disk =
            format!("# govern-managed\n{old_canonical}\n\n# Rust\n/target\n\n# Go\n/bin\n");
        fs::write(&path, &on_disk).unwrap();

        let result = run(&args(&path, new_canonical, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "updated");

        let body = fs::read_to_string(&path).unwrap();
        let expected =
            format!("# govern-managed\n{new_canonical}\n\n# Rust\n/target\n\n# Go\n/bin\n");
        assert_eq!(
            body, expected,
            "all appended groups must insert; every adopter tail section must survive"
        );

        let rerun = run(&args(&path, new_canonical, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(rerun.action, "unchanged");
        assert_eq!(fs::read_to_string(&path).unwrap(), expected);
    }

    #[test]
    fn line_prefix_trailing_append_with_colliding_adopter_heading_keeps_adopter_copy() {
        // Edge case: the adopter's tail section heading happens to equal the
        // newly appended canonical group's heading (`# OS`). It is still
        // adopter content: the canonical group lands inside the managed
        // block, and the adopter's outside copy is left untouched (its
        // patterns differ from the canonical's, so dedup leaves it alone —
        // resolving the duplication is the adopter's judgment, not the
        // merge's). See scenarios/merge-managed-block-trailing-append.md.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        let old_canonical = "\
# Environment and secrets
.env

# IDE
.vscode/";
        let new_canonical = "\
# Environment and secrets
.env

# IDE
.vscode/

# OS
.DS_Store
Thumbs.db";
        let on_disk = format!("# govern-managed\n{old_canonical}\n\n# OS\n*.swp\n");
        fs::write(&path, &on_disk).unwrap();

        let result = run(&args(&path, new_canonical, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "updated");

        let body = fs::read_to_string(&path).unwrap();
        let expected = format!("# govern-managed\n{new_canonical}\n\n# OS\n*.swp\n");
        assert_eq!(
            body, expected,
            "canonical group inserts inside the block; adopter's colliding-heading copy survives"
        );
        // Both `# OS` headings present: canonical inside, adopter's outside.
        assert_eq!(body.matches("# OS\n").count(), 2);

        let rerun = run(&args(&path, new_canonical, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(rerun.action, "unchanged");
        assert_eq!(fs::read_to_string(&path).unwrap(), expected);
    }

    #[test]
    fn line_prefix_comment_only_subsection_rerun_is_unchanged() {
        // A canonical subsection made only of comment lines has no pattern
        // identity; group alignment falls back to its header. A rerun with
        // the same canonical must still bound the block correctly and reach
        // `unchanged` — the comment-only group must not end the walk early
        // and spill the tail subsections as orphans.
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        let canonical = "\
# Environment and secrets
.env

# govern session state — per-user, ephemeral; managed by /project:target.
# (nothing tracked here yet)

# OS
.DS_Store
Thumbs.db";
        let on_disk = format!("# govern-managed\n{canonical}\n\n# Rust\n/target\n");
        fs::write(&path, &on_disk).unwrap();
        let mtime_before = fs::metadata(&path).unwrap().modified().unwrap();

        let result = run(&args(&path, canonical, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "unchanged");
        assert_eq!(
            fs::metadata(&path).unwrap().modified().unwrap(),
            mtime_before,
            "comment-only subsection must not break rerun idempotency"
        );
        assert_eq!(fs::read_to_string(&path).unwrap(), on_disk);
    }

    #[test]
    fn line_prefix_does_not_remove_canonical_inside_block_even_if_repeated() {
        // The dedup pass should never touch lines INSIDE the managed
        // block — the canonical-block wins rule means the canonical
        // copy stays, even when the same line appears again inside
        // (canonical block is itself the trusted region).
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".gitignore");
        fs::write(&path, "foo/\n\n# govern-managed\n.claude/\nspecs/.cache/\n").unwrap();
        let block = ".claude/\nspecs/.cache/";
        let result = run(&args(&path, block, Some("line-prefix")), tmp.path()).unwrap();
        assert_eq!(result.action, "unchanged");
        assert_eq!(result.dedup_removed, Some(0));
    }
}
