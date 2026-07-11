//! `check-artifacts` — the residual deterministic check families from
//! `/gov:analyze`'s markdown-only reference, mechanized for one feature.
//!
//! Owns four families (spec 022, scenario analyze-artifact-checks). Each
//! family MIRRORS `framework/commands/analyze.md`'s markdown-only
//! reference — severity tiers and skip rules come from the reference, the
//! primitive introduces no policy of its own:
//!
//! - **artifact-completeness** (blocking) — reference §"Artifact
//!   completeness (blocking)" (analyze.md lines 97–101): `plan.md` and
//!   `tasks.md` are required when status is `planned` or later
//!   (`planned` / `in-progress` / `done`). `data-model.md` is **never**
//!   required here: the reference conditions it on "feature introduces or
//!   modifies domain entities" — a semantic judgment the runtime cannot
//!   make deterministically, so it stays optional (and with the prose
//!   check on the markdown-only path).
//! - **task-consistency** (blocking) — reference §"Task consistency
//!   (blocking if tasks exist)" (analyze.md lines 110–114): task numbers
//!   are strictly increasing in declaration order, and every task section
//!   carries a `Done when` clause. The reference's "tasks reference the
//!   plan" item is a semantic-link judgment and stays in the
//!   markdown-only reference. Runs only when `tasks.md` exists.
//! - **scenario-consistency** (advisory) — reference §"Scenario
//!   consistency (advisory)" (analyze.md lines 116–120): every
//!   `scenarios/*.md` has a referencing task in `tasks.md` *only while
//!   that task is still pending*. Never flags a scenario under a `done`
//!   spec, and never requires a pruned spent task to persist
//!   (constitution §tasks-phase — `tasks.md` is ephemeral; see
//!   [`pruning_evidence`] for the documented heuristic).
//! - **review-state-drift** (blocking) — reference §"Review state drift
//!   (blocking)" (analyze.md lines 142–153): a `done` spec with
//!   `review.last-run` unset, or `review.blocking: true`, drifted. The
//!   grandfather rule applies: a `done` spec with no `review:` block at
//!   all predates `/gov:review` and is exempt.
//!
//! Parsing reuses the shared machinery — `split_frontmatter` for the spec
//! frontmatter and [`crate::primitives::read_tasks`] for the task list —
//! so this primitive sees exactly the artifact structure every other
//! primitive sees (no hand-rolled parsers).

use std::path::Path;

use crate::primitives::{
    PrimitiveError, Result, list_scenario_files, read_tasks, read_text, rel_path, split_frontmatter,
};
use crate::schema::paths;
use crate::schema::primitives::{
    ArtifactFinding, CheckArtifactsArgs, CheckArtifactsResult, Frontmatter, ReadTasksArgs, Task,
};
use crate::schema::status::COMPATIBLE_STATUSES;

/// Execute the `check-artifacts` primitive against the given repo root.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeatureNotFound`] when the feature directory
/// is absent, [`PrimitiveError::MissingFrontmatter`] /
/// [`PrimitiveError::Yaml`] when `spec.md` has no parseable frontmatter
/// (the frontmatter-schema family is `validate-frontmatter`'s job — this
/// primitive needs a readable `status` to classify tiers at all), or
/// [`PrimitiveError::Io`] on filesystem failures.
pub fn run(args: &CheckArtifactsArgs, repo: &Path) -> Result<CheckArtifactsResult> {
    let root = paths::Paths::load(repo).specs_root;
    let feature_dir = repo.join(&root).join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            root,
            feature: args.feature.clone(),
        });
    }
    let spec_path = feature_dir.join("spec.md");
    let content = read_text(&spec_path)?;
    let (fm_text, _body) = split_frontmatter(&content, &spec_path)?;
    let frontmatter: Frontmatter =
        serde_norway::from_str(fm_text).map_err(|source| PrimitiveError::Yaml {
            path: spec_path.clone(),
            source,
        })?;
    let status = frontmatter.status.clone();

    let mut findings: Vec<ArtifactFinding> = Vec::new();
    check_completeness(&mut findings, &feature_dir, &root, &args.feature, &status);

    // Task parsing is shared by families (b) and (c); parse once.
    let tasks = if feature_dir.join("tasks.md").is_file() {
        Some(read_tasks::run(
            &ReadTasksArgs {
                feature: args.feature.clone(),
            },
            repo,
        )?)
    } else {
        None
    };
    if let Some(tasks) = &tasks {
        check_task_consistency(&mut findings, &tasks.tasks, &tasks.path);
    }
    check_scenario_consistency(
        &mut findings,
        &feature_dir,
        &root,
        &args.feature,
        &status,
        tasks.as_ref().map(|t| t.tasks.as_slice()),
    );
    check_review_drift(&mut findings, &frontmatter, &status, &spec_path, repo);

    let clean = findings.is_empty();
    Ok(CheckArtifactsResult {
        feature: args.feature.clone(),
        status,
        findings,
        clean,
        path: rel_path(&spec_path, repo),
    })
}

/// (a) Artifact completeness — reference §"Artifact completeness
/// (blocking)": `plan.md` / `tasks.md` required at `planned` or later. A
/// `draft` or `clarified` spec with neither file produces no finding
/// (files are required by status tier, not universally). `data-model.md`
/// is never required (see module docs).
fn check_completeness(
    findings: &mut Vec<ArtifactFinding>,
    feature_dir: &Path,
    root: &str,
    feature: &str,
    status: &str,
) {
    // "planned or later" is the same lifecycle tail `schema::status`
    // derives as `COMPATIBLE_STATUSES` (planned / in-progress / done).
    if !COMPATIBLE_STATUSES.contains(&status) {
        return;
    }
    for file in ["plan.md", "tasks.md"] {
        if !feature_dir.join(file).is_file() {
            findings.push(ArtifactFinding {
                family: "artifact-completeness".into(),
                severity: "blocking".into(),
                message: format!("{file} is required at status '{status}' but does not exist"),
                path: format!("{root}/{feature}/{file}"),
            });
        }
    }
}

/// (b) Task consistency — reference §"Task consistency (blocking if
/// tasks exist)": numbered headings strictly increasing in declaration
/// order, and every task section carries a `Done when` clause.
fn check_task_consistency(findings: &mut Vec<ArtifactFinding>, tasks: &[Task], tasks_path: &str) {
    let mut prev: Option<u32> = None;
    for task in tasks {
        if let Ok(number) = task.number.parse::<u32>() {
            if let Some(previous) = prev
                && number <= previous
            {
                findings.push(ArtifactFinding {
                    family: "task-consistency".into(),
                    severity: "blocking".into(),
                    message: format!(
                        "task numbering is not strictly increasing: task {number} follows task {previous}"
                    ),
                    path: tasks_path.to_string(),
                });
            }
            prev = Some(number);
        }
        if task.done_when.is_none() {
            findings.push(ArtifactFinding {
                family: "task-consistency".into(),
                severity: "blocking".into(),
                message: format!(
                    "task {} ({}) has no Done when clause",
                    task.number, task.heading
                ),
                path: tasks_path.to_string(),
            });
        }
    }
}

/// (c) Scenario→task mapping — reference §"Scenario consistency
/// (advisory)". Skip rules, in order:
///
/// - Spec at `done` → the whole family is skipped (its tasks may have
///   been pruned or the file reset; the durable record is the scenario
///   file, the code, and git history).
/// - No `tasks.md` → not evaluable; the completeness family already owns
///   the missing-file signal at `planned`+, and a pre-plan spec's
///   scenarios have no tasks yet by design.
/// - `tasks.md` shows [`pruning_evidence`] → the mapping is satisfied for
///   every unmapped scenario (§tasks-phase: a pruned spent task never
///   produces a finding).
///
/// A scenario is *mapped* when its slug appears in any task's heading,
/// subtask text, or `Done when` clause — this matches `append-task`'s
/// default-body convention (`scenarios/{slug}.md`) while tolerating
/// hand-written references that name the slug without the path.
fn check_scenario_consistency(
    findings: &mut Vec<ArtifactFinding>,
    feature_dir: &Path,
    root: &str,
    feature: &str,
    status: &str,
    tasks: Option<&[Task]>,
) {
    if status == "done" {
        return;
    }
    let Some(tasks) = tasks else {
        return;
    };
    let slugs = scenario_slugs(feature_dir);
    if slugs.is_empty() {
        return;
    }
    if pruning_evidence(tasks) {
        return;
    }
    for slug in slugs {
        if !scenario_mapped(tasks, &slug) {
            findings.push(ArtifactFinding {
                family: "scenario-consistency".into(),
                severity: "advisory".into(),
                message: format!(
                    "scenario {slug}.md has no corresponding task in tasks.md and the file \
                     shows no pruning evidence"
                ),
                path: format!("{root}/{feature}/scenarios/{slug}.md"),
            });
        }
    }
}

/// List scenario slugs (`*.md` basenames without extension) under the
/// feature's `scenarios/` directory, sorted. Empty when the directory is
/// absent. Enumerates via the shared [`list_scenario_files`] so the `.md`
/// match is CASE-INSENSITIVE — the same set `dashboard` counts, closing
/// the `FOO.MD`-counted-by-one-surface-only divergence.
fn scenario_slugs(feature_dir: &Path) -> Vec<String> {
    let mut slugs: Vec<String> = list_scenario_files(&feature_dir.join("scenarios"))
        .iter()
        .filter_map(|name| {
            Path::new(name)
                .file_stem()
                .and_then(|stem| stem.to_str())
                .map(str::to_string)
        })
        .collect();
    slugs.sort();
    slugs
}

/// `true` when any task references the scenario slug (heading, subtask
/// text, or `Done when` clause).
fn scenario_mapped(tasks: &[Task], slug: &str) -> bool {
    tasks.iter().any(|task| {
        task.heading.contains(slug)
            || task.subtasks.iter().any(|s| s.text.contains(slug))
            || task.done_when.as_deref().is_some_and(|d| d.contains(slug))
    })
}

/// Pruning-evidence heuristic (§tasks-phase). `prune-tasks` reduces a
/// `tasks.md` in two shapes, and each leaves a deterministic fingerprint:
///
/// - **reset** rewrites the file to template state → the file parses to
///   **zero task sections**;
/// - **keep-pending** drops spent sections verbatim without renumbering
///   the survivors → the surviving numbers are **non-contiguous** (the
///   first number exceeds 1, or a gap appears between consecutive
///   numbers).
///
/// Either fingerprint counts as evidence. The heuristic is deliberately
/// coarse: evidence anywhere in the file vouches for *every* unmapped
/// scenario, because the primitive cannot know which pruned section
/// referenced which scenario — and §tasks-phase forbids requiring a spent
/// task to persist, so the mandated direction of error is the missed
/// finding, never the false one. (A fresh template-state `tasks.md` on a
/// pre-implementation spec also matches the zero-sections fingerprint and
/// is likewise not flagged — same direction of error.)
fn pruning_evidence(tasks: &[Task]) -> bool {
    if tasks.is_empty() {
        return true;
    }
    let numbers: Vec<u32> = tasks
        .iter()
        .filter_map(|t| t.number.parse::<u32>().ok())
        .collect();
    if let Some(first) = numbers.first()
        && *first > 1
    {
        return true;
    }
    numbers.windows(2).any(|pair| pair[1] > pair[0] + 1)
}

/// (d) Review-state drift — reference §"Review state drift (blocking)":
/// for a `done` spec, `review.last-run` must be set and `review.blocking`
/// must be `false`. Grandfather rule: a `done` spec with **no** `review:`
/// block at all predates `/gov:review` and is exempt. Specs not at `done`
/// are silently exempt (the block populates lazily on first review).
fn check_review_drift(
    findings: &mut Vec<ArtifactFinding>,
    frontmatter: &Frontmatter,
    status: &str,
    spec_path: &Path,
    repo: &Path,
) {
    if status != "done" {
        return;
    }
    let Some(review) = &frontmatter.review else {
        return; // grandfathered: no review block at all
    };
    let spec_rel = rel_path(spec_path, repo);
    if review.last_run.is_none() {
        findings.push(ArtifactFinding {
            family: "review-state-drift".into(),
            severity: "blocking".into(),
            message: "review drift: done spec missing review (review.last-run unset) — \
                      run the review command"
                .into(),
            path: spec_rel.clone(),
        });
    }
    if review.blocking {
        findings.push(ArtifactFinding {
            family: "review-state-drift".into(),
            severity: "blocking".into(),
            message: "review drift: done spec has unresolved MUST violations \
                      (review.blocking true) — see review.md"
                .into(),
            path: spec_rel,
        });
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;
    use tempfile::tempdir;

    const FEATURE: &str = "042-demo";

    fn args() -> CheckArtifactsArgs {
        CheckArtifactsArgs {
            feature: FEATURE.into(),
        }
    }

    fn write(repo: &Path, rel: &str, body: &str) {
        let path = repo.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    fn spec(status: &str, review: Option<&str>) -> String {
        let review_block = review
            .map(|r| format!("review:\n{r}\n"))
            .unwrap_or_default();
        format!("---\nstatus: {status}\ndependencies: []\n{review_block}---\n\n# Demo\n")
    }

    const GOOD_TASKS: &str = "# Demo Tasks\n\n\
        ## 1. Implement retry\n\n\
        - [x] Implement the behavior described in `scenarios/retry-on-timeout.md`\n\n\
        - **Done when**: retries pass.\n\n\
        ## 2. Wire CLI\n\n\
        - [ ] sub\n\n\
        - **Done when**: CLI works.\n";

    fn families(result: &CheckArtifactsResult) -> Vec<(&str, &str)> {
        result
            .findings
            .iter()
            .map(|f| (f.family.as_str(), f.severity.as_str()))
            .collect()
    }

    // --- artifact completeness -------------------------------------------------

    #[test]
    fn draft_spec_with_no_plan_or_tasks_is_clean() {
        // Edge case from the scenario: files are required by status tier,
        // not universally.
        let tmp = tempdir().unwrap();
        write(tmp.path(), "specs/042-demo/spec.md", &spec("draft", None));
        let result = run(&args(), tmp.path()).unwrap();
        assert!(result.clean, "{:?}", result.findings);
        assert_eq!(result.status, "draft");
        assert_eq!(result.path, "specs/042-demo/spec.md");
    }

    #[test]
    fn planned_spec_missing_plan_and_tasks_yields_blocking_findings() {
        let tmp = tempdir().unwrap();
        write(tmp.path(), "specs/042-demo/spec.md", &spec("planned", None));
        let result = run(&args(), tmp.path()).unwrap();
        assert_eq!(
            families(&result),
            vec![
                ("artifact-completeness", "blocking"),
                ("artifact-completeness", "blocking")
            ]
        );
        assert!(result.findings[0].message.contains("plan.md"));
        assert_eq!(result.findings[0].path, "specs/042-demo/plan.md");
        assert!(result.findings[1].message.contains("tasks.md"));
    }

    #[test]
    fn data_model_is_never_required() {
        let tmp = tempdir().unwrap();
        write(tmp.path(), "specs/042-demo/spec.md", &spec("planned", None));
        write(tmp.path(), "specs/042-demo/plan.md", "# Plan\n");
        write(tmp.path(), "specs/042-demo/tasks.md", GOOD_TASKS);
        let result = run(&args(), tmp.path()).unwrap();
        assert!(
            result.clean,
            "no data-model.md finding expected: {:?}",
            result.findings
        );
    }

    // --- task consistency --------------------------------------------------------

    #[test]
    fn strictly_increasing_numbered_tasks_with_done_when_are_clean() {
        let tmp = tempdir().unwrap();
        write(
            tmp.path(),
            "specs/042-demo/spec.md",
            &spec("in-progress", None),
        );
        write(tmp.path(), "specs/042-demo/plan.md", "# Plan\n");
        write(tmp.path(), "specs/042-demo/tasks.md", GOOD_TASKS);
        let result = run(&args(), tmp.path()).unwrap();
        assert!(result.clean, "{:?}", result.findings);
    }

    #[test]
    fn non_increasing_numbering_yields_blocking_finding() {
        let tmp = tempdir().unwrap();
        write(
            tmp.path(),
            "specs/042-demo/spec.md",
            &spec("in-progress", None),
        );
        write(tmp.path(), "specs/042-demo/plan.md", "# Plan\n");
        let tasks = "# T\n\n\
            ## 2. Second\n\n- [ ] a\n\n- **Done when**: done.\n\n\
            ## 1. Out of order\n\n- [ ] b\n\n- **Done when**: done.\n";
        write(tmp.path(), "specs/042-demo/tasks.md", tasks);
        let result = run(&args(), tmp.path()).unwrap();
        let numbering: Vec<&ArtifactFinding> = result
            .findings
            .iter()
            .filter(|f| f.message.contains("strictly increasing"))
            .collect();
        assert_eq!(numbering.len(), 1);
        assert_eq!(numbering[0].family, "task-consistency");
        assert_eq!(numbering[0].severity, "blocking");
        assert_eq!(numbering[0].path, "specs/042-demo/tasks.md");
        assert!(numbering[0].message.contains("task 1 follows task 2"));
    }

    #[test]
    fn missing_done_when_yields_blocking_finding() {
        let tmp = tempdir().unwrap();
        write(
            tmp.path(),
            "specs/042-demo/spec.md",
            &spec("in-progress", None),
        );
        write(tmp.path(), "specs/042-demo/plan.md", "# Plan\n");
        let tasks = "# T\n\n## 1. No done-when\n\n- [ ] a\n";
        write(tmp.path(), "specs/042-demo/tasks.md", tasks);
        let result = run(&args(), tmp.path()).unwrap();
        assert_eq!(families(&result), vec![("task-consistency", "blocking")]);
        assert!(
            result.findings[0]
                .message
                .contains("task 1 (No done-when) has no Done when clause")
        );
    }

    #[test]
    fn task_checks_skip_when_tasks_file_absent() {
        // "blocking if tasks exist" — a clarified spec with no tasks.md
        // gets no task-consistency findings (and no completeness ones
        // either, below the planned tier).
        let tmp = tempdir().unwrap();
        write(
            tmp.path(),
            "specs/042-demo/spec.md",
            &spec("clarified", None),
        );
        let result = run(&args(), tmp.path()).unwrap();
        assert!(result.clean, "{:?}", result.findings);
    }

    // --- scenario consistency ------------------------------------------------------

    #[test]
    fn unmapped_scenario_yields_advisory_finding() {
        let tmp = tempdir().unwrap();
        write(
            tmp.path(),
            "specs/042-demo/spec.md",
            &spec("in-progress", None),
        );
        write(tmp.path(), "specs/042-demo/plan.md", "# Plan\n");
        write(tmp.path(), "specs/042-demo/tasks.md", GOOD_TASKS);
        write(
            tmp.path(),
            "specs/042-demo/scenarios/unmapped-scenario.md",
            "---\nsection: \"X\"\n---\n\n# Unmapped\n",
        );
        let result = run(&args(), tmp.path()).unwrap();
        assert_eq!(
            families(&result),
            vec![("scenario-consistency", "advisory")]
        );
        assert_eq!(
            result.findings[0].path,
            "specs/042-demo/scenarios/unmapped-scenario.md"
        );
    }

    #[test]
    fn mapped_scenario_produces_no_finding() {
        let tmp = tempdir().unwrap();
        write(
            tmp.path(),
            "specs/042-demo/spec.md",
            &spec("in-progress", None),
        );
        write(tmp.path(), "specs/042-demo/plan.md", "# Plan\n");
        write(tmp.path(), "specs/042-demo/tasks.md", GOOD_TASKS);
        write(
            tmp.path(),
            "specs/042-demo/scenarios/retry-on-timeout.md",
            "---\nsection: \"X\"\n---\n\n# Retry\n",
        );
        let result = run(&args(), tmp.path()).unwrap();
        assert!(result.clean, "{:?}", result.findings);
    }

    #[test]
    fn pruned_gap_numbering_satisfies_the_mapping() {
        // Scenario edge case: a scenario whose task was pruned after
        // completion produces no finding. keep-pending pruning leaves
        // non-contiguous numbers (task 1 was dropped; 2 survives).
        let tmp = tempdir().unwrap();
        write(
            tmp.path(),
            "specs/042-demo/spec.md",
            &spec("in-progress", None),
        );
        write(tmp.path(), "specs/042-demo/plan.md", "# Plan\n");
        let pruned = "# T\n\n## 2. Wire CLI\n\n- [ ] sub\n\n- **Done when**: CLI works.\n";
        write(tmp.path(), "specs/042-demo/tasks.md", pruned);
        write(
            tmp.path(),
            "specs/042-demo/scenarios/pruned-away.md",
            "---\nsection: \"X\"\n---\n\n# Pruned\n",
        );
        let result = run(&args(), tmp.path()).unwrap();
        assert!(
            result.clean,
            "pruning evidence must satisfy the mapping: {:?}",
            result.findings
        );
    }

    #[test]
    fn reset_template_tasks_satisfy_the_mapping() {
        // Reset-to-template parses as zero tasks — the other pruning
        // fingerprint (§tasks-phase).
        let tmp = tempdir().unwrap();
        write(
            tmp.path(),
            "specs/042-demo/spec.md",
            &spec("in-progress", None),
        );
        write(tmp.path(), "specs/042-demo/plan.md", "# Plan\n");
        write(
            tmp.path(),
            "specs/042-demo/tasks.md",
            "# T\n\nTasks derived from the [plan](plan.md). Complete in order.\n",
        );
        write(
            tmp.path(),
            "specs/042-demo/scenarios/reset-away.md",
            "---\nsection: \"X\"\n---\n\n# Reset\n",
        );
        let result = run(&args(), tmp.path()).unwrap();
        assert!(result.clean, "{:?}", result.findings);
    }

    #[test]
    fn done_spec_scenarios_are_never_flagged() {
        let tmp = tempdir().unwrap();
        write(
            tmp.path(),
            "specs/042-demo/spec.md",
            &spec(
                "done",
                Some("  last-run: 2026-07-01T00:00:00Z\n  blocking: false"),
            ),
        );
        write(tmp.path(), "specs/042-demo/plan.md", "# Plan\n");
        write(tmp.path(), "specs/042-demo/tasks.md", GOOD_TASKS);
        write(
            tmp.path(),
            "specs/042-demo/scenarios/unmapped-under-done.md",
            "---\nsection: \"X\"\n---\n\n# X\n",
        );
        let result = run(&args(), tmp.path()).unwrap();
        assert!(result.clean, "{:?}", result.findings);
    }

    // --- review state drift ---------------------------------------------------------

    #[test]
    fn done_spec_with_unset_last_run_yields_blocking_finding() {
        let tmp = tempdir().unwrap();
        write(
            tmp.path(),
            "specs/042-demo/spec.md",
            &spec("done", Some("  blocking: false")),
        );
        write(tmp.path(), "specs/042-demo/plan.md", "# Plan\n");
        write(tmp.path(), "specs/042-demo/tasks.md", GOOD_TASKS);
        let result = run(&args(), tmp.path()).unwrap();
        assert_eq!(families(&result), vec![("review-state-drift", "blocking")]);
        assert!(result.findings[0].message.contains("review.last-run unset"));
        assert_eq!(result.findings[0].path, "specs/042-demo/spec.md");
    }

    #[test]
    fn done_spec_with_blocking_review_yields_blocking_finding() {
        let tmp = tempdir().unwrap();
        write(
            tmp.path(),
            "specs/042-demo/spec.md",
            &spec(
                "done",
                Some("  last-run: 2026-07-01T00:00:00Z\n  blocking: true\n  must-violations: 2"),
            ),
        );
        write(tmp.path(), "specs/042-demo/plan.md", "# Plan\n");
        write(tmp.path(), "specs/042-demo/tasks.md", GOOD_TASKS);
        let result = run(&args(), tmp.path()).unwrap();
        assert_eq!(families(&result), vec![("review-state-drift", "blocking")]);
        assert!(
            result.findings[0]
                .message
                .contains("unresolved MUST violations")
        );
    }

    #[test]
    fn done_spec_without_review_block_is_grandfathered() {
        let tmp = tempdir().unwrap();
        write(tmp.path(), "specs/042-demo/spec.md", &spec("done", None));
        write(tmp.path(), "specs/042-demo/plan.md", "# Plan\n");
        write(tmp.path(), "specs/042-demo/tasks.md", GOOD_TASKS);
        let result = run(&args(), tmp.path()).unwrap();
        assert!(result.clean, "{:?}", result.findings);
    }

    #[test]
    fn non_done_spec_with_empty_review_block_is_exempt() {
        let tmp = tempdir().unwrap();
        write(
            tmp.path(),
            "specs/042-demo/spec.md",
            &spec("in-progress", Some("  blocking: false")),
        );
        write(tmp.path(), "specs/042-demo/plan.md", "# Plan\n");
        write(tmp.path(), "specs/042-demo/tasks.md", GOOD_TASKS);
        let result = run(&args(), tmp.path()).unwrap();
        assert!(result.clean, "{:?}", result.findings);
    }

    // --- plumbing --------------------------------------------------------------------

    #[test]
    fn missing_feature_errors() {
        let tmp = tempdir().unwrap();
        let err = run(&args(), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::FeatureNotFound { .. }));
    }

    #[test]
    fn multiple_families_report_in_declared_order() {
        let tmp = tempdir().unwrap();
        write(
            tmp.path(),
            "specs/042-demo/spec.md",
            &spec("done", Some("  blocking: true")),
        );
        // done + no plan.md/tasks.md + review drift (last-run unset AND
        // blocking true) → completeness ×2, then review drift ×2. The
        // scenario family is skipped at done.
        write(
            tmp.path(),
            "specs/042-demo/scenarios/some-scenario.md",
            "---\nsection: \"X\"\n---\n\n# X\n",
        );
        let result = run(&args(), tmp.path()).unwrap();
        assert_eq!(
            families(&result),
            vec![
                ("artifact-completeness", "blocking"),
                ("artifact-completeness", "blocking"),
                ("review-state-drift", "blocking"),
                ("review-state-drift", "blocking"),
            ]
        );
        assert!(!result.clean);
    }
}
