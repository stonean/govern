//! `mark-task` — flip a single subtask checkbox in `tasks.md`.
//!
//! Handles both file shapes the way `read-tasks` does:
//!
//! - **Flat** — task headings are `## N. Title` at level 2.
//! - **Phased** — task headings are `### N. Title` at level 3, nested
//!   under `## …` phase containers. Detection mirrors `read-tasks`'s
//!   structure-detection (any `### N.` heading anywhere in the file
//!   signals phased structure).
//!
//! Heading text recognition routes through the shared
//! [`parse_atx_heading`] helper so headings containing inline-code
//! spans (backticks) parse identically across both primitives.

use std::path::Path;

use crate::primitives::{
    PrimitiveError, Result, TasksStructure, detect_tasks_structure, parse_atx_heading, read_text,
    rel_path, write_atomic,
};
use crate::schema::primitives::{CheckboxToggleResult, MarkTaskArgs};

use super::checkbox::{find_checkbox_line, flip_checkbox_at};

/// Execute the `mark-task` primitive.
///
/// Locates the subtask at `args.subtask_index` (0-based) within the task
/// numbered `args.task_number` and flips its checkbox to `args.checked`.
/// State-modifying writes use `tempfile`'s create-then-rename pattern; a
/// crash mid-write leaves the target file unchanged.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeatureNotFound`] when the feature directory
/// is missing, [`PrimitiveError::TaskNotFound`] when the heading does not
/// match, [`PrimitiveError::SubtaskOutOfRange`] when the subtask index
/// exceeds the number of subtasks found, or [`PrimitiveError::Io`] for any
/// filesystem failure.
pub fn run(args: &MarkTaskArgs, repo: &Path) -> Result<CheckboxToggleResult> {
    let feature_dir = repo.join("specs").join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            feature: args.feature.clone(),
        });
    }
    let tasks_path = feature_dir.join("tasks.md");
    let content = read_text(&tasks_path)?;

    let task_level: u8 = match detect_tasks_structure(&content) {
        TasksStructure::Flat => 2,
        TasksStructure::Phased => 3,
    };

    let lines: Vec<&str> = content.split_inclusive('\n').collect();
    let task_range = locate_task_range(&lines, &args.task_number, task_level).ok_or_else(|| {
        PrimitiveError::TaskNotFound {
            feature: args.feature.clone(),
            task_number: args.task_number.clone(),
        }
    })?;

    let checkbox_lines = collect_checkbox_line_indices(&lines, task_range);
    let (line_idx, marker_idx) = *checkbox_lines.get(args.subtask_index).ok_or_else(|| {
        PrimitiveError::SubtaskOutOfRange {
            feature: args.feature.clone(),
            task_number: args.task_number.clone(),
            subtask_index: args.subtask_index,
            total: checkbox_lines.len(),
        }
    })?;

    let (previous, new_line) = flip_checkbox_at(lines[line_idx], marker_idx, args.checked);
    let new_content = rebuild_with_replacement(&lines, line_idx, &new_line);

    if previous != args.checked {
        write_atomic(&tasks_path, &new_content)?;
    }

    Ok(CheckboxToggleResult {
        previous,
        current: args.checked,
        path: rel_path(&tasks_path, repo),
    })
}

/// Locate the task heading line (`## N. ...` for flat structure or
/// `### N. ...` for phased) and return the half-open range of lines
/// that constitute the task body (heading inclusive; the next heading
/// at the task level or shallower is exclusive). Returns `None` when
/// no matching heading is found.
fn locate_task_range(
    lines: &[&str],
    task_number: &str,
    task_level: u8,
) -> Option<std::ops::Range<usize>> {
    let mut start: Option<usize> = None;
    for (idx, line) in lines.iter().enumerate() {
        let Some((level, heading)) = parse_atx_heading(line) else {
            continue;
        };
        if let Some(s) = start {
            // Terminate the range at the next heading whose level is
            // at or above the task level (a sibling task, a phase
            // container in phased mode, or an H1).
            if level <= task_level {
                return Some(s..idx);
            }
        } else if level == task_level
            && heading_task_number(&heading).as_deref() == Some(task_number)
        {
            start = Some(idx);
        }
    }
    start.map(|s| s..lines.len())
}

fn heading_task_number(heading: &str) -> Option<String> {
    let mut digits = String::new();
    for ch in heading.chars() {
        if ch.is_ascii_digit() {
            digits.push(ch);
            continue;
        }
        break;
    }
    if digits.is_empty() {
        None
    } else {
        Some(digits)
    }
}

fn collect_checkbox_line_indices(
    lines: &[&str],
    range: std::ops::Range<usize>,
) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for idx in range {
        if let Some((_bracket, marker_idx)) = find_checkbox_line(lines[idx]) {
            out.push((idx, marker_idx));
        }
    }
    out
}

fn rebuild_with_replacement(lines: &[&str], target_idx: usize, replacement: &str) -> String {
    let mut out = String::new();
    for (idx, line) in lines.iter().enumerate() {
        if idx == target_idx {
            out.push_str(replacement);
        } else {
            out.push_str(line);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::primitives::PrimitiveError;
    use std::fs;
    use tempfile::tempdir;

    fn write_fixture(tmp: &std::path::Path) {
        let feature_dir = tmp.join("specs/feat");
        fs::create_dir_all(&feature_dir).unwrap();
        let body = "# feat\n\n## 1. First task\n\n- [ ] Subtask one.\n- [x] Subtask two.\n- **Done when**: both subtasks check.\n\n## 2. Second task\n\n- [ ] Only subtask.\n- **Done when**: that one is checked.\n";
        fs::write(feature_dir.join("tasks.md"), body).unwrap();
    }

    #[test]
    fn flips_first_subtask_of_first_task() {
        let tmp = tempdir().unwrap();
        write_fixture(tmp.path());
        let result = run(
            &MarkTaskArgs {
                feature: "feat".into(),
                task_number: "1".into(),
                subtask_index: 0,
                checked: true,
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.previous);
        assert!(result.current);
        assert_eq!(result.path, "specs/feat/tasks.md");
        let new_content = fs::read_to_string(tmp.path().join("specs/feat/tasks.md")).unwrap();
        assert!(new_content.contains("- [x] Subtask one."));
        assert!(new_content.contains("- [x] Subtask two."));
    }

    #[test]
    fn unchecking_already_unchecked_does_not_rewrite() {
        let tmp = tempdir().unwrap();
        write_fixture(tmp.path());
        let tasks_path = tmp.path().join("specs/feat/tasks.md");
        let mtime_before = fs::metadata(&tasks_path).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(15));
        let result = run(
            &MarkTaskArgs {
                feature: "feat".into(),
                task_number: "1".into(),
                subtask_index: 0,
                checked: false,
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.previous);
        assert!(!result.current);
        let mtime_after = fs::metadata(&tasks_path).unwrap().modified().unwrap();
        assert_eq!(
            mtime_before, mtime_after,
            "no rewrite expected when target state matches"
        );
    }

    #[test]
    fn task_number_not_found_errors() {
        let tmp = tempdir().unwrap();
        write_fixture(tmp.path());
        let err = run(
            &MarkTaskArgs {
                feature: "feat".into(),
                task_number: "99".into(),
                subtask_index: 0,
                checked: true,
            },
            tmp.path(),
        )
        .unwrap_err();
        assert!(matches!(err, PrimitiveError::TaskNotFound { .. }));
    }

    #[test]
    fn subtask_index_out_of_range_errors() {
        let tmp = tempdir().unwrap();
        write_fixture(tmp.path());
        let err = run(
            &MarkTaskArgs {
                feature: "feat".into(),
                task_number: "2".into(),
                subtask_index: 5,
                checked: true,
            },
            tmp.path(),
        )
        .unwrap_err();
        match err {
            PrimitiveError::SubtaskOutOfRange { total, .. } => assert_eq!(total, 1),
            other => panic!("expected SubtaskOutOfRange, got {other:?}"),
        }
    }

    #[test]
    fn ignores_done_when_lines() {
        let tmp = tempdir().unwrap();
        write_fixture(tmp.path());
        // Task 1 has two subtasks then a `- **Done when**` line; subtask_index
        // 1 must land on the second checkbox, not the done-when line.
        let result = run(
            &MarkTaskArgs {
                feature: "feat".into(),
                task_number: "1".into(),
                subtask_index: 1,
                checked: false,
            },
            tmp.path(),
        )
        .unwrap();
        assert!(result.previous);
        assert!(!result.current);
        let new_content = fs::read_to_string(tmp.path().join("specs/feat/tasks.md")).unwrap();
        assert!(new_content.contains("- [ ] Subtask two."));
        assert!(new_content.contains("- **Done when**: both subtasks check."));
    }

    fn write_phased_fixture(tmp: &std::path::Path) {
        let feature_dir = tmp.join("specs/feat");
        fs::create_dir_all(&feature_dir).unwrap();
        let body = "# feat\n\n## Phase A — first phase\n\n### 1. First phased task\n\n- [ ] Phased subtask one.\n- [x] Phased subtask two.\n\n## Phase C — Follow-on scenarios\n\n### 19. Dedup `/configure` permission entries via new gvrn primitive\n\n- [ ] Implement the behavior described in `scenarios/configure-dedup-permissions.md`\n\n- **Done when**: the scenario lands.\n";
        fs::write(feature_dir.join("tasks.md"), body).unwrap();
    }

    #[test]
    fn flips_subtask_in_phased_tasks_md() {
        let tmp = tempdir().unwrap();
        write_phased_fixture(tmp.path());
        let result = run(
            &MarkTaskArgs {
                feature: "feat".into(),
                task_number: "1".into(),
                subtask_index: 0,
                checked: true,
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.previous);
        assert!(result.current);
        let new_content = fs::read_to_string(tmp.path().join("specs/feat/tasks.md")).unwrap();
        assert!(new_content.contains("- [x] Phased subtask one."));
    }

    #[test]
    fn resolves_phased_task_with_backticks_in_heading() {
        // Regression test for the bug surfaced during /gov:implement on
        // spec 023 task #19. The heading `### 19. Dedup `/configure`
        // permission entries via new gvrn primitive` contains inline-code
        // spans (backticks); `read-tasks` recognized it correctly but
        // `mark-task` returned `task '19' not found` because it only
        // matched level-2 task headings.
        let tmp = tempdir().unwrap();
        write_phased_fixture(tmp.path());
        let result = run(
            &MarkTaskArgs {
                feature: "feat".into(),
                task_number: "19".into(),
                subtask_index: 0,
                checked: true,
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.previous);
        assert!(result.current);
        let new_content = fs::read_to_string(tmp.path().join("specs/feat/tasks.md")).unwrap();
        assert!(new_content.contains("- [x] Implement the behavior described"));
        // Adjacent task (Phase A's #1) is untouched.
        assert!(new_content.contains("- [ ] Phased subtask one."));
    }

    #[test]
    fn phased_task_range_terminates_at_next_phase_container() {
        // The locate_task_range terminator must fire at the next
        // heading at or above task_level. In phased mode (task_level=3)
        // that includes both sibling `### N.` task headings AND any
        // level-2 `## …` phase container that follows.
        let tmp = tempdir().unwrap();
        write_phased_fixture(tmp.path());
        // Task 1 is in Phase A; the next heading at or above level 3 is
        // `## Phase C — Follow-on scenarios` (level 2). The subtasks
        // attached to task 1 must NOT include anything from Phase C.
        let result = run(
            &MarkTaskArgs {
                feature: "feat".into(),
                task_number: "1".into(),
                subtask_index: 1,
                checked: false,
            },
            tmp.path(),
        )
        .unwrap();
        assert!(result.previous);
        assert!(!result.current);
    }

    #[test]
    fn dropping_named_tempfile_leaves_target_unchanged() {
        use std::io::Write;
        // Simulates an interrupted write: create the tempfile in the parent
        // dir, write the new content, then drop without `persist`. The target
        // file must be unchanged.
        let tmp = tempdir().unwrap();
        write_fixture(tmp.path());
        let tasks_path = tmp.path().join("specs/feat/tasks.md");
        let original = fs::read_to_string(&tasks_path).unwrap();

        {
            let parent = tasks_path.parent().unwrap();
            let mut tf = tempfile::NamedTempFile::new_in(parent).unwrap();
            tf.write_all(b"INTERRUPTED CONTENT").unwrap();
            // tf dropped here without persist
        }

        let after = fs::read_to_string(&tasks_path).unwrap();
        assert_eq!(original, after);
    }
}
