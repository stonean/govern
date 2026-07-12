//! `append-inbox` — append one bullet to `{specs-root}/inbox.md`.
//!
//! The single deterministic surface behind `/gov:log`, `/gov:implement`'s
//! auto-capture rule, and the bootstrap security audit's dedup-by-prefix
//! append (spec 022, scenario scaffolding-primitives) — each of which
//! previously hand-rolled the same atomic append.
//!
//! Creation: when `inbox.md` is missing, the file is created from the
//! project inbox template at `framework/templates/project/inbox.md` when
//! that file exists (the govern source repo, where project templates
//! live), else from a bare `# Inbox` heading. Adopter repos don't carry
//! `framework/templates/project/` — their `inbox.md` was scaffolded at
//! adoption — so the heading fallback is the common adopter-side create.
//!
//! Form: entries are written as checkboxes — `- [ ] {text}` — matching the
//! inbox template's documented forms (manual `/gov:log` entries, auto-captured
//! findings, and audit findings are all `- [ ]`) and the constitution's
//! §bug-handling ("tracked as a checkbox … resolved by being done, then
//! removed"). Dedup and removal strip the checkbox marker, so both forms
//! still compare by content.
//!
//! Dedup: with `dedup-prefix` supplied, an existing bullet whose text
//! starts with the prefix suppresses the append and the result reports
//! `deduped: true`. Bullet scanning is comment/fence-aware (the inbox
//! template's `<!-- Rules: … -->` guidance embeds `- ` lines that are not
//! items), and text is read after stripping the `- ` marker and an optional
//! checkbox (`[ ]` / `[x]`), so the prefix matches both the checkbox form
//! this primitive writes and any legacy plain `- {text}` bullet.

use std::path::Path;

use crate::primitives::{
    PrimitiveError, Result, SkipScanner, bullet_text, count_inbox_bullets, iter_bullets, rel_path,
    write_atomic,
};
use crate::schema::paths;
use crate::schema::primitives::{AppendInboxArgs, AppendInboxResult};

/// Fallback content for a freshly-created inbox when no project template
/// exists on disk (the adopter-side create).
const FALLBACK_HEADING: &str = "# Inbox\n\n";

/// Repo-relative path of the project inbox template (framework source
/// layout only; see module docs).
const PROJECT_TEMPLATE: &str = "framework/templates/project/inbox.md";

/// Execute the `append-inbox` primitive against the given repo root.
///
/// # Errors
///
/// Returns [`PrimitiveError::InvalidArgument`] when `text` is empty,
/// whitespace-only, or carries an embedded newline (structure injection
/// into `inbox.md`, matching `append-task`'s single-line rule), or when
/// `dedup-prefix` is supplied empty (it would match every bullet).
/// Filesystem failures surface as [`PrimitiveError::Io`].
pub fn run(args: &AppendInboxArgs, repo: &Path) -> Result<AppendInboxResult> {
    validate_text(&args.text)?;
    if let Some(prefix) = &args.dedup_prefix
        && prefix.is_empty()
    {
        return Err(PrimitiveError::InvalidArgument {
            primitive: "append-inbox".into(),
            argument: "dedup-prefix".into(),
            reason: "empty prefix would match every bullet; omit the argument to skip dedup".into(),
        });
    }

    let root = paths::Paths::load(repo).specs_root;
    let inbox_path = repo.join(&root).join("inbox.md");

    let (existing, created) = match std::fs::read_to_string(&inbox_path) {
        Ok(text) => (text, false),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => (creation_base(repo), true),
        Err(source) => {
            return Err(PrimitiveError::Io {
                path: inbox_path,
                source,
            });
        }
    };

    // Dedup applies only against a pre-existing file — a template base
    // has no real bullets to dedup against (its placeholder bullet must
    // not suppress the very first append).
    if !created
        && let Some(prefix) = &args.dedup_prefix
        && has_bullet_with_prefix(&existing, prefix)
    {
        return Ok(AppendInboxResult {
            path: rel_path(&inbox_path, repo),
            created: false,
            deduped: true,
            item_count: count_inbox_bullets(&existing),
        });
    }

    let new_content = append_bullet(&existing, args.text.trim());
    write_atomic(&inbox_path, &new_content)?;

    Ok(AppendInboxResult {
        path: rel_path(&inbox_path, repo),
        created,
        deduped: false,
        item_count: count_inbox_bullets(&new_content),
    })
}

/// Reject empty or multi-line bullet text. The bullet renders as a
/// one-line `- [ ] {text}` entry; an embedded newline would smuggle extra
/// markdown structure into `inbox.md` (same rule as `append-task`).
fn validate_text(text: &str) -> Result<()> {
    if text.trim().is_empty() {
        return Err(PrimitiveError::InvalidArgument {
            primitive: "append-inbox".into(),
            argument: "text".into(),
            reason: "text is empty".into(),
        });
    }
    if text.contains('\n') || text.contains('\r') {
        return Err(PrimitiveError::InvalidArgument {
            primitive: "append-inbox".into(),
            argument: "text".into(),
            reason: "embedded newlines would inject markdown structure into inbox.md; \
                     supply single-line text"
                .into(),
        });
    }
    Ok(())
}

/// Base content for a freshly-created inbox: the project template's
/// content when it exists on disk, else the bare heading.
fn creation_base(repo: &Path) -> String {
    std::fs::read_to_string(repo.join(PROJECT_TEMPLATE))
        .unwrap_or_else(|_| FALLBACK_HEADING.to_string())
}

/// `true` when any real (comment/fence-aware) bullet's text starts with
/// `prefix`.
fn has_bullet_with_prefix(content: &str, prefix: &str) -> bool {
    iter_bullets(content).any(|(_, text)| text.starts_with(prefix))
}

/// Append `- [ ] {text}` (the checkbox inbox form) to `content` at a
/// position the comment/fence-aware read side ([`iter_bullets`] /
/// [`count_inbox_bullets`]) will count.
///
/// The write side must agree with the read side about what counts as inbox
/// content: were the bullet appended blindly at EOF and `inbox.md` ended
/// inside an *unterminated* `<!--` comment or ` ``` ` fence, the new bullet
/// would land inside that skipped region — invisible to the reader and
/// undercounted. So a trailing unterminated region is split off first and the
/// bullet is inserted before it; the region is preserved after. A well-formed
/// inbox (every comment and fence closed) has no such region and appends at
/// the end exactly as before.
fn append_bullet(content: &str, text: &str) -> String {
    let bullet = format!("- [ ] {text}");
    let boundary = trailing_unterminated_offset(content);
    if boundary >= content.len() {
        return append_after(content, &bullet);
    }
    // `content[..boundary]` is the balanced prefix (it ends at the start of
    // the unterminated region's opener line); `content[boundary..]` is the
    // region that runs to EOF. Insert the bullet into the prefix, then
    // re-attach the region after a blank-line separator.
    let head = append_after(&content[..boundary], &bullet);
    format!("{head}\n{tail}", tail = &content[boundary..])
}

/// Append `bullet` after `content`: a single newline joins onto an existing
/// bullet run, a blank line separates the bullet from any non-list tail
/// (markdownlint's lists-surrounded-by-blanks rule). Output ends with exactly
/// one trailing newline. Operates on the balanced prefix only — comment/fence
/// awareness lives in [`append_bullet`].
fn append_after(content: &str, bullet: &str) -> String {
    let trimmed = content.trim_end_matches(['\n', '\r']);
    if trimmed.is_empty() {
        return format!("{bullet}\n");
    }
    let last_line = trimmed.lines().last().unwrap_or("");
    let sep = if bullet_text(last_line).is_some() {
        "\n"
    } else {
        "\n\n"
    };
    format!("{trimmed}{sep}{bullet}\n")
}

/// Byte offset at which a trailing *unterminated* HTML comment or fenced code
/// block begins — the region the comment/fence-aware read side would skip
/// through to EOF. Returns `content.len()` when the document is well-formed
/// (every comment and fence closed), so an append lands at the end.
fn trailing_unterminated_offset(content: &str) -> usize {
    let mut skip = SkipScanner::default();
    let mut region_start: Option<usize> = None;
    let mut offset = 0usize;
    for raw in content.split_inclusive('\n') {
        // Feed `skip` the newline-stripped line, matching how `iter_bullets`
        // drives the scanner via `content.lines()`.
        let line = raw.strip_suffix('\n').unwrap_or(raw);
        let line = line.strip_suffix('\r').unwrap_or(line);
        let was_in_region = skip.in_region();
        skip.skip(line);
        match (was_in_region, skip.in_region()) {
            (false, true) => region_start = Some(offset),
            (true, false) => region_start = None,
            _ => {}
        }
        offset += raw.len();
    }
    region_start.unwrap_or(content.len())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn args(text: &str, dedup_prefix: Option<&str>) -> AppendInboxArgs {
        AppendInboxArgs {
            text: text.into(),
            dedup_prefix: dedup_prefix.map(Into::into),
        }
    }

    fn read_inbox(repo: &Path) -> String {
        fs::read_to_string(repo.join("specs/inbox.md")).unwrap()
    }

    #[test]
    fn appends_bullet_to_existing_inbox() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("specs")).unwrap();
        fs::write(
            tmp.path().join("specs/inbox.md"),
            "# Inbox\n\n- [ ] first item\n",
        )
        .unwrap();
        let result = run(&args("second item", None), tmp.path()).unwrap();
        assert_eq!(result.path, "specs/inbox.md");
        assert!(!result.created);
        assert!(!result.deduped);
        assert_eq!(result.item_count, 2);
        assert_eq!(
            read_inbox(tmp.path()),
            "# Inbox\n\n- [ ] first item\n- [ ] second item\n"
        );
    }

    #[test]
    fn creates_missing_inbox_with_heading_fallback() {
        let tmp = tempdir().unwrap();
        let result = run(&args("first item", None), tmp.path()).unwrap();
        assert!(result.created);
        assert!(!result.deduped);
        assert_eq!(result.item_count, 1);
        assert_eq!(read_inbox(tmp.path()), "# Inbox\n\n- [ ] first item\n");
    }

    #[test]
    fn creates_missing_inbox_from_project_template_when_present() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("framework/templates/project")).unwrap();
        fs::write(
            tmp.path().join("framework/templates/project/inbox.md"),
            "# Inbox\n\nCapture queue prose.\n\n<!-- Rules -->\n",
        )
        .unwrap();
        let result = run(&args("first item", None), tmp.path()).unwrap();
        assert!(result.created);
        assert_eq!(
            read_inbox(tmp.path()),
            "# Inbox\n\nCapture queue prose.\n\n<!-- Rules -->\n\n- [ ] first item\n"
        );
    }

    #[test]
    fn item_count_ignores_bullets_inside_comment_regions() {
        // The inbox template embeds `- ` lines inside a multi-line
        // `<!-- Rules: … -->` comment; those are not real items. Only the
        // one real appended bullet is counted.
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("specs")).unwrap();
        fs::write(
            tmp.path().join("specs/inbox.md"),
            "# Inbox\n\n<!-- Rules:\n     - not an item\n     - also not an item\n-->\n",
        )
        .unwrap();
        let result = run(&args("real item", None), tmp.path()).unwrap();
        assert_eq!(result.item_count, 1, "comment bullets must not be counted");
    }

    #[test]
    fn appends_before_a_trailing_unterminated_comment() {
        // scenarios/append-inbox-comment-aware-write.md: an inbox that ends
        // inside an unclosed `<!--` comment must not swallow the new bullet.
        // It lands before the comment, so the comment/fence-aware read side
        // counts it (the write side agrees with the read side).
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("specs")).unwrap();
        fs::write(
            tmp.path().join("specs/inbox.md"),
            "# Inbox\n\n- [ ] first\n<!-- dangling note, never closed\nmore comment text\n",
        )
        .unwrap();
        let result = run(&args("second", None), tmp.path()).unwrap();
        assert_eq!(
            result.item_count, 2,
            "the appended bullet must be counted, not swallowed by the comment"
        );
        let content = read_inbox(tmp.path());
        let bullet_pos = content.find("- [ ] second").expect("bullet written");
        let comment_pos = content.find("<!-- dangling").expect("comment preserved");
        assert!(
            bullet_pos < comment_pos,
            "bullet must precede the unterminated comment:\n{content}"
        );
        assert_eq!(
            count_inbox_bullets(&content),
            2,
            "read side counts both bullets"
        );
    }

    #[test]
    fn appends_before_a_trailing_unterminated_fence() {
        // The fence counterpart to the unterminated-comment case: a bullet
        // appended to an inbox ending in an unclosed ``` fence lands before
        // the fence, where the read side counts it.
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("specs")).unwrap();
        fs::write(
            tmp.path().join("specs/inbox.md"),
            "# Inbox\n\n- [ ] first\n```\n- [ ] fenced, not an item\n",
        )
        .unwrap();
        let result = run(&args("second", None), tmp.path()).unwrap();
        assert_eq!(result.item_count, 2);
        let content = read_inbox(tmp.path());
        assert!(
            content.find("- [ ] second").unwrap() < content.find("```").unwrap(),
            "bullet must precede the unterminated fence:\n{content}"
        );
    }

    #[test]
    fn blank_line_separates_bullet_from_non_list_tail() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("specs")).unwrap();
        fs::write(tmp.path().join("specs/inbox.md"), "# Inbox\n").unwrap();
        run(&args("item", None), tmp.path()).unwrap();
        assert_eq!(read_inbox(tmp.path()), "# Inbox\n\n- [ ] item\n");
    }

    #[test]
    fn dedup_prefix_match_suppresses_write_and_reports_deduped() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("specs")).unwrap();
        let before = "# Inbox\n\n- [ ] SEC-BE-014: spec.md does not address — token logging\n";
        fs::write(tmp.path().join("specs/inbox.md"), before).unwrap();
        let result = run(
            &args(
                "SEC-BE-014: spec.md does not address — token logging (rerun)",
                Some("SEC-BE-014:"),
            ),
            tmp.path(),
        )
        .unwrap();
        assert!(result.deduped);
        assert!(!result.created);
        assert_eq!(read_inbox(tmp.path()), before, "no write on dedup");
    }

    #[test]
    fn dedup_prefix_matches_plain_bullet_form_too() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("specs")).unwrap();
        fs::write(
            tmp.path().join("specs/inbox.md"),
            "# Inbox\n\n- perf: slow scan on startup\n",
        )
        .unwrap();
        let result = run(&args("perf: slow scan again", Some("perf:")), tmp.path()).unwrap();
        assert!(result.deduped);
    }

    #[test]
    fn dedup_prefix_without_match_appends_normally() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("specs")).unwrap();
        fs::write(
            tmp.path().join("specs/inbox.md"),
            "# Inbox\n\n- [ ] other item\n",
        )
        .unwrap();
        let result = run(&args("new item", Some("new item")), tmp.path()).unwrap();
        assert!(!result.deduped);
        assert!(read_inbox(tmp.path()).contains("- [ ] new item\n"));
    }

    #[test]
    fn dedup_uses_shared_checkbox_grammar_for_no_space_variant() {
        // `- [x]no-space` is NOT a valid checkbox in the shared grammar, so
        // its bullet text is the literal `[x]…` — a dedup prefix targeting
        // the checkbox *content* must not match it. This closes the
        // divergence where the old hand-rolled strip accepted the malformed
        // form while the read/mark side rejected it.
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("specs")).unwrap();
        fs::write(
            tmp.path().join("specs/inbox.md"),
            "# Inbox\n\n- [x]SEC-1: malformed checkbox item\n",
        )
        .unwrap();
        let result = run(&args("SEC-1: real capture", Some("SEC-1:")), tmp.path()).unwrap();
        assert!(
            !result.deduped,
            "malformed checkbox must not dedup by checkbox-content prefix"
        );
    }

    #[test]
    fn dedup_ignores_non_bullet_lines() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("specs")).unwrap();
        // The prefix text appears in prose, not in a bullet — no dedup.
        fs::write(
            tmp.path().join("specs/inbox.md"),
            "# Inbox\n\nsecurity: mentioned in prose only.\n",
        )
        .unwrap();
        let result = run(
            &args("security: real capture", Some("security:")),
            tmp.path(),
        )
        .unwrap();
        assert!(!result.deduped);
    }

    #[test]
    fn dedup_does_not_fire_on_template_placeholder_at_creation() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("framework/templates/project")).unwrap();
        fs::write(
            tmp.path().join("framework/templates/project/inbox.md"),
            "# Inbox\n\n- [ ] {Brief description of the issue}\n",
        )
        .unwrap();
        // Prefix would match the template's placeholder bullet; creation
        // must still append.
        let result = run(&args("{Brief item}", Some("{Brief")), tmp.path()).unwrap();
        assert!(result.created);
        assert!(!result.deduped);
        assert!(read_inbox(tmp.path()).contains("- [ ] {Brief item}\n"));
    }

    #[test]
    fn rejects_empty_and_multiline_text() {
        let tmp = tempdir().unwrap();
        for bad in ["", "   ", "line one\nline two", "cr\rline"] {
            let err = run(&args(bad, None), tmp.path()).unwrap_err();
            assert!(
                matches!(err, PrimitiveError::InvalidArgument { .. }),
                "expected InvalidArgument for {bad:?}"
            );
        }
        assert!(!tmp.path().join("specs/inbox.md").exists());
    }

    #[test]
    fn rejects_empty_dedup_prefix() {
        let tmp = tempdir().unwrap();
        let err = run(&args("item", Some("")), tmp.path()).unwrap_err();
        match err {
            PrimitiveError::InvalidArgument { argument, .. } => {
                assert_eq!(argument, "dedup-prefix");
            }
            other => panic!("expected InvalidArgument, got {other:?}"),
        }
    }

    #[test]
    fn honors_configured_specs_root() {
        let tmp = tempdir().unwrap();
        fs::write(
            tmp.path().join(".govern.toml"),
            "[paths]\nspecs-root = \"governance\"\n",
        )
        .unwrap();
        let result = run(&args("routed item", None), tmp.path()).unwrap();
        assert_eq!(result.path, "governance/inbox.md");
        assert!(tmp.path().join("governance/inbox.md").is_file());
        assert!(!tmp.path().join("specs").exists());
    }
}
