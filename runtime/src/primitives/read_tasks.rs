//! `read-tasks` — parse `tasks.md` into a structured task list.

use std::path::Path;

use crate::primitives::{PrimitiveError, Result, parse_atx_heading, read_text, rel_path};
use crate::schema::primitives::{ReadTasksArgs, ReadTasksResult, Subtask, Task};

const DONE_WHEN_PREFIX: &str = "**Done when**";

/// Execute the `read-tasks` primitive against the given repo root.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeatureNotFound`] when `specs/<feature>/` does
/// not exist, or [`PrimitiveError::Io`] when `tasks.md` cannot be read.
pub fn run(args: &ReadTasksArgs, repo: &Path) -> Result<ReadTasksResult> {
    let feature_dir = repo.join("specs").join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            feature: args.feature.clone(),
        });
    }
    let tasks_path = feature_dir.join("tasks.md");
    let content = read_text(&tasks_path)?;

    let mut tasks: Vec<Task> = Vec::new();
    let mut current: Option<Task> = None;

    for line in content.lines() {
        if let Some((level, heading)) = parse_atx_heading(line) {
            if level == 2 {
                if let Some(task) = current.take() {
                    tasks.push(task);
                }
                if let Some((number, title)) = split_numbered_heading(&heading) {
                    current = Some(Task {
                        number,
                        heading: title,
                        subtasks: Vec::new(),
                        done_when: None,
                    });
                }
                continue;
            }
            if level == 1 {
                if let Some(task) = current.take() {
                    tasks.push(task);
                }
                current = None;
                continue;
            }
        }
        let Some(task) = current.as_mut() else {
            continue;
        };
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("- ") {
            if let Some(done) = parse_done_when(rest) {
                task.done_when = Some(done);
                continue;
            }
            if let Some((checked, text)) = parse_subtask(rest) {
                task.subtasks.push(Subtask { text, checked });
            }
        }
    }
    if let Some(task) = current {
        tasks.push(task);
    }

    Ok(ReadTasksResult {
        tasks,
        path: rel_path(&tasks_path, repo),
    })
}

fn split_numbered_heading(heading: &str) -> Option<(String, String)> {
    let mut chars = heading.char_indices();
    let mut end_num: Option<usize> = None;
    let mut have_digit = false;
    for (idx, ch) in chars.by_ref() {
        if ch.is_ascii_digit() {
            have_digit = true;
            continue;
        }
        end_num = Some(idx);
        break;
    }
    if !have_digit {
        return None;
    }
    let end_num = end_num.unwrap_or(heading.len());
    let (number, after) = heading.split_at(end_num);
    let after = after.strip_prefix('.').unwrap_or(after);
    Some((number.to_string(), after.trim_start().to_string()))
}

fn parse_subtask(rest: &str) -> Option<(bool, String)> {
    let bytes = rest.as_bytes();
    if bytes.first() != Some(&b'[') {
        return None;
    }
    if bytes.len() < 4 || bytes[2] != b']' {
        return None;
    }
    let checked = matches!(bytes[1], b'x' | b'X');
    let text = rest[3..].trim().to_string();
    Some((checked, text))
}

fn parse_done_when(rest: &str) -> Option<String> {
    let trimmed = rest.trim_start();
    if !trimmed.starts_with(DONE_WHEN_PREFIX) {
        return None;
    }
    let after_label = trimmed[DONE_WHEN_PREFIX.len()..]
        .trim_start_matches("**")
        .trim_start();
    let body = after_label.strip_prefix(':').unwrap_or(after_label);
    Some(body.trim().to_string())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::path::PathBuf;

    fn fixture_repo() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/primitives/sample-repo")
    }

    #[test]
    fn parses_fixture_tasks() {
        let repo = fixture_repo();
        let result = run(
            &ReadTasksArgs {
                feature: "001-basic".into(),
            },
            &repo,
        )
        .unwrap();
        assert_eq!(result.path, "specs/001-basic/tasks.md");
        assert_eq!(result.tasks.len(), 2);

        let first = &result.tasks[0];
        assert_eq!(first.number, "1");
        assert_eq!(first.heading, "First task heading");
        assert_eq!(first.subtasks.len(), 2);
        assert!(first.subtasks[0].checked);
        assert_eq!(first.subtasks[0].text, "Subtask one — completed.");
        assert!(!first.subtasks[1].checked);
        assert_eq!(first.done_when.as_deref(), Some("both subtasks check."));

        let second = &result.tasks[1];
        assert_eq!(second.number, "2");
        assert_eq!(second.subtasks.len(), 1);
        assert_eq!(second.done_when.as_deref(), Some("the subtask is checked."));
    }

    #[test]
    fn split_numbered_heading_extracts_number_and_title() {
        assert_eq!(
            split_numbered_heading("12. Implement the parser"),
            Some(("12".into(), "Implement the parser".into()))
        );
        assert_eq!(
            split_numbered_heading("3. Wire CLI"),
            Some(("3".into(), "Wire CLI".into()))
        );
        assert_eq!(split_numbered_heading("Not numbered"), None);
    }
}
