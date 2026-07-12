//! `remove-inbox-item` — remove one bullet from `{specs-root}/inbox.md`.
//!
//! The complement of `append-inbox` and the deterministic surface behind
//! `/gov:groom`'s per-item inbox removal (step 8), which previously edited
//! the file by hand. The first bullet whose text (via the shared
//! [`bullet_text`] grammar) equals the given `item`, trimmed, is removed;
//! a no-match — or a missing inbox file — is a clean domain outcome
//! (`removed: false`), never an operational error. The write is atomic.

use std::path::Path;

use crate::primitives::{PrimitiveError, Result, bullet_text, rel_path, write_atomic};
use crate::schema::paths;
use crate::schema::primitives::{RemoveInboxItemArgs, RemoveInboxItemResult};

/// Execute the `remove-inbox-item` primitive against the given repo root.
///
/// # Errors
///
/// Returns [`PrimitiveError::InvalidArgument`] when `item` is empty,
/// whitespace-only, or carries an embedded newline. Filesystem failures
/// other than a missing inbox surface as [`PrimitiveError::Io`].
pub fn run(args: &RemoveInboxItemArgs, repo: &Path) -> Result<RemoveInboxItemResult> {
    super::validate_single_line("remove-inbox-item", "item", &args.item)?;
    let target = args.item.trim();
    if target.is_empty() {
        return Err(PrimitiveError::InvalidArgument {
            primitive: "remove-inbox-item".into(),
            argument: "item".into(),
            reason: "item is empty".into(),
        });
    }

    let root = paths::Paths::load(repo).specs_root;
    let inbox_path = repo.join(&root).join("inbox.md");

    let existing = match std::fs::read_to_string(&inbox_path) {
        Ok(text) => text,
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            // Nothing to remove — a clean domain outcome, not an error.
            return Ok(RemoveInboxItemResult {
                path: rel_path(&inbox_path, repo),
                removed: false,
                remaining_count: 0,
            });
        }
        Err(source) => {
            return Err(PrimitiveError::Io {
                path: inbox_path,
                source,
            });
        }
    };

    match remove_bullet(&existing, target) {
        Some(new_content) => {
            let remaining = count_bullets(&new_content);
            write_atomic(&inbox_path, &new_content)?;
            Ok(RemoveInboxItemResult {
                path: rel_path(&inbox_path, repo),
                removed: true,
                remaining_count: remaining,
            })
        }
        None => Ok(RemoveInboxItemResult {
            path: rel_path(&inbox_path, repo),
            removed: false,
            remaining_count: count_bullets(&existing),
        }),
    }
}

/// Remove the first bullet line whose text equals `target`, returning the
/// rewritten content, or `None` when no bullet matches.
fn remove_bullet(content: &str, target: &str) -> Option<String> {
    let lines: Vec<&str> = content.lines().collect();
    let idx = lines
        .iter()
        .position(|line| bullet_text(line).as_deref() == Some(target))?;
    let mut kept: Vec<&str> = Vec::with_capacity(lines.len().saturating_sub(1));
    kept.extend_from_slice(&lines[..idx]);
    kept.extend_from_slice(&lines[idx + 1..]);
    Some(normalize(&kept))
}

/// Join the kept lines, collapse any run of two or more blank lines to a
/// single blank (removing a bullet between blank-separated items would
/// otherwise double the blank and trip markdownlint MD012), trim leading
/// and trailing blank lines, and end with exactly one newline.
fn normalize(lines: &[&str]) -> String {
    let mut out: Vec<&str> = Vec::with_capacity(lines.len());
    let mut prev_blank = false;
    for line in lines {
        let blank = line.trim().is_empty();
        if blank && prev_blank {
            continue;
        }
        out.push(line);
        prev_blank = blank;
    }
    while out.first().is_some_and(|l| l.trim().is_empty()) {
        out.remove(0);
    }
    while out.last().is_some_and(|l| l.trim().is_empty()) {
        out.pop();
    }
    format!("{}\n", out.join("\n"))
}

/// Count the bullet items in `content`.
fn count_bullets(content: &str) -> u32 {
    u32::try_from(content.lines().filter(|l| bullet_text(l).is_some()).count()).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn args(item: &str) -> RemoveInboxItemArgs {
        RemoveInboxItemArgs { item: item.into() }
    }

    fn write_inbox(repo: &Path, body: &str) {
        fs::create_dir_all(repo.join("specs")).unwrap();
        fs::write(repo.join("specs/inbox.md"), body).unwrap();
    }

    fn read_inbox(repo: &Path) -> String {
        fs::read_to_string(repo.join("specs/inbox.md")).unwrap()
    }

    #[test]
    fn removes_matching_bullet_and_reports_remaining() {
        let tmp = tempdir().unwrap();
        write_inbox(
            tmp.path(),
            "# Inbox\n\n- first item\n- second item\n- third item\n",
        );
        let result = run(&args("second item"), tmp.path()).unwrap();
        assert!(result.removed);
        assert_eq!(result.remaining_count, 2);
        assert_eq!(result.path, "specs/inbox.md");
        assert_eq!(
            read_inbox(tmp.path()),
            "# Inbox\n\n- first item\n- third item\n"
        );
    }

    #[test]
    fn removes_checkbox_bullet_by_content() {
        let tmp = tempdir().unwrap();
        write_inbox(tmp.path(), "# Inbox\n\n- [ ] a task item\n- plain item\n");
        let result = run(&args("a task item"), tmp.path()).unwrap();
        assert!(result.removed);
        assert_eq!(read_inbox(tmp.path()), "# Inbox\n\n- plain item\n");
    }

    #[test]
    fn collapses_double_blank_at_removal_seam() {
        let tmp = tempdir().unwrap();
        // The first bullet is followed by a blank line; removing it must not
        // leave a double blank after the heading.
        write_inbox(tmp.path(), "# Inbox\n\n- lonely item\n\n- kept item\n");
        let result = run(&args("lonely item"), tmp.path()).unwrap();
        assert!(result.removed);
        assert_eq!(read_inbox(tmp.path()), "# Inbox\n\n- kept item\n");
    }

    #[test]
    fn removing_last_bullet_leaves_heading() {
        let tmp = tempdir().unwrap();
        write_inbox(tmp.path(), "# Inbox\n\n- only item\n");
        let result = run(&args("only item"), tmp.path()).unwrap();
        assert!(result.removed);
        assert_eq!(result.remaining_count, 0);
        assert_eq!(read_inbox(tmp.path()), "# Inbox\n");
    }

    #[test]
    fn no_match_is_clean_outcome_without_write() {
        let tmp = tempdir().unwrap();
        let before = "# Inbox\n\n- present item\n";
        write_inbox(tmp.path(), before);
        let result = run(&args("absent item"), tmp.path()).unwrap();
        assert!(!result.removed);
        assert_eq!(result.remaining_count, 1);
        assert_eq!(read_inbox(tmp.path()), before, "no write on no-match");
    }

    #[test]
    fn missing_inbox_is_clean_outcome() {
        let tmp = tempdir().unwrap();
        let result = run(&args("anything"), tmp.path()).unwrap();
        assert!(!result.removed);
        assert_eq!(result.remaining_count, 0);
        assert!(!tmp.path().join("specs/inbox.md").exists());
    }

    #[test]
    fn removes_only_the_first_of_duplicate_bullets() {
        let tmp = tempdir().unwrap();
        write_inbox(tmp.path(), "# Inbox\n\n- dup\n- dup\n- other\n");
        let result = run(&args("dup"), tmp.path()).unwrap();
        assert!(result.removed);
        assert_eq!(result.remaining_count, 2);
        assert_eq!(read_inbox(tmp.path()), "# Inbox\n\n- dup\n- other\n");
    }

    #[test]
    fn rejects_empty_and_multiline_item() {
        let tmp = tempdir().unwrap();
        write_inbox(tmp.path(), "# Inbox\n\n- item\n");
        for bad in ["", "   ", "a\nb", "c\rd"] {
            let err = run(&args(bad), tmp.path()).unwrap_err();
            assert!(
                matches!(err, PrimitiveError::InvalidArgument { .. }),
                "expected InvalidArgument for {bad:?}"
            );
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
        fs::create_dir_all(tmp.path().join("governance")).unwrap();
        fs::write(
            tmp.path().join("governance/inbox.md"),
            "# Inbox\n\n- routed item\n",
        )
        .unwrap();
        let result = run(&args("routed item"), tmp.path()).unwrap();
        assert!(result.removed);
        assert_eq!(result.path, "governance/inbox.md");
    }
}
