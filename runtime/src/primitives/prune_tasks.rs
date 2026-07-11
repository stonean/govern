//! `prune-tasks` — reduce a feature's `tasks.md`.
//!
//! Two modes, selected by `args.reset`:
//!
//! - **keep-pending** (default) — drop every *spent* task section (a section
//!   with ≥ 1 checkbox, all checked) and every phase container left with no
//!   surviving task section; preserve the preamble and every pending /
//!   no-checkbox section verbatim.
//! - **reset** (`--reset`) — rewrite the file to the template's initial state:
//!   the existing `# …` heading followed by [`CANONICAL_EMPTY_TASKS_BODY`].
//!   Gated on spec status: permitted only when the spec is `done`, unless
//!   `--force` is supplied.
//!
//! Parsing reuses the shared `tasks.md` machinery
//! (`detect_tasks_structure`, `parse_atx_heading`, `checkbox::find_checkbox_line`)
//! so `prune-tasks` recognizes exactly the task set `read-tasks` /
//! `mark-task` see. The result is a compact summary — it never carries the
//! file body; the reduced content is produced and written entirely inside the
//! runtime (`apply: true`) or withheld (`apply: false` preview).

use std::collections::HashSet;
use std::path::Path;

use crate::primitives::{
    PrimitiveError, Result, TasksStructure, checkbox, detect_tasks_structure, parse_atx_heading,
    read_text, rel_path, split_frontmatter, write_atomic,
};
use crate::schema::paths;
use crate::schema::primitives::{
    Classification, PruneAction, PruneGate, PruneMode, PruneSection, PruneTasksArgs,
    PruneTasksResult, SizeSummary,
};

/// The template `tasks.md` body with its `# …` H1 line removed — the reset
/// target that follows the preserved feature heading. A unit test
/// ([`tests::canonical_empty_body_matches_template`]) asserts this equals
/// `framework/templates/spec/tasks.md` minus its H1 so the two never drift.
const CANONICAL_EMPTY_TASKS_BODY: &str = "Tasks derived from the [plan](plan.md). Complete in order.\n\n<!-- Each task should be small enough to implement and verify independently.\n     Mark tasks as they are completed. Example:\n\n## 1. Create sessions table migration\n\n- [ ] Write SQL migration for `sessions` table\n- [ ] Run migration and verify schema\n\n## 2. Implement session store\n\n- [ ] Create `shared/auth/session.go` with Create, Get, Delete methods\n- [ ] Write store integration tests against real PostgreSQL\n\n## 3. Update README link to migration guide\n\n- [ ] Edit `README.md` to point at the new path\n\n-->\n";

/// Frontmatter shape used only to read `status` for the `--reset` gate.
#[derive(serde::Deserialize)]
struct StatusOnly {
    status: Option<String>,
}

/// Kind of a segmented block.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    /// Preamble, H1, or any non-task structural heading group. Always kept.
    Structure,
    /// A `## …` phase container (phased files only). Kept iff a task section
    /// within it survives.
    Phase,
    /// A numbered task section. Dropped when spent.
    Task,
}

/// One segmented block of the file.
struct Block {
    kind: Kind,
    lines: Vec<String>,
    number: String,
    heading: String,
    phase: Option<String>,
    checkbox_total: u32,
    checkbox_checked: u32,
    /// Index into `blocks` of the governing phase container (Task blocks in
    /// phased files); `None` in flat files.
    governing_phase: Option<usize>,
}

impl Block {
    fn new(kind: Kind, first_line: &str) -> Self {
        Self {
            kind,
            lines: vec![first_line.to_string()],
            number: String::new(),
            heading: String::new(),
            phase: None,
            checkbox_total: 0,
            checkbox_checked: 0,
            governing_phase: None,
        }
    }

    fn classification(&self) -> Classification {
        if self.checkbox_total == 0 {
            Classification::NoCheckbox
        } else if self.checkbox_checked == self.checkbox_total {
            Classification::Spent
        } else {
            Classification::Pending
        }
    }
}

/// Execute the `prune-tasks` primitive against the given repo root.
///
/// # Errors
///
/// - [`PrimitiveError::FeatureNotFound`] when the feature directory is absent.
/// - [`PrimitiveError::TasksFileMissing`] when the feature has no `tasks.md`.
/// - [`PrimitiveError::MalformedTasks`] when a `--reset` file has no `# …`
///   heading.
/// - [`PrimitiveError::MissingSpecFile`] / [`PrimitiveError::StatusFieldMissing`]
///   when a `--reset` cannot read the spec status.
/// - [`PrimitiveError::Io`] / [`PrimitiveError::Yaml`] on filesystem or
///   frontmatter failure.
pub fn run(args: &PruneTasksArgs, repo: &Path) -> Result<PruneTasksResult> {
    let root = paths::Paths::load(repo).specs_root;
    let feature_dir = repo.join(&root).join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            root,
            feature: args.feature.clone(),
        });
    }
    let tasks_path = feature_dir.join("tasks.md");
    if !tasks_path.is_file() {
        return Err(PrimitiveError::TasksFileMissing {
            root,
            feature: args.feature.clone(),
        });
    }
    let content = read_text(&tasks_path)?;

    // The `--reset` status gate reads the spec status before touching tasks.
    let (mode, gate, status) = if args.reset {
        let status = read_status(&feature_dir, &root, &args.feature)?;
        let gate = if status == "done" || args.force {
            PruneGate::Allowed
        } else {
            PruneGate::BlockedNeedsForce
        };
        (PruneMode::Reset, gate, Some(status))
    } else {
        (PruneMode::KeepPending, PruneGate::NotApplicable, None)
    };

    let blocks = segment(&content);

    // Per-mode reduction: compute the would-be output, the section records,
    // and the removed/kept counts.
    let (new_content, sections, removed, kept) = match mode {
        PruneMode::KeepPending => reduce_keep_pending(&content, &blocks),
        PruneMode::Reset => reduce_reset(&content, &blocks, &tasks_path)?,
    };

    let nothing_to_prune = new_content == content;
    // A write happens only on `apply`, only when there is a change, and (for
    // reset) only when the gate permits it. keep-pending is never gated.
    let gate_permits = !matches!(gate, PruneGate::BlockedNeedsForce);
    let applied = args.apply && !nothing_to_prune && gate_permits;
    if applied {
        write_atomic(&tasks_path, &new_content)?;
    }

    Ok(PruneTasksResult {
        mode,
        applied,
        gate,
        status,
        nothing_to_prune,
        removed_count: removed,
        kept_count: kept,
        size_before: size_of(&content),
        size_after: size_of(&new_content),
        sections,
        path: rel_path(&tasks_path, repo),
    })
}

/// Read the spec's frontmatter `status` for the `--reset` gate.
fn read_status(feature_dir: &Path, root: &str, feature: &str) -> Result<String> {
    let spec_path = feature_dir.join("spec.md");
    if !spec_path.is_file() {
        return Err(PrimitiveError::MissingSpecFile {
            root: root.to_string(),
            feature: feature.to_string(),
        });
    }
    let spec = read_text(&spec_path)?;
    let (frontmatter, _body) = split_frontmatter(&spec, &spec_path)?;
    let parsed: StatusOnly =
        serde_norway::from_str(frontmatter).map_err(|source| PrimitiveError::Yaml {
            path: spec_path.clone(),
            source,
        })?;
    parsed.status.ok_or(PrimitiveError::StatusFieldMissing {
        root: root.to_string(),
        feature: feature.to_string(),
    })
}

/// Segment `content` into structure / phase / task blocks in document order.
fn segment(content: &str) -> Vec<Block> {
    let task_level: u8 = match detect_tasks_structure(content) {
        TasksStructure::Flat => 2,
        TasksStructure::Phased => 3,
    };
    let phased = task_level == 3;

    let mut blocks: Vec<Block> = Vec::new();
    let mut cur = Block::new(Kind::Structure, "");
    // The initial block starts empty (no first line); clear the placeholder.
    cur.lines.clear();
    let mut in_fence = false;
    let mut current_phase_name: Option<String> = None;
    let mut current_phase_idx: Option<usize> = None;

    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            cur.lines.push(line.to_string());
            continue;
        }
        if in_fence {
            cur.lines.push(line.to_string());
            continue;
        }
        if let Some((level, heading)) = parse_atx_heading(line)
            && level <= task_level
        {
            // Close the current block; record a pushed phase's index.
            let closed_kind = cur.kind;
            blocks.push(std::mem::replace(
                &mut cur,
                Block::new(Kind::Structure, line),
            ));
            if closed_kind == Kind::Phase {
                current_phase_idx = Some(blocks.len() - 1);
            }

            let is_task = level == task_level && heading_is_numeric(&heading);
            let is_phase = phased && level == 2 && !heading_is_numeric(&heading);
            if is_task {
                cur.kind = Kind::Task;
                let (number, title) = split_numbered_heading(&heading);
                cur.number = number;
                cur.heading = title;
                cur.phase.clone_from(&current_phase_name);
                cur.governing_phase = current_phase_idx;
            } else if is_phase {
                cur.kind = Kind::Phase;
                current_phase_name = Some(heading);
            } else {
                cur.kind = Kind::Structure;
                if level == 1 {
                    // A top-level heading resets phase context.
                    current_phase_name = None;
                    current_phase_idx = None;
                }
            }
            continue;
        }

        // Body line: attach to the current block, counting checkboxes when
        // this is a task section.
        if cur.kind == Kind::Task
            && let Some((_bracket, marker)) = checkbox::find_checkbox_line(line)
        {
            cur.checkbox_total += 1;
            if matches!(line.as_bytes()[marker], b'x' | b'X') {
                cur.checkbox_checked += 1;
            }
        }
        cur.lines.push(line.to_string());
    }
    blocks.push(cur);
    blocks
}

/// keep-pending reduction: drop spent task sections and emptied phase
/// containers. Returns `(new_content, sections, removed, kept)`.
fn reduce_keep_pending(content: &str, blocks: &[Block]) -> (String, Vec<PruneSection>, u32, u32) {
    // Phases with at least one surviving (non-spent) task section.
    let mut phase_has_survivor: HashSet<usize> = HashSet::new();
    for block in blocks {
        if block.kind == Kind::Task
            && block.classification() != Classification::Spent
            && let Some(p) = block.governing_phase
        {
            phase_has_survivor.insert(p);
        }
    }

    let mut sections = Vec::new();
    let mut removed = 0u32;
    let mut kept = 0u32;
    let mut kept_lines: Vec<&Block> = Vec::new();
    let mut dropped_any = false;

    for (idx, block) in blocks.iter().enumerate() {
        match block.kind {
            Kind::Structure => kept_lines.push(block),
            Kind::Phase => {
                if phase_has_survivor.contains(&idx) {
                    kept_lines.push(block);
                } else {
                    dropped_any = true;
                }
            }
            Kind::Task => {
                let spent = block.classification() == Classification::Spent;
                sections.push(section_record(
                    block,
                    if spent {
                        PruneAction::Removed
                    } else {
                        PruneAction::Kept
                    },
                ));
                if spent {
                    removed += 1;
                    dropped_any = true;
                } else {
                    kept += 1;
                    kept_lines.push(block);
                }
            }
        }
    }

    // No spent section and no dropped phase: leave the file byte-for-byte
    // unchanged rather than reformat seams.
    let new_content = if dropped_any {
        render(&kept_lines)
    } else {
        content.to_string()
    };
    (new_content, sections, removed, kept)
}

/// reset reduction: existing H1 + [`CANONICAL_EMPTY_TASKS_BODY`]. Every task
/// section is reported as removed.
fn reduce_reset(
    _content: &str,
    blocks: &[Block],
    tasks_path: &Path,
) -> Result<(String, Vec<PruneSection>, u32, u32)> {
    let h1 = blocks
        .iter()
        .flat_map(|b| b.lines.iter())
        .find(|line| matches!(parse_atx_heading(line.as_str()), Some((1, _))))
        .ok_or_else(|| PrimitiveError::MalformedTasks {
            path: tasks_path.to_path_buf(),
            reason: "no top-level (`#`) heading to preserve the feature identity".to_string(),
        })?;

    let new_content = format!("{h1}\n\n{CANONICAL_EMPTY_TASKS_BODY}");

    let mut sections = Vec::new();
    let mut removed = 0u32;
    for block in blocks {
        if block.kind == Kind::Task {
            sections.push(section_record(block, PruneAction::Removed));
            removed += 1;
        }
    }
    Ok((new_content, sections, removed, 0))
}

/// Build a compact per-section record (identity + classification + counts).
fn section_record(block: &Block, action: PruneAction) -> PruneSection {
    PruneSection {
        number: block.number.clone(),
        heading: block.heading.clone(),
        phase: block.phase.clone(),
        classification: block.classification(),
        checkbox_total: block.checkbox_total,
        checkbox_checked: block.checkbox_checked,
        action,
    }
}

/// Render kept blocks: one blank line between blocks, single trailing
/// newline, no leading blanks — `markdownlint`-clean seams.
fn render(blocks: &[&Block]) -> String {
    let rendered: Vec<String> = blocks
        .iter()
        .map(|b| render_block(&b.lines))
        .filter(|s| !s.is_empty())
        .collect();
    let mut out = rendered.join("\n\n");
    out.push('\n');
    out
}

/// Join a block's lines, trimming leading and trailing blank lines.
fn render_block(lines: &[String]) -> String {
    let mut start = 0;
    let mut end = lines.len();
    while start < end && lines[start].trim().is_empty() {
        start += 1;
    }
    while end > start && lines[end - 1].trim().is_empty() {
        end -= 1;
    }
    lines[start..end].join("\n")
}

fn size_of(content: &str) -> SizeSummary {
    SizeSummary {
        lines: content.lines().count(),
        bytes: content.len(),
    }
}

/// `true` when `heading` begins with `N.` (decimal digits, then a dot).
fn heading_is_numeric(heading: &str) -> bool {
    let bytes = heading.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    i > 0 && i < bytes.len() && bytes[i] == b'.'
}

/// Split a numbered heading (`"12. Title"`) into `(number, title)`. Returns
/// an empty title when there is no text after the number.
fn split_numbered_heading(heading: &str) -> (String, String) {
    let bytes = heading.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    let number = heading[..i].to_string();
    let after = heading[i..].strip_prefix('.').unwrap_or(&heading[i..]);
    (number, after.trim_start().to_string())
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn write_repo(tasks: &str, status: Option<&str>) -> (tempfile::TempDir, PathBuf) {
        let tmp = tempdir().unwrap();
        let dir = tmp.path().join("specs/041-task-pruning");
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("tasks.md"), tasks).unwrap();
        if let Some(s) = status {
            fs::write(
                dir.join("spec.md"),
                format!("---\nstatus: {s}\ndependencies: []\n---\n\n# Spec\n"),
            )
            .unwrap();
        }
        let repo = tmp.path().to_path_buf();
        (tmp, repo)
    }

    fn args(reset: bool, force: bool, apply: bool) -> PruneTasksArgs {
        PruneTasksArgs {
            feature: "041-task-pruning".into(),
            reset,
            force,
            apply,
        }
    }

    const FLAT: &str = "# 041 — Task Pruning Tasks\n\nTasks derived from the [plan](plan.md). Complete in order.\n\n## 1. Done task\n\n- [x] a\n- [x] b\n\n## 2. Pending task\n\n- [ ] c\n- [x] d\n\n## 3. Prose task\n\nNo checkboxes here.\n";

    #[test]
    fn keep_pending_drops_spent_preserves_pending_and_prose() {
        let (_tmp, repo) = write_repo(FLAT, None);
        let result = run(&args(false, false, true), &repo).unwrap();
        assert_eq!(result.mode, PruneMode::KeepPending);
        assert_eq!(result.gate, PruneGate::NotApplicable);
        assert!(result.applied);
        assert_eq!(result.removed_count, 1);
        assert_eq!(result.kept_count, 2);
        let written = fs::read_to_string(repo.join("specs/041-task-pruning/tasks.md")).unwrap();
        assert!(!written.contains("## 1. Done task"));
        assert!(written.contains("## 2. Pending task"));
        assert!(written.contains("## 3. Prose task"));
        // Preamble preserved.
        assert!(written.starts_with("# 041 — Task Pruning Tasks\n\nTasks derived"));
        // Output ends with exactly one trailing newline and no double blanks.
        assert!(written.ends_with('\n') && !written.ends_with("\n\n"));
        assert!(!written.contains("\n\n\n"));
    }

    #[test]
    fn preview_does_not_write() {
        let (_tmp, repo) = write_repo(FLAT, None);
        let before = fs::read_to_string(repo.join("specs/041-task-pruning/tasks.md")).unwrap();
        let result = run(&args(false, false, false), &repo).unwrap();
        assert!(!result.applied);
        assert_eq!(result.removed_count, 1);
        let after = fs::read_to_string(repo.join("specs/041-task-pruning/tasks.md")).unwrap();
        assert_eq!(before, after, "preview must not modify the file");
    }

    #[test]
    fn keep_pending_no_op_when_nothing_spent() {
        let tasks = "# T\n\nTasks derived from the [plan](plan.md). Complete in order.\n\n## 1. Pending\n\n- [ ] a\n";
        let (_tmp, repo) = write_repo(tasks, None);
        let result = run(&args(false, false, true), &repo).unwrap();
        assert!(result.nothing_to_prune);
        assert!(!result.applied);
        assert_eq!(result.removed_count, 0);
        let after = fs::read_to_string(repo.join("specs/041-task-pruning/tasks.md")).unwrap();
        assert_eq!(after, tasks, "no-op must leave the file byte-for-byte");
    }

    const PHASED: &str = "# T\n\nTasks derived from the [plan](plan.md). Complete in order.\n\n## Phase A — Done\n\n### 1. Done one\n\n- [x] a\n\n### 2. Done two\n\n- [x] b\n\n## Phase B — Live\n\n### 3. Pending\n\n- [ ] c\n";

    #[test]
    fn keep_pending_phased_drops_spent_and_empty_phase() {
        let (_tmp, repo) = write_repo(PHASED, None);
        let result = run(&args(false, false, true), &repo).unwrap();
        assert_eq!(result.removed_count, 2);
        assert_eq!(result.kept_count, 1);
        let written = fs::read_to_string(repo.join("specs/041-task-pruning/tasks.md")).unwrap();
        // Phase A had only spent tasks → dropped entirely.
        assert!(!written.contains("Phase A — Done"));
        assert!(!written.contains("### 1. Done one"));
        // Phase B and its pending task survive.
        assert!(written.contains("## Phase B — Live"));
        assert!(written.contains("### 3. Pending"));
        assert!(!written.contains("\n\n\n"));
    }

    #[test]
    fn reset_produces_template_state_when_done() {
        let (_tmp, repo) = write_repo(FLAT, Some("done"));
        let result = run(&args(true, false, true), &repo).unwrap();
        assert_eq!(result.mode, PruneMode::Reset);
        assert_eq!(result.gate, PruneGate::Allowed);
        assert!(result.applied);
        let written = fs::read_to_string(repo.join("specs/041-task-pruning/tasks.md")).unwrap();
        let expected = format!("# 041 — Task Pruning Tasks\n\n{CANONICAL_EMPTY_TASKS_BODY}");
        assert_eq!(written, expected);
    }

    #[test]
    fn reset_blocked_on_non_done_without_force() {
        let (_tmp, repo) = write_repo(FLAT, Some("in-progress"));
        let before = fs::read_to_string(repo.join("specs/041-task-pruning/tasks.md")).unwrap();
        let result = run(&args(true, false, true), &repo).unwrap();
        assert_eq!(result.gate, PruneGate::BlockedNeedsForce);
        assert!(!result.applied, "blocked reset must not write");
        let after = fs::read_to_string(repo.join("specs/041-task-pruning/tasks.md")).unwrap();
        assert_eq!(before, after);
    }

    #[test]
    fn reset_forced_on_non_done_writes() {
        let (_tmp, repo) = write_repo(FLAT, Some("in-progress"));
        let result = run(&args(true, true, true), &repo).unwrap();
        assert_eq!(result.gate, PruneGate::Allowed);
        assert!(result.applied);
        assert_eq!(result.status.as_deref(), Some("in-progress"));
    }

    #[test]
    fn missing_tasks_file_errors() {
        let tmp = tempdir().unwrap();
        let dir = tmp.path().join("specs/041-task-pruning");
        fs::create_dir_all(&dir).unwrap();
        let err = run(&args(false, false, false), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::TasksFileMissing { .. }));
    }

    #[test]
    fn reset_on_file_without_h1_errors() {
        let tasks = "Tasks derived from the [plan](plan.md).\n\n## 1. X\n\n- [x] a\n";
        let (_tmp, repo) = write_repo(tasks, Some("done"));
        let err = run(&args(true, false, true), &repo).unwrap_err();
        assert!(matches!(err, PrimitiveError::MalformedTasks { .. }));
    }

    #[test]
    fn canonical_empty_body_matches_template() {
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();
        let template =
            fs::read_to_string(repo_root.join("framework/templates/spec/tasks.md")).unwrap();
        // Strip the H1 line and any leading blank lines that follow it.
        let after_h1 = template.split_once('\n').map(|(_, rest)| rest).unwrap();
        let body = after_h1.trim_start_matches('\n');
        assert_eq!(
            body.trim_end(),
            CANONICAL_EMPTY_TASKS_BODY.trim_end(),
            "CANONICAL_EMPTY_TASKS_BODY drifted from framework/templates/spec/tasks.md"
        );
    }
}
