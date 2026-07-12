//! Per-primitive args + result shapes.
//!
//! Mirrors the canonical JSON shapes in
//! `specs/022-deterministic-runtime/data-model.md`. Each primitive has an
//! `…Args` struct (also the `clap`-derive shape for the CLI surface) and a
//! `…Result` struct. JSON field names are kebab-case across the surface.

#![allow(clippy::module_name_repetitions, clippy::struct_excessive_bools)]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// -- read-spec ---------------------------------------------------------------

/// Args for `read-spec`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct ReadSpecArgs {
    /// Feature directory name under `specs/`.
    #[arg(long)]
    pub feature: String,
    /// Whether to populate `sections[].body`.
    #[serde(default)]
    #[arg(long)]
    pub include_body: bool,
}

/// Frontmatter review block (initial-release fields).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ReviewBlock {
    /// ISO-8601 UTC timestamp of the last `/gov:review`, if any.
    #[serde(default)]
    pub last_run: Option<String>,
    /// Constitution sha the review was run against.
    #[serde(default)]
    pub reviewed_against: Option<String>,
    /// MUST violations from the last review.
    #[serde(default)]
    pub must_violations: u32,
    /// SHOULD violations from the last review.
    #[serde(default)]
    pub should_violations: u32,
    /// Low-confidence findings from the last review.
    #[serde(default)]
    pub low_confidence: u32,
    /// Whether the last review left blocking findings.
    #[serde(default)]
    pub blocking: bool,
}

// -- discover-rule-files -----------------------------------------------------

/// Args for `discover-rule-files`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct DiscoverRuleFilesArgs {
    /// Surfaces detected by the host's stack detection, consulted ONLY when
    /// `.govern.toml` `[rules] surfaces` is unset. Members are `backend`
    /// and/or `frontend`. When the config key is set it wins; when both are
    /// absent, every recognized surface is loaded.
    #[serde(default)]
    #[arg(long = "detected-surface")]
    pub detected_surfaces: Vec<String>,
}

/// Result for `discover-rule-files`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct DiscoverRuleFilesResult {
    /// Repo-relative rule-file directory that was listed (`framework/rules`
    /// in govern's own repo, `{specs-root}/rules` in adopters). Empty when
    /// neither exists.
    pub rules_dir: String,
    /// Selected rule-file basenames, sorted, after surface selection and the
    /// disabled-rule-files filter.
    pub selected: Vec<String>,
    /// Ordered stdout notice lines to emit verbatim: unrecognized-suffix
    /// warnings, then disabled-rule-file notices, then the closing
    /// `loading rule files: …` line.
    pub notices: Vec<String>,
}

// -- process-waivers ---------------------------------------------------------

/// A currently-firing `(rule, file)` finding — input to `process-waivers`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct FiredFinding {
    /// Rule ID that fires.
    pub rule: String,
    /// Repo-relative file path where it fires.
    pub file: String,
}

/// A resolved waiver reference in a `process-waivers` result.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct WaiverRef {
    /// Waived rule ID.
    pub rule: String,
    /// Anchored file path.
    pub file: String,
    /// Operator-supplied justification.
    pub reason: String,
}

/// Args for `process-waivers`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct ProcessWaiversArgs {
    /// Feature directory name whose `spec.md` carries `review.waivers`.
    #[arg(long)]
    pub feature: String,
    /// Currently-firing `(rule, file)` findings from the review passes.
    /// Supplied via MCP/interpreter JSON; not a CLI flag.
    #[serde(default)]
    #[arg(skip)]
    pub fired: Vec<FiredFinding>,
}

/// Result for `process-waivers`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ProcessWaiversResult {
    /// Waivers that apply this run (anchor exists and the rule still fires).
    pub applied: Vec<WaiverRef>,
    /// Waivers that expired this run (anchor gone or rule no longer fires);
    /// `write-review` drops these on the next frontmatter write.
    pub expired: Vec<WaiverRef>,
    /// Ordered notice lines: `waiver expired: …`, `malformed waiver …`, and
    /// `duplicate waiver: …`, in entry order.
    pub notices: Vec<String>,
}

// -- compute-review-scope ----------------------------------------------------

/// Args for `compute-review-scope`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct ComputeReviewScopeArgs {
    /// Feature directory name whose review scope is computed.
    #[arg(long)]
    pub feature: String,
    /// Optional diff-base override (a git ref or sha). When omitted, the
    /// commit the spec advanced to `in-progress` at is used.
    #[serde(default)]
    #[arg(long)]
    pub since: Option<String>,
}

/// Result for `compute-review-scope`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ComputeReviewScopeResult {
    /// Resolved diff-base sha (empty when the spec never reached `in-progress`
    /// and no `--since` was given).
    pub diff_base: String,
    /// The review scope: the larger of `plan-affected` and `modified-since`.
    pub scope: Vec<String>,
    /// Files changed between `diff-base` and HEAD, sorted.
    pub modified_since: Vec<String>,
    /// Files listed under the plan's `## Affected Files` section.
    pub plan_affected: Vec<String>,
    /// Lines added to `{specs-root}/inbox.md` in the `diff-base..HEAD` window.
    pub captured_issues: Vec<String>,
}

// -- write-review ------------------------------------------------------------

/// One review finding — the record shape a `performReview` pass returns and
/// `write-review` consumes. `rule` / `severity` / `file` / `line-range` /
/// `confidence` are the extension-point contract; the render extras
/// (`summary` / `finding` / `rule-text` / `auto-fixable` / `suggested-fix`)
/// populate the per-finding block in `review.md` and default to empty so a
/// minimal finding still deserializes.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ReviewFinding {
    /// Rule ID (e.g., "SEC-BE-014").
    pub rule: String,
    /// Severity tier: `must` or `should`.
    pub severity: String,
    /// Repo-relative file path the finding anchors to.
    pub file: String,
    /// Line range within the file (e.g., "42-55" or "42"); empty means the
    /// whole file (overlaps any range with the same rule + file for dedup).
    #[serde(default)]
    pub line_range: String,
    /// Confidence tier: `high` or `low`. A `low` finding lands in the
    /// Low-confidence section regardless of severity.
    #[serde(default)]
    pub confidence: String,
    /// One-line finding summary (the `### … — <summary>` heading tail).
    #[serde(default)]
    pub summary: String,
    /// One-to-three-sentence explanation.
    #[serde(default)]
    pub finding: String,
    /// Verbatim rule text quoted from the rule file.
    #[serde(default)]
    pub rule_text: String,
    /// Whether a mechanical auto-fix exists.
    #[serde(default)]
    pub auto_fixable: bool,
    /// Suggested fix (code block or prose); omitted from the render when empty.
    #[serde(default)]
    pub suggested_fix: String,
}

/// Args for `write-review`. Findings cross the runtime boundary as a single
/// `findings` array (the content-ingestion convention), never as several
/// large per-section prose params.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct WriteReviewArgs {
    /// Feature directory name whose `review.md` is written.
    #[arg(long)]
    pub feature: String,
    /// ISO-8601 UTC timestamp recorded as `reviewed-at` / `review.last-run`.
    #[arg(long)]
    pub reviewed_at: String,
    /// HEAD sha the review ran against (`reviewed-against`).
    #[arg(long)]
    pub reviewed_against: String,
    /// diff-base sha from `compute-review-scope` (recorded in the report).
    #[arg(long)]
    pub diff_base: String,
    /// Scenario slug, when the run was scenario-targeted.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub scenario: Option<String>,
    /// When true, render the "nothing to review yet" empty-scope report.
    #[serde(default)]
    #[arg(long)]
    pub empty_scope: bool,
    /// Optional Summary override; a deterministic count line is generated when
    /// absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub summary: Option<String>,
    /// Dimensions skipped this run (via `--security` / `--simplicity` / …);
    /// echoed to `skipped-passes` and omitted from the counts.
    #[serde(default)]
    #[arg(long = "skipped-pass")]
    pub skipped_passes: Vec<String>,
    /// Pass findings as a single array (the content-ingestion convention).
    /// Supplied via MCP/interpreter JSON; not a CLI flag.
    #[serde(default)]
    #[arg(skip)]
    pub findings: Vec<ReviewFinding>,
    /// Applied waivers from `process-waivers`; matching findings are excluded
    /// from the counts and listed under Waived findings. The `alias` reads
    /// `process-waivers`' `applied` result key directly, so the waiver set
    /// threads through the exec walker's context (which merges primitive
    /// results by their bare key) as well as the MCP/host path.
    #[serde(default, alias = "applied")]
    #[arg(skip)]
    pub applied_waivers: Vec<WaiverRef>,
    /// Expired waivers from `process-waivers`; dropped from the spec
    /// frontmatter `review.waivers` list on this write. The `alias` reads
    /// `process-waivers`' `expired` result key (see `applied_waivers`).
    #[serde(default, alias = "expired")]
    #[arg(skip)]
    pub expired_waivers: Vec<WaiverRef>,
    /// Inbox additions in the review window from `compute-review-scope`;
    /// listed under Captured issues (informational).
    #[serde(default)]
    #[arg(skip)]
    pub captured_issues: Vec<String>,
}

/// Result for `write-review`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct WriteReviewResult {
    /// Repo-relative path of the `review.md` written.
    pub path: String,
    /// Repo-relative path of the spec file whose `review:` block was updated.
    pub spec_path: String,
    /// MUST violations counted (waived findings excluded).
    pub must_violations: u32,
    /// SHOULD violations counted (waived findings excluded).
    pub should_violations: u32,
    /// Low-confidence findings counted.
    pub low_confidence: u32,
    /// Findings excluded by an applied waiver.
    pub waived: u32,
    /// `true` when `must-violations` exceeds zero.
    pub blocking: bool,
    /// Derived exit code: 1 when blocking, else 0.
    pub exit_code: i32,
}

/// Parsed spec frontmatter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct Frontmatter {
    /// Pipeline status (e.g., "clarified", "planned", "in-progress", "done").
    pub status: String,
    /// Dependency feature names.
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Topic tags (e.g., `[format, process, pipeline]`).
    #[serde(default)]
    pub tags: Vec<String>,
    /// Last-review block, when set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub review: Option<ReviewBlock>,
}

/// One parsed body section.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct SpecSection {
    /// Section heading.
    pub heading: String,
    /// Markdown heading level (2 for `##`, etc.).
    pub level: u8,
    /// Section body text (empty unless `include-body` was set).
    pub body: String,
}

/// One acceptance-criterion checkbox.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct AcceptanceCriterion {
    /// Whether the checkbox is checked.
    pub checked: bool,
    /// Criterion text.
    pub text: String,
}

/// One open-question entry.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct OpenQuestion {
    /// Question text.
    pub text: String,
}

/// Result for `read-spec`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ReadSpecResult {
    /// Parsed frontmatter.
    pub frontmatter: Frontmatter,
    /// Body sections in document order.
    pub sections: Vec<SpecSection>,
    /// Acceptance-criteria checkboxes.
    pub acceptance_criteria: Vec<AcceptanceCriterion>,
    /// Open Questions list.
    pub open_questions: Vec<OpenQuestion>,
    /// Repo-relative path to the spec file.
    pub path: String,
}

// -- read-tasks --------------------------------------------------------------

/// Args for `read-tasks`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct ReadTasksArgs {
    /// Feature directory name under `specs/`.
    #[arg(long)]
    pub feature: String,
}

/// One sub-item under a top-level task.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct Subtask {
    /// Sub-item text.
    pub text: String,
    /// Whether the checkbox is checked.
    pub checked: bool,
}

/// One top-level task.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct Task {
    /// Top-level task number (e.g., "1", "12").
    pub number: String,
    /// Task heading text.
    pub heading: String,
    /// Subtask list.
    pub subtasks: Vec<Subtask>,
    /// `Done when:` clause, if present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub done_when: Option<String>,
    /// Phase container heading text, when the task lives under a `## …`
    /// phase (e.g., `Phase A — Refactor`). `None` for flat-structure tasks
    /// declared directly at level 2 (`## N. Title`). Absent from the JSON
    /// output when `None`, so existing consumers that don't know about
    /// phased structure still parse correctly.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
}

/// Result for `read-tasks`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ReadTasksResult {
    /// Tasks in declaration order.
    pub tasks: Vec<Task>,
    /// Repo-relative path to the tasks file.
    pub path: String,
}

// -- mark-task ---------------------------------------------------------------

/// Args for `mark-task`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct MarkTaskArgs {
    /// Feature directory name.
    #[arg(long)]
    pub feature: String,
    /// Top-level task number (e.g., "1").
    #[arg(long)]
    pub task_number: String,
    /// Subtask index within the task (0-based).
    #[arg(long)]
    pub subtask_index: usize,
    /// Desired checkbox state.
    #[arg(long)]
    pub checked: bool,
}

/// Result shape shared by `mark-task` and `mark-criterion`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct CheckboxToggleResult {
    /// Previous checkbox state.
    pub previous: bool,
    /// New checkbox state after the write.
    pub current: bool,
    /// Repo-relative path to the file written.
    pub path: String,
}

// -- mark-criterion ----------------------------------------------------------

/// Args for `mark-criterion`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct MarkCriterionArgs {
    /// Feature directory name.
    #[arg(long)]
    pub feature: String,
    /// Acceptance criterion index (0-based, ordered as in the spec).
    #[arg(long)]
    pub criterion_index: usize,
    /// Desired checkbox state.
    #[arg(long)]
    pub checked: bool,
}

// -- set-status --------------------------------------------------------------

/// Args for `set-status`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct SetStatusArgs {
    /// Feature directory name.
    #[arg(long)]
    pub feature: String,
    /// Expected current status on disk.
    #[arg(long)]
    pub from: String,
    /// Desired status to write.
    #[arg(long)]
    pub to: String,
}

/// Result for `set-status`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct SetStatusResult {
    /// Previous status field value.
    pub previous: String,
    /// New status after the write.
    pub current: String,
    /// Repo-relative path to the spec file.
    pub path: String,
}

// -- derive-boundary ---------------------------------------------------------

/// Args for `derive-boundary`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct DeriveBoundaryArgs {
    /// Feature directory name.
    #[arg(long)]
    pub feature: String,
}

/// Result for `derive-boundary`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct DeriveBoundaryResult {
    /// Boundary entries (glob patterns and concrete paths).
    pub boundary: Vec<String>,
    /// First commit that touched the spec dir.
    pub first_commit: String,
    /// Current `HEAD` sha at derivation time.
    pub current_head: String,
}

// -- check-stuck -------------------------------------------------------------

/// Args for `check-stuck`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct CheckStuckArgs {
    /// Feature directory name.
    #[arg(long)]
    pub feature: String,
    /// Commit-count threshold above which the task is considered stuck.
    #[arg(long)]
    pub threshold: u32,
}

/// Result for `check-stuck`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct CheckStuckResult {
    /// Number of commits on `tasks.md` since `since-sha`.
    pub commit_count: u32,
    /// Whether `commit-count >= threshold` and the same task is still
    /// incomplete.
    pub stuck: bool,
    /// Sha at which the status entered `in-progress` (origin of the count).
    pub since_sha: String,
    /// Threshold echoed from args.
    pub threshold: u32,
}

// -- validate-frontmatter ---------------------------------------------------

/// One frontmatter validation finding.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct FrontmatterFinding {
    /// Severity tier.
    pub severity: String,
    /// Field path that failed validation (may be empty for cross-field issues).
    pub field: String,
    /// Human-readable description.
    pub message: String,
}

/// Args for `validate-frontmatter`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct ValidateFrontmatterArgs {
    /// Repo-relative path to the spec file.
    #[arg(long)]
    pub path: String,
}

/// Result for `validate-frontmatter`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ValidateFrontmatterResult {
    /// All findings collected (empty when `clean`).
    pub findings: Vec<FrontmatterFinding>,
    /// Whether the frontmatter is clean.
    pub clean: bool,
}

// -- resolve-anchor ----------------------------------------------------------

/// Args for `resolve-anchor`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct ResolveAnchorArgs {
    /// Repo-relative path to the markdown file to scan.
    #[arg(long)]
    pub path: String,
}

/// One anchor reference.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct AnchorReference {
    /// Anchor name (without `§` prefix).
    pub anchor: String,
    /// 1-based line of the reference.
    pub line: u32,
    /// Whether the anchor resolves to a marker.
    pub resolved: bool,
}

/// Result for `resolve-anchor`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ResolveAnchorResult {
    /// All anchor references found in the file.
    pub references: Vec<AnchorReference>,
    /// Anchor names with no matching marker.
    pub unresolved: Vec<String>,
}

// -- traverse-deps -----------------------------------------------------------

/// Args for `traverse-deps`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct TraverseDepsArgs {
    /// Feature directory name.
    #[arg(long)]
    pub feature: String,
}

/// One dependency edge.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct DependencyEdge {
    /// Dependency feature name.
    pub feature: String,
    /// Whether the dependency directory exists.
    pub exists: bool,
    /// Status of the dependency (empty when `exists` is false).
    #[serde(default)]
    pub status: String,
    /// Whether the dependency status is compatible with this feature.
    pub compatible: bool,
}

/// Result for `traverse-deps`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct TraverseDepsResult {
    /// All dependency edges.
    pub dependencies: Vec<DependencyEdge>,
    /// Overall compatibility (logical AND across direct edges, plus
    /// `cycles.is_empty()`).
    pub compatible: bool,
    /// Strongly-connected components forming cycles in the reachable dep
    /// subgraph rooted at the targeted feature. Each entry is one SCC as
    /// a list of slugs in traversal order — multi-node cycles (size ≥ 2)
    /// and self-cycles (size 1 with a self-edge) both surface here.
    /// Empty when the walked subgraph is acyclic.
    #[serde(default)]
    pub cycles: Vec<Vec<String>>,
}

// -- dashboard ---------------------------------------------------------------

/// Args for `dashboard`. The primitive takes no caller-supplied inputs —
/// the repo root, `.govern.toml` (committed config), and
/// `.govern.session.toml` (gitignored per-user session state) are the only
/// state it reads. The empty args struct preserves clap-derive consistency
/// with every other primitive.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct DashboardArgs {}

/// Per-spec entry in the dashboard payload. The fields mirror the dashboard
/// table 1:1 — `slug` / `status` / `dependencies` / `tags` / `open-question-count`
/// drive the row's identity and labels; `has-plan` / `has-tasks` /
/// `has-data-model` / `scenarios-count` populate the artifact-existence
/// columns; `blocked-by` carries the deterministically-computed list of
/// dependency slugs whose own `status` is below `clarified` (empty when
/// unblocked).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct DashboardSpec {
    /// Directory basename (e.g., "022-deterministic-runtime").
    pub slug: String,
    /// Frontmatter status (one of `draft`, `clarified`, `planned`,
    /// `in-progress`, `done`).
    pub status: String,
    /// Frontmatter `dependencies` array (empty when absent).
    pub dependencies: Vec<String>,
    /// Frontmatter `tags` array (empty when absent).
    pub tags: Vec<String>,
    /// Count of unresolved questions in the spec body's `## Open Questions`
    /// section, matching `read-spec`'s open-question semantics.
    pub open_question_count: u32,
    /// `true` when `specs/{slug}/plan.md` exists on disk.
    pub has_plan: bool,
    /// `true` when `specs/{slug}/tasks.md` exists on disk.
    pub has_tasks: bool,
    /// `true` when `specs/{slug}/data-model.md` exists on disk.
    pub has_data_model: bool,
    /// Count of `*.md` files under `specs/{slug}/scenarios/` (0 when the
    /// directory is absent).
    pub scenarios_count: u32,
    /// Dependency slugs whose own `status` is below `clarified`; empty when
    /// every dependency is at `clarified` or later. The caller renders the
    /// "blocked specs" callout straight from a non-empty array.
    pub blocked_by: Vec<String>,
}

/// `.govern.toml` review-state summary returned alongside the per-spec
/// inventory. The `present` flag distinguishes "config absent" from
/// "config present but section absent / empty" so callers can drive the
/// callout-suppression rule correctly.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct DashboardConfig {
    /// `true` when `.govern.toml` exists at the repo root.
    pub present: bool,
    /// Basenames from `[[review.disabled-rule-files]]`. Empty when the
    /// section is absent or its array is empty.
    pub disabled_rule_files: Vec<String>,
}

/// Scenario-level detail returned when the session target names a scenario.
/// Populated so callers render the scenario header line without a separate
/// file read.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct DashboardScenarioDetail {
    /// Scenario frontmatter `section` field (or the legacy `spec-ref` field
    /// for pre-017 scenarios). Empty when neither is present.
    pub section: String,
    /// One-line summary of the scenario's `## Context` section (first
    /// non-blank line, trimmed). Empty when the section is absent.
    pub context_summary: String,
    /// Count of unresolved questions in the scenario body's
    /// `## Open Questions` section.
    pub open_question_count: u32,
}

/// Session-target summary returned when the repo-root `.govern.session.toml`
/// exists and names a target. The `feature` field always names the targeted
/// feature; `scenario` is populated when a scenario is targeted;
/// `scenario-detail` is populated alongside `scenario` to spare callers an
/// extra read.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct DashboardSessionTarget {
    /// Targeted feature slug as recorded in the session file.
    pub feature: String,
    /// Targeted scenario slug, when one is set.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario: Option<String>,
    /// Scenario header detail; present when `scenario` is `Some` and the
    /// scenario file is readable.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario_detail: Option<DashboardScenarioDetail>,
}

/// Result for `dashboard`. One call returns everything `/gov:status` needs
/// to render the full pipeline view: the per-spec inventory, the
/// repo-wide `tags-union`, the `.govern.toml` review-state summary, and
/// the optional session target.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct DashboardResult {
    /// Session target when the repo-root `.govern.session.toml` exists and
    /// names a target; `None` otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub session_target: Option<DashboardSessionTarget>,
    /// Per-spec entries in directory-name order.
    pub specs: Vec<DashboardSpec>,
    /// Sorted, deduplicated union of every spec's `tags` array. Empty when
    /// no spec has tags.
    pub tags_union: Vec<String>,
    /// `.govern.toml` review-state summary.
    pub config: DashboardConfig,
}

// -- check-rule-ids ----------------------------------------------------------

/// Args for `check-rule-ids`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct CheckRuleIdsArgs {
    /// Repo-relative path to the file scanned for citations.
    #[arg(long)]
    pub path: String,
    /// Repo-relative paths to rule files defining the known rule IDs.
    #[arg(long = "rule-file")]
    pub rule_files: Vec<String>,
}

/// One rule-ID citation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct RuleCitation {
    /// Rule ID as cited (e.g., "SEC-AUTH-001").
    pub rule_id: String,
    /// Whether the ID exists in any rule file.
    pub found: bool,
    /// Whether the ID is deprecated.
    pub deprecated: bool,
}

/// Result for `check-rule-ids`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct CheckRuleIdsResult {
    /// All citations parsed from the file.
    pub citations: Vec<RuleCitation>,
    /// Cited rule IDs that don't exist.
    pub missing: Vec<String>,
    /// Cited rule IDs that exist but are deprecated.
    pub deprecated: Vec<String>,
}

// -- run-generator -----------------------------------------------------------

/// Args for `run-generator`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct RunGeneratorArgs {
    /// Repo-relative path to the bash script.
    #[arg(long)]
    pub script: String,
}

/// Result for `run-generator`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct RunGeneratorResult {
    /// Whether the script reported drift (non-zero exit treated as drift).
    pub drift: bool,
    /// Captured stdout.
    pub stdout: String,
    /// Captured stderr.
    pub stderr: String,
    /// Script's exit code.
    pub exit_code: i32,
}

// -- lint-markdown -----------------------------------------------------------

/// Args for `lint-markdown`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct LintMarkdownArgs {
    /// Paths or globs to lint.
    #[arg(long = "path")]
    pub paths: Vec<String>,
    /// Whether to invoke `markdownlint-cli2` in fix mode.
    #[serde(default)]
    #[arg(long)]
    pub fix: bool,
}

/// One markdown-lint violation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct MarkdownViolation {
    /// Repo-relative file path.
    pub path: String,
    /// 1-based line.
    pub line: u32,
    /// `markdownlint` rule name (e.g., "MD013").
    pub rule: String,
    /// Description of the violation.
    pub message: String,
}

/// Result for `lint-markdown`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct LintMarkdownResult {
    /// All violations.
    pub violations: Vec<MarkdownViolation>,
    /// Whether the lint produced no violations and exited zero.
    pub clean: bool,
    /// `markdownlint-cli2` exit code.
    pub exit_code: i32,
}

// -- merge-claude-md ---------------------------------------------------------

/// Args for `merge-claude-md`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct MergeClaudeMdArgs {
    /// Local path to the adopter's `CLAUDE.md` (relative paths resolve
    /// against the runtime's `repo`).
    #[arg(long)]
    pub path: String,
    /// Markdown block the framework wants to install (between the BEGIN /
    /// END marker pair). Trailing whitespace is normalized to a single
    /// newline before write.
    #[arg(long)]
    pub block: String,
    /// Marker name used to delimit the framework-managed region.
    /// Defaults to `govern-managed`. Multiple frameworks can coexist in
    /// the same CLAUDE.md by using different marker names.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub marker: Option<String>,
}

/// Result for `merge-claude-md`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct MergeClaudeMdResult {
    /// Repo-relative or absolute path of the merged file.
    pub path: String,
    /// One of `created`, `inserted`, `updated`, `unchanged`.
    pub action: String,
    /// Marker name actually applied (echoes the arg's value or the default).
    pub marker: String,
}

// -- apply-manifest ----------------------------------------------------------

/// One entry in an `apply-manifest` request.
///
/// `source` is a path relative to the args' `source-root`; `dest` is a
/// path relative to the args' `target-root`. Both use forward slashes;
/// the primitive normalizes to the host OS when joining.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ManifestEntry {
    /// Path under `source-root` to read.
    pub source: String,
    /// Path under `target-root` to write.
    pub dest: String,
    /// Per-entry strategy: `update` / `create` / `skip-if-conflict`.
    pub strategy: String,
    /// Substitution keys (without braces) to exclude for this entry only.
    /// Unlisted keys are substituted normally; unknown keys are no-ops.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keep_literals: Option<Vec<String>>,
}

/// Args for `apply-manifest`.
///
/// Only `source-root` and `target-root` are exposed as CLI flags; `entries`,
/// `pinned`, and `substitutions` are set via the JSON context (the CLI surface
/// of this primitive is a debug entry point, not the production path).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct ApplyManifestArgs {
    /// Local path to the source tree (typically a prior `extract-archive`
    /// staging directory).
    #[arg(long)]
    pub source_root: String,
    /// Local path to the destination tree; created on demand for each entry.
    #[arg(long)]
    pub target_root: String,
    /// Per-entry manifest. Set via JSON context — not exposed as CLI flags.
    #[serde(default)]
    #[arg(skip)]
    pub entries: Vec<ManifestEntry>,
    /// Destination paths the primitive must never touch, regardless of strategy.
    /// Forward-slash form, relative to `target-root`. Set via JSON context.
    #[serde(default)]
    #[arg(skip)]
    pub pinned: Vec<String>,
    /// `{key}` → value substitution map applied to text files. Per-entry
    /// `keep-literals` masks specific keys for individual entries. Set via
    /// JSON context.
    #[serde(default)]
    #[arg(skip)]
    pub substitutions: std::collections::BTreeMap<String, String>,
}

/// One per-entry outcome from `apply-manifest`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ManifestEntryResult {
    /// Echo of the entry's `source` field.
    pub source: String,
    /// Echo of the entry's `dest` field.
    pub dest: String,
    /// One of `created` / `updated` / `unchanged` / `skipped-exists` /
    /// `skipped-pinned` / `source-missing`.
    pub action: String,
}

/// Result for `apply-manifest`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ApplyManifestResult {
    /// Per-entry outcomes in declaration order.
    pub entries: Vec<ManifestEntryResult>,
    /// Count of `created` actions across all entries.
    pub created: u32,
    /// Count of `updated` actions across all entries.
    pub updated: u32,
    /// Count of `unchanged` actions across all entries.
    pub unchanged: u32,
    /// Count of `skipped-exists` actions across all entries.
    pub skipped_exists: u32,
    /// Count of `skipped-pinned` actions across all entries.
    pub skipped_pinned: u32,
    /// Count of `source-missing` actions across all entries.
    pub source_missing: u32,
}

// -- enforce-manifest --------------------------------------------------------

/// Args for `enforce-manifest`.
///
/// Walks `directory`, removes files matching `glob-include` that are not
/// in `expected` and not in `pinned`, and returns the per-file outcome.
/// The primitive does not create `directory` when missing — that's
/// `apply-manifest`'s job.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct EnforceManifestArgs {
    /// Local path to the directory to enforce.
    #[arg(long)]
    pub directory: String,
    /// Files relative to `directory` that must remain (basenames for
    /// top-level, slash-delimited relative paths for recursive). Set via
    /// JSON context.
    #[serde(default)]
    #[arg(skip)]
    pub expected: Vec<String>,
    /// Files relative to `directory` that must remain regardless of
    /// `expected`. Reported under `pinned-kept` so callers can surface
    /// the count in completion messages. Set via JSON context.
    #[serde(default)]
    #[arg(skip)]
    pub pinned: Vec<String>,
    /// When `true`, walk subdirectories recursively. Default `false` —
    /// the bootstrap's slash-command cleanup is top-level only.
    #[serde(default)]
    #[arg(long)]
    pub recursive: bool,
    /// Glob applied to each file's basename. Default `*.md`. Files whose
    /// basename does not match the glob are left untouched (not even
    /// considered for removal).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub glob_include: Option<String>,
}

/// Result for `enforce-manifest`.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct EnforceManifestResult {
    /// Forward-slash relative paths of files removed during the walk.
    pub removed: Vec<String>,
    /// Forward-slash relative paths of files kept because they were in
    /// `expected`.
    pub kept: Vec<String>,
    /// Forward-slash relative paths of files kept because they were in
    /// `pinned`.
    pub pinned_kept: Vec<String>,
}

// -- merge-managed-block -----------------------------------------------------

/// Args for `merge-managed-block`.
///
/// Generalization of [`MergeClaudeMdArgs`] that handles configurable
/// marker shapes. `marker-style: "html-comment"` (default) reproduces
/// `merge-claude-md`'s exact behavior; `marker-style: "line-prefix"`
/// uses a single `# {marker}` preamble line followed by the block,
/// terminated by a blank line or EOF — matching `.gitignore` and
/// `.gitattributes` conventions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct MergeManagedBlockArgs {
    /// Local path to the file to merge into (relative paths resolve
    /// against the runtime's `repo`).
    #[arg(long)]
    pub path: String,
    /// Markdown / plain-text block the framework wants to install.
    /// Trailing whitespace is normalized to a single newline before
    /// write.
    #[arg(long)]
    pub block: String,
    /// Marker name used to delimit the framework-managed region.
    /// Defaults to `govern-managed`. Multiple frameworks can coexist in
    /// the same file by using different marker names.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub marker: Option<String>,
    /// One of `html-comment` (default) or `line-prefix`. The former
    /// uses `<!-- BEGIN/END {marker} -->` pairs; the latter uses a
    /// single `# {marker}` preamble line followed by the block,
    /// terminated by a blank line or EOF.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub marker_style: Option<String>,
}

/// Result for `merge-managed-block`. Extends [`MergeClaudeMdResult`]'s
/// shape with two `line-prefix`-only fields for the cross-boundary
/// dedup pass (`dedup-removed` count, `dedup-removed-lines` listing).
/// Both fields are absent for `html-comment` invocations (the
/// `merge-claude-md` compat shim ends up here too).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct MergeManagedBlockResult {
    /// Repo-relative or absolute path of the merged file.
    pub path: String,
    /// One of `created`, `inserted`, `updated`, `unchanged`.
    pub action: String,
    /// Marker name actually applied (echoes the arg's value or the default).
    pub marker: String,
    /// Marker style actually applied (echoes the arg's value or the default).
    pub marker_style: String,
    /// Count of adopter-area lines removed by the cross-boundary dedup
    /// pass. `Some(n)` only on `line-prefix` invocations; `None` for
    /// `html-comment` callsites (the dedup contract is line-list-shaped
    /// and doesn't apply to prose blocks).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dedup_removed: Option<u32>,
    /// Verbatim content of the adopter-area lines removed by the
    /// cross-boundary dedup pass, in source order. `Some(vec)` only on
    /// `line-prefix` invocations; `None` for `html-comment` callsites.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dedup_removed_lines: Option<Vec<String>>,
}

// -- merge-permissions -------------------------------------------------------

/// Args for `merge-permissions` — idempotently merge a canonical
/// permission allow/deny set into a JSON file, removing exact-match
/// duplicates from each array. The primitive is the deterministic surface
/// `/configure` calls; see spec 022's `framework-list-dedup` scenario for
/// the contract.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct MergePermissionsArgs {
    /// Repo-relative path of the JSON file to merge into (e.g.,
    /// `.claude/settings.local.json` on Claude Code,
    /// `.augment/settings.json` on Auggie). Host-supplied from the
    /// bootstrap-substituted `{cli-config-dir}/settings.local.json`
    /// template — no default, so a missing path fails loudly instead of
    /// silently writing to a Claude-shaped location on a non-Claude host.
    #[arg(long)]
    pub path: String,
    /// Canonical entries to ensure under `permissions.allow`.
    #[serde(default)]
    #[arg(long, value_delimiter = ',')]
    pub allow: Vec<String>,
    /// Canonical entries to ensure under `permissions.deny`.
    #[serde(default)]
    #[arg(long, value_delimiter = ',')]
    pub deny: Vec<String>,
}

/// Result for `merge-permissions`. Reports the action taken plus
/// per-array counts of entries added (canonical members that were
/// not present) vs. duplicates removed (exact-match entries that
/// were redundant).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct MergePermissionsResult {
    /// Repo-relative or absolute path of the merged file.
    pub path: String,
    /// One of `created`, `updated`, `unchanged`.
    pub action: String,
    /// Count of canonical `allow` entries appended (not already present).
    pub allow_added: u32,
    /// Count of duplicate `allow` entries removed.
    pub allow_deduped: u32,
    /// Count of canonical `deny` entries appended (not already present).
    pub deny_added: u32,
    /// Count of duplicate `deny` entries removed.
    pub deny_deduped: u32,
}

// -- substitute-templates ----------------------------------------------------

/// Args for `substitute-templates`.
///
/// The source/target fields use the `-dir` suffix (rather than the
/// shorter `source`/`dest`) so they don't collide with
/// [`ExtractArchiveArgs::dest`] when both primitives share a single
/// context map in a procedure walk (the bootstrap chains
/// extract → substitute and needs both primitives' destinations to
/// resolve to distinct context keys).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct SubstituteTemplatesArgs {
    /// Local path to the source tree (typically the staging directory a
    /// prior `extract-archive` step produced).
    #[arg(long)]
    pub source_dir: String,
    /// Local path to the destination tree; created if missing.
    #[arg(long)]
    pub target_dir: String,
    /// Key→value substitution map. Each text file in the source tree has
    /// every literal `{key}` replaced with `value` before being written
    /// to the destination. Binary files are copied unchanged. Set via
    /// JSON context — not exposed as CLI flags.
    #[serde(default)]
    #[arg(skip)]
    pub substitutions: std::collections::BTreeMap<String, String>,
}

/// Result for `substitute-templates`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct SubstituteTemplatesResult {
    /// Repo-relative or absolute path of the destination tree.
    pub target_dir: String,
    /// Count of regular files written to the destination.
    pub files_written: u32,
    /// Total count of substitution replacements applied across all files.
    pub substitutions_applied: u32,
    /// Repo-relative paths (under `target-dir`) of every file written,
    /// in directory-walk order.
    pub files: Vec<String>,
}

// -- extract-archive ---------------------------------------------------------

/// Args for `extract-archive`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct ExtractArchiveArgs {
    /// Local path to the archive (`.tar.gz`, `.tgz`, `.zip`).
    #[arg(long)]
    pub archive: String,
    /// Destination directory; created if missing.
    #[arg(long)]
    pub dest: String,
    /// Explicit format override (`tar-gz` / `zip`). Auto-detected from the
    /// archive's extension when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub format: Option<String>,
}

/// Result for `extract-archive`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ExtractArchiveResult {
    /// Repo-relative or absolute path of the destination directory.
    pub dest: String,
    /// Repo-relative paths of every regular file extracted, in archive order.
    pub files: Vec<String>,
    /// Count of regular files extracted (directories are not counted).
    pub count: u32,
    /// Detected or override format echoed back (`tar-gz` or `zip`).
    pub format: String,
}

// -- fetch-archive -----------------------------------------------------------

/// Args for `fetch-archive`.
///
/// The local destination uses the `archive` field name (not `dest`) so
/// it shares a context key with [`ExtractArchiveArgs::archive`] when both
/// primitives appear in the same procedure walk — fetch writes the
/// downloaded archive to that path; extract then reads it from the same
/// path without the host having to thread two keys.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct FetchArchiveArgs {
    /// URL of the archive (`.tar.gz`, `.zip`, etc.).
    #[arg(long)]
    pub url: String,
    /// URL of the sha256 sidecar file (matching the `shasum -a 256` format —
    /// one or more lines of `<hex>  <filename>`). **Optional**: when
    /// absent the primitive downloads without verifying but still
    /// returns the computed sha256 in the result, so callers can verify
    /// out-of-band against a known-good digest if desired.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub sha256_url: Option<String>,
    /// Local path where the downloaded archive is written. Used as the
    /// `archive` input by a subsequent `extract-archive` step in the
    /// bootstrap procedure.
    #[arg(long)]
    pub archive: String,
}

/// Result for `fetch-archive`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct FetchArchiveResult {
    /// Repo-relative or absolute path where the archive was written.
    pub path: String,
    /// Lowercase hex sha256 of the downloaded archive. When the args
    /// included `sha256_url`, this value also matched the sidecar's
    /// digest (verification succeeded). When the sidecar URL was
    /// absent, this is the computed digest only — the host can
    /// compare it against a known-good value out-of-band.
    pub sha256: String,
    /// Whether the sha256 was verified against a sidecar URL provided
    /// in the args. `false` when no sidecar URL was supplied.
    pub verified: bool,
    /// Size of the downloaded archive in bytes.
    pub bytes: u64,
}

// -- gate-confirm ------------------------------------------------------------

/// Args for `gate-confirm`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct GateConfirmArgs {
    /// Named gate (e.g., "plan-finalize-status").
    #[arg(long)]
    pub gate: String,
    /// Prompt shown to the user.
    #[arg(long)]
    pub prompt: String,
}

/// Result for `gate-confirm`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct GateConfirmResult {
    /// Whether the user confirmed.
    pub confirmed: bool,
}

// -- create-scenario ---------------------------------------------------------

/// Args for `create-scenario`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct CreateScenarioArgs {
    /// Repo-relative feature directory (e.g., `specs/042-foo`).
    #[arg(long)]
    pub feature_path: String,
    /// Scenario slug (no extension; the filename becomes `{slug}.md`).
    #[arg(long)]
    pub slug: String,
    /// Parent-spec section name written into the scenario frontmatter.
    #[arg(long)]
    pub section: String,
    /// Assembled scenario body — the `## Context` … `## Edge Cases` markdown
    /// the LLM authored, crossing the runtime boundary as one payload (the
    /// content-ingestion convention). The primitive frames it with the
    /// `section:` frontmatter, the H1-from-slug, and the auto-appended
    /// Open / Resolved Questions scaffolding.
    #[arg(long)]
    pub body: String,
}

/// Result for `create-scenario`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct CreateScenarioResult {
    /// Repo-relative path of the newly-created scenario file.
    pub created: String,
}

// -- append-task -------------------------------------------------------------

/// Args for `append-task`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct AppendTaskArgs {
    /// Repo-relative feature directory (e.g., `specs/042-foo`).
    #[arg(long)]
    pub feature_path: String,
    /// Task title (the text after the `## N. ` heading prefix).
    #[arg(long)]
    pub title: String,
    /// Body content for the task's `Done when:` clause.
    #[arg(long)]
    pub done_when: String,
    /// Optional checkbox sub-items to render inside the task block. When
    /// omitted, the primitive emits a single default
    /// `- [ ] Implement the behavior described in scenarios/{slug}.md`
    /// line using the explicit `slug` argument below.
    #[arg(long)]
    pub body: Option<Vec<String>>,
    /// Scenario slug used by the default-body line. Required when `body`
    /// is omitted (the default body needs a slug to fill
    /// `scenarios/{slug}.md`). Ignored when `body` is supplied — the
    /// caller has provided the full body, so no slug is needed. Pairs
    /// with the slug previously passed to `create-scenario` when both
    /// primitives are invoked together by the scenario branch of
    /// `/gov:amend`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub slug: Option<String>,
    /// Heading of an existing `## …` phase container under which the new
    /// task should be appended (e.g., `Phase B — Implementation`). Only
    /// consulted when the target `tasks.md` is phased — i.e., contains
    /// at least one `### N.` heading. In a flat file the argument is
    /// ignored and the task is appended at file bottom as `## N. …`.
    ///
    /// When phased and `parent-heading` is omitted, the primitive
    /// creates a default follow-on phase using the auto-computed letter:
    /// `## Phase {next-letter} — Follow-on scenarios`, where
    /// `{next-letter}` is the next alphabetical letter after existing
    /// `Phase X` labels (defaulting to `A` when none are present).
    ///
    /// When phased and the supplied heading does not match any existing
    /// phase, the primitive refuses with
    /// `PrimitiveError::ParentHeadingNotFound` rather than silently
    /// creating a new phase or appending at file bottom.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub parent_heading: Option<String>,
}

/// Result for `append-task`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct AppendTaskResult {
    /// Number assigned to the newly-appended task (`max(existing) + 1`).
    pub task_number: u32,
    /// Repo-relative path of the tasks file written.
    pub path: String,
    /// Whether `tasks.md` was created by this invocation. `false` when an
    /// existing file was extended.
    pub created: bool,
}

// -- prune-tasks -------------------------------------------------------------

/// Args for `prune-tasks`. Reduces the target feature's `tasks.md` by
/// dropping spent (fully-checked) task sections, or resetting the file to
/// its template initial state. See
/// `specs/041-task-pruning/data-model.md`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct PruneTasksArgs {
    /// Feature directory name under `specs/`.
    #[arg(long)]
    pub feature: String,
    /// Full reset to the template's initial state, rather than the default
    /// keep-pending prune.
    #[serde(default)]
    #[arg(long)]
    pub reset: bool,
    /// Override the `--reset` status gate on a non-`done` spec.
    #[serde(default)]
    #[arg(long)]
    pub force: bool,
    /// Write the reduced file. When false (the default) the primitive is a
    /// pure preview: it computes and classifies but does not write, and the
    /// file body never leaves the runtime.
    #[serde(default)]
    #[arg(long)]
    pub apply: bool,
}

/// Which reduction the primitive performed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PruneMode {
    /// Drop spent sections; keep every pending section.
    KeepPending,
    /// Reset to the template's initial state.
    Reset,
}

/// Completion state of a task section.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Classification {
    /// >= 1 checkbox and every one is checked. Removable.
    Spent,
    /// >= 1 checkbox and at least one is unchecked. Preserved.
    Pending,
    /// Zero checkboxes. Preserved; never classified spent.
    NoCheckbox,
}

/// Outcome of the `--reset` status gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PruneGate {
    /// Keep-pending mode — the gate does not apply.
    NotApplicable,
    /// Reset is permitted (status is `done`, or `force` was supplied).
    Allowed,
    /// Reset refused: status is not `done` and `force` was absent.
    BlockedNeedsForce,
}

/// What prune did with a task section.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PruneAction {
    /// The section was dropped from the output.
    Removed,
    /// The section was kept verbatim.
    Kept,
}

/// Line and byte size of a `tasks.md`, before or after pruning.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct SizeSummary {
    /// Line count.
    pub lines: usize,
    /// Byte count.
    pub bytes: usize,
}

/// One compact per-section record. Carries the section's identity,
/// classification, and checkbox counts — never its body.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct PruneSection {
    /// Task number (e.g., "1", "12").
    pub number: String,
    /// Task heading text.
    pub heading: String,
    /// Containing phase heading, when phased. Absent for flat structure.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub phase: Option<String>,
    /// Completion classification.
    pub classification: Classification,
    /// Task-list checkboxes in the section.
    pub checkbox_total: u32,
    /// Of which are checked.
    pub checkbox_checked: u32,
    /// What prune did with the section.
    pub action: PruneAction,
}

/// Result for `prune-tasks`. A compact summary; the file body is never
/// included — the token-reduction contract that motivates the primitive
/// performing its own write.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct PruneTasksResult {
    /// The reduction performed.
    pub mode: PruneMode,
    /// Whether a write happened. `false` on preview, on `nothing-to-prune`,
    /// and on a blocked reset.
    pub applied: bool,
    /// `--reset` status-gate outcome.
    pub gate: PruneGate,
    /// Spec status, read only when `reset` is set (otherwise `null`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Output equals input — nothing spent to prune (keep-pending), or
    /// already template-state (reset).
    pub nothing_to_prune: bool,
    /// Task sections removed.
    pub removed_count: u32,
    /// Task sections kept.
    pub kept_count: u32,
    /// Size before pruning.
    pub size_before: SizeSummary,
    /// Size the tasks file would have after pruning (reported even on a
    /// preview, which computes but does not write it); equal to
    /// `size_before` only on a no-op (nothing to prune).
    pub size_after: SizeSummary,
    /// Per-section classification records.
    pub sections: Vec<PruneSection>,
    /// Repo-relative path to the tasks file.
    pub path: String,
}

// -- migrate-session-file ----------------------------------------------------

/// Args for `migrate-session-file`. Translates a pre-0.10.0 legacy
/// session JSON at `legacy-path` into the consolidated
/// `<repo>/.govern.session.toml` and deletes the legacy file. The
/// destination is hardcoded (it's `write-session`'s `SESSION_FILE`
/// constant) so the migration cannot drift from the runtime's read
/// path.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct MigrateSessionFileArgs {
    /// Repo-relative path of the legacy session JSON, host-supplied
    /// from the bootstrap-substituted `{cli-config-dir}/{project}-session.json`
    /// template (e.g., `.claude/gov-session.json`,
    /// `.claude/anvil-session.json`, `.augment/anvil-session.json`).
    /// Validated as relative-and-no-`..`.
    #[arg(long)]
    pub legacy_path: String,
}

/// Result for `migrate-session-file`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct MigrateSessionFileResult {
    /// Repo-relative path of the legacy file the primitive operated on
    /// (echoes the input `legacy-path`).
    pub source: String,
    /// Repo-relative path of the consolidated session file. Always
    /// `.govern.session.toml` (the runtime's `write-session::SESSION_FILE`).
    pub dest: String,
    /// `"migrated"` — legacy file translated into a fresh
    /// `.govern.session.toml` and deleted.
    /// `"kept-existing"` — `.govern.session.toml` already existed; the
    /// new file was left untouched and the legacy file was deleted.
    /// `"no-legacy"` — no legacy file present at `legacy-path`; no-op.
    pub action: String,
    /// `true` when the legacy file was removed from disk; `false` only
    /// when `action == "no-legacy"`.
    pub legacy_deleted: bool,
}

// -- write-session -----------------------------------------------------------

/// Args for `write-session`. Sets the session state at the canonical
/// `<repo>/.govern.session.toml` location (gitignored, repo-root, no
/// host/project variability). The `scenario` and `scenario-path` fields
/// are paired — both must be supplied together or both omitted; omitting
/// both clears any previously set scenario.
///
/// Three write shapes, in precedence order:
///
/// 1. **Clear write** (`clear: true`) — removes the target block
///    (feature / path / scenario / scenario-path / set-at) while
///    preserving `cli-config-dir`. Mutually exclusive with every target
///    field; `cli-config-dir` may still be supplied and overrides the
///    preserved value.
/// 2. **Target write** (`feature` + `path`) — sets the target and a
///    fresh `set-at`, preserving `cli-config-dir`.
/// 3. **Host-config write** (only `cli-config-dir`) — sets the agent
///    identity, preserving the existing target.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct WriteSessionArgs {
    /// Feature slug (e.g., `022-deterministic-runtime`). Supplying it makes
    /// this a *target write* — feature, path, scenario, and a fresh `set-at`
    /// are written, preserving the per-contributor `cli-config-dir`. Omit it
    /// (supplying only `cli-config-dir`) for a *host-config write* that sets
    /// the agent identity while preserving the existing target. Must be
    /// supplied together with `path`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub feature: Option<String>,
    /// Repo-relative spec directory (e.g., `specs/022-deterministic-runtime`).
    /// The TOML key in the written session file is `path`, matching the
    /// convention used by `dashboard`'s reader and by host-written
    /// sessions in adopter repos pre-consolidation. Must be supplied
    /// together with `feature`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub path: Option<String>,
    /// Optional scenario slug. Must be supplied iff `scenario-path` is set,
    /// and only on a target write (with `feature`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub scenario: Option<String>,
    /// Optional repo-relative scenario file path. Must be supplied iff
    /// `scenario` is set. Stored as `scenario-path` (kebab-case) in the
    /// written session TOML.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub scenario_path: Option<String>,
    /// Optional per-contributor agent config-dir name (`.claude`, `.augment`,
    /// `.opencode`, `.agents`). Written to the gitignored session file by
    /// `/govern` so a teammate's agent choice never lands in committed
    /// config. Read back by `crate::host::Host`. On a target write it is
    /// preserved from the existing file unless supplied here.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "cli-config-dir"
    )]
    #[arg(long = "cli-config-dir")]
    pub cli_config_dir: Option<String>,
    /// Clear mode: remove the target block (feature / path / scenario /
    /// scenario-path / set-at) while preserving the per-contributor
    /// `cli-config-dir`. Mutually exclusive with a target write —
    /// supplying `clear` together with any of `feature`, `path`,
    /// `scenario`, or `scenario-path` is rejected. A `cli-config-dir`
    /// supplied alongside `clear` still applies (the supplied value
    /// overrides the preserved one).
    #[serde(default)]
    #[arg(long)]
    pub clear: bool,
}

/// Result for `write-session`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct WriteSessionResult {
    /// Repo-relative path of the written session file (always
    /// `.govern.session.toml` — kept on the result for symmetry with
    /// other write primitives' return shapes).
    pub path: String,
    /// `true` when the file did not exist before this call, `false` when
    /// an existing file was overwritten in place.
    pub created: bool,
}

// -- resolve-references ------------------------------------------------------

/// Args for `resolve-references`. Resolves the consumer feature's derived
/// `references:` index (see spec 030) against the `.govern.toml` `[services]`
/// registry, reading each linked spec's live `status` from its local
/// checkout. Takes only the consumer feature; the repo root is supplied by
/// the runtime.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct ResolveReferencesArgs {
    /// Consumer feature directory name under `specs/` whose `references:`
    /// index is resolved.
    #[arg(long)]
    pub feature: String,
}

/// Closed outcome enum for one resolved cross-service reference. Decided by
/// deterministic predicates — no prose is read for intent. Canonical in
/// `specs/030-cross-service-references/data-model.md`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ReferenceOutcome {
    /// Registered, checkout reachable, target spec resolves, `status` present
    /// and in the allowed set. `status` carries the linked lifecycle value.
    Ok,
    /// The reference's service is null (the href repo matched no `[services]`
    /// entry at harvest time, or the alias is no longer registered) — a plain
    /// navigational link; status not attempted.
    Unregistered,
    /// Registered, but the service's local `path` is missing or not a usable
    /// checkout. Informational unknown, never reported as broken.
    NotCheckedOut,
    /// Registered and reachable, but the target spec does not resolve
    /// (renamed / moved / deleted / mistyped upstream, or a malformed URL that
    /// yielded no such spec). A provable defect — an analyze finding.
    Broken,
    /// The target file exists but its `status` cannot be read (no frontmatter,
    /// malformed YAML, missing or out-of-set `status`). Surfaced, never silent;
    /// the defect is upstream's.
    StatusUnreadable,
}

/// One resolution record: the input reference plus its classified outcome
/// and, on `ok`, the linked lifecycle status.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ResolutionRecord {
    /// Matched registry alias, or `null` when the reference is `unregistered`.
    pub service: Option<String>,
    /// Target `NNN-slug` (the stable reference identity).
    pub spec: String,
    /// Classified outcome.
    pub outcome: ReferenceOutcome,
    /// Linked lifecycle status; non-null only when `outcome` is `ok`.
    pub status: Option<String>,
}

/// Result for `resolve-references`: one record per entry in the consumer's
/// `references:` index, in index order.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ResolveReferencesResult {
    /// Resolution records in the consumer spec's `references:` order.
    pub references: Vec<ResolutionRecord>,
    /// Repo-relative path to the consumer spec file.
    pub path: String,
}

// -- resolve-feature ----------------------------------------------------------

/// Args for `resolve-feature`. Scans the configured spec root and resolves
/// a user-supplied identifier to a feature directory — the deterministic
/// core of `/gov:target`'s specs-dir scan.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct ResolveFeatureArgs {
    /// Identifier to resolve: an exact feature directory name
    /// (`022-deterministic-runtime`), a feature number (`22` or `022` —
    /// both match the zero-padded `022-` prefix), or a partial slug
    /// substring (`deterministic`, matched case-insensitively).
    #[arg(long)]
    pub identifier: String,
    /// Optional scenario slug. When supplied and the feature resolves, the
    /// result's `scenario` block reports the scenario file's existence and
    /// its `section` frontmatter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub scenario: Option<String>,
}

/// Closed outcome enum for `resolve-feature`. Ambiguity and no-match are
/// **domain outcomes** (the host mediates the follow-up prompt), never
/// operational errors — per the scaffolding-primitives scenario's edge
/// cases ("choosing stays with the user through the host").
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ResolveFeatureOutcome {
    /// Exactly one feature matched the identifier.
    Resolved,
    /// A partial identifier matched more than one feature; `candidates`
    /// carries the sorted matches for the host's disambiguation prompt.
    Ambiguous,
    /// No feature matched the identifier.
    NotFound,
}

/// Scenario detail attached to a `resolve-feature` result when the args
/// named a scenario slug and the feature resolved.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ResolvedScenario {
    /// Scenario slug echoed from the args.
    pub slug: String,
    /// Repo-relative path of the scenario file (reported whether or not
    /// the file exists, so the host can offer to create it).
    pub path: String,
    /// Whether the scenario file exists on disk.
    pub exists: bool,
    /// Scenario frontmatter `section` field (falling back to the legacy
    /// pre-017 `spec-ref` field). Empty when the file is absent,
    /// unreadable, or carries neither — mirroring `dashboard`'s
    /// scenario-detail degradation.
    pub section: String,
}

/// Result for `resolve-feature`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ResolveFeatureResult {
    /// How the identifier resolved.
    pub outcome: ResolveFeatureOutcome,
    /// Resolved feature directory name; present only on `resolved`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature: Option<String>,
    /// Repo-relative feature directory path; present only on `resolved`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Spec frontmatter `status`; present on `resolved` when the feature's
    /// `spec.md` is readable (best-effort — a malformed spec degrades to
    /// an absent status rather than failing the resolution).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Sorted candidate directory names; populated on `ambiguous`, empty
    /// on the other outcomes.
    #[serde(default)]
    pub candidates: Vec<String>,
    /// Scenario detail; present when the args named a scenario slug and
    /// the outcome is `resolved`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario: Option<ResolvedScenario>,
}

// -- create-feature -----------------------------------------------------------

/// Args for `create-feature`. Computes the next feature number, derives
/// the kebab-case slug from `title`, creates `{specs-root}/{NNN-slug}/`,
/// and copies the spec template into it — the deterministic scaffold step
/// of `/gov:specify`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct CreateFeatureArgs {
    /// Feature title. The directory slug is derived from it: lowercased,
    /// every run of non-alphanumeric characters collapsed to a single
    /// hyphen, leading/trailing hyphens trimmed.
    #[arg(long)]
    pub title: String,
}

/// Result for `create-feature`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct CreateFeatureResult {
    /// Whether the feature directory was created. `false` is the refusal
    /// domain outcome: the derived directory already existed and nothing
    /// was written (no overwrite path).
    pub created: bool,
    /// Feature directory name (`{NNN}-{slug}`).
    pub feature: String,
    /// Repo-relative feature directory path.
    pub path: String,
    /// Repo-relative path of the spec template copied into the new
    /// directory; absent on the refusal outcome.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

// -- create-plan-artifacts -------------------------------------------------

/// Args for `create-plan-artifacts`. Copies the plan/tasks (and, on
/// request, data-model) templates into an existing feature directory —
/// the deterministic template-copy and existing-artifact-detection step
/// of `/gov:plan` (the plan-side mirror of `create-feature`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct CreatePlanArtifactsArgs {
    /// Feature directory name under the configured spec root.
    #[arg(long)]
    pub feature: String,
    /// Also copy the data-model template. Whether the feature introduces
    /// or modifies domain entities is the host's judgment, so
    /// `data-model.md` joins the copy set only on request. A pre-existing
    /// `data-model.md` is reported (`kept`) regardless, so the
    /// existing-artifact prompt always sees the full set.
    #[serde(default)]
    #[arg(long)]
    pub include_data_model: bool,
    /// Copy fresh templates over pre-existing artifacts — the "replace"
    /// branch of the existing-artifact prompt, passed only after the user
    /// confirms. Default `false`: pre-existing artifacts are never
    /// touched (`kept`).
    #[serde(default)]
    #[arg(long)]
    pub overwrite: bool,
}

/// Outcome for one plan artifact.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum PlanArtifactAction {
    /// The artifact was missing; the template was copied in.
    Created,
    /// The artifact pre-existed and was left untouched.
    Kept,
    /// The artifact pre-existed and the template was copied over it
    /// (`overwrite: true`).
    Replaced,
}

/// Per-artifact report entry for `create-plan-artifacts`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct PlanArtifact {
    /// Artifact file name: `plan.md`, `tasks.md`, or `data-model.md`.
    pub file: String,
    /// Repo-relative artifact path.
    pub path: String,
    /// What happened to the artifact this call.
    pub action: PlanArtifactAction,
    /// Repo-relative path of the template copied in; absent on `kept`.
    /// No last-modified stamp accompanies `kept` entries — primitive
    /// results carry no wall-clock data (same rule as `write-session`,
    /// whose `set-at` goes into the file, never the result), so the
    /// envelope stream stays deterministic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub template: Option<String>,
}

/// Result for `create-plan-artifacts`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct CreatePlanArtifactsResult {
    /// Repo-relative feature directory path.
    pub path: String,
    /// Per-artifact outcomes in canonical order (`plan.md`, `tasks.md`,
    /// `data-model.md`). A `data-model.md` that is neither requested nor
    /// on disk is omitted.
    pub artifacts: Vec<PlanArtifact>,
}

// -- check-review-gate -------------------------------------------------------

/// Args for `check-review-gate`. Evaluates `/gov:implement`'s pre-done
/// review gate for one feature: the feature directory's markdown lint,
/// then the spec frontmatter `review:` block, in the completion gate's
/// documented order (first failing check wins).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct CheckReviewGateArgs {
    /// Feature directory name under the configured spec root.
    #[arg(long)]
    pub feature: String,
}

/// First failing check of the pre-done review gate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewGateBlock {
    /// The feature directory's markdown files failed `markdownlint-cli2`
    /// (violations, or a non-zero exit the parser could not attribute).
    MarkdownLint,
    /// The spec has no completed review: the `review:` block is absent or
    /// its `last-run` is null.
    NotReviewed,
    /// The last review left blocking MUST violations
    /// (`review.blocking: true`).
    MustViolations,
}

/// Result for `check-review-gate`. A blocked gate is a domain outcome —
/// the host halts with `message` and does not propose the in-progress →
/// done transition; it is never an operational error.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct CheckReviewGateResult {
    /// Whether the gate passes and the transition may be proposed.
    pub passed: bool,
    /// First failing check, in gate order; absent on pass.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub blocked_by: Option<ReviewGateBlock>,
    /// The canonical blocked message for the failing check (the
    /// `blocked: …` texts documented in `/gov:implement`'s completion
    /// gate, with the adopter's `[host] project` command namespace
    /// substituted); absent on pass.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    /// Follow-up guidance accompanying the message — the
    /// resolve-or-waive options on `must-violations`; absent otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guidance: Option<String>,
    /// Markdown-lint violations backing a `markdown-lint` block; empty
    /// otherwise.
    pub violations: Vec<MarkdownViolation>,
}

// -- append-question ---------------------------------------------------------

/// Args for `append-question`. Appends one question bullet to the target
/// artifact's `## Open Questions` section — `/gov:amend`'s question-route
/// write, including the same-write status back-edge on spec targets.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct AppendQuestionArgs {
    /// Feature directory name under the configured spec root.
    #[arg(long)]
    pub feature: String,
    /// Refined question text, appended as a `- {question}` bullet.
    /// Single-line; embedded newlines are rejected (structure injection).
    #[arg(long)]
    pub question: String,
    /// Optional scenario slug: the target artifact becomes
    /// `scenarios/{slug}.md` instead of `spec.md`, and no status
    /// back-edge applies (scenarios have no status field).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub scenario: Option<String>,
}

/// Result for `append-question`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct AppendQuestionResult {
    /// Repo-relative path of the target artifact.
    pub path: String,
    /// Whether the question was appended. `false` is the dedup domain
    /// outcome: an equivalent entry already exists and nothing was
    /// written.
    pub appended: bool,
    /// The existing entry that suppressed the append (normalized-
    /// whitespace, case-insensitive match); present only when deduped.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duplicate_of: Option<String>,
    /// Whether the `## Open Questions` section had to be created.
    pub section_created: bool,
    /// Whether the same-write status back-edge fired: a spec target
    /// whose status was `clarified`, `planned`, `in-progress`, or `done`
    /// reverts to `draft` in the same atomic write as the append.
    pub status_reverted: bool,
    /// The status the back-edge reverted from; present only when
    /// `status-reverted` is `true`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_status: Option<String>,
}

// -- diff-cross-spec ---------------------------------------------------------

/// Args for `diff-cross-spec`. Computes `/gov:implement`'s cross-spec
/// impact surface: the diff from the feature's first spec-dir commit to
/// the working tree, scoped to the spec root and filtered to paths
/// outside the feature's own directory, plus the lines added to
/// `{specs-root}/inbox.md` in the same window.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct DiffCrossSpecArgs {
    /// Feature directory name under the configured spec root.
    #[arg(long)]
    pub feature: String,
}

/// Result for `diff-cross-spec`. Read-only; both lists empty is the
/// no-impact domain outcome.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct DiffCrossSpecResult {
    /// First commit touching the feature's spec dir (the diff base;
    /// shared derivation with `derive-boundary`).
    pub first_commit: String,
    /// Current HEAD commit.
    pub current_head: String,
    /// Changed paths under the spec root but outside the feature's own
    /// directory (sorted; `{specs-root}/inbox.md` is excluded — its
    /// additions report separately below). The diff runs against the
    /// working tree (index + untracked included), so uncommitted sibling
    /// edits surface at the per-task summary; on a clean tree this equals
    /// the documented `git diff <first-commit>..HEAD -- {specs-root}/`.
    pub cross_spec_paths: Vec<String>,
    /// Bullet lines added to `{specs-root}/inbox.md` in the window — the
    /// issues captured during the feature's work (§brownfield-inbox).
    /// Filtered through the shared bullet grammar, so structural
    /// additions (the heading, blanks when the whole file is new) never
    /// report as captured items.
    pub inbox_additions: Vec<String>,
}

// -- append-inbox --------------------------------------------------------------

/// Args for `append-inbox`. Appends one `- {text}` bullet to
/// `{specs-root}/inbox.md`, creating the file when missing. The optional
/// `dedup-prefix` makes the append idempotent for auto-capture callers
/// (the bootstrap audit's dedup-by-prefix contract).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct AppendInboxArgs {
    /// Single-line bullet text, appended as `- {text}`. Embedded newlines
    /// are rejected (structure injection into inbox.md).
    #[arg(long)]
    pub text: String,
    /// Optional dedup guard: when an existing inbox bullet's text starts
    /// with this prefix, nothing is written and the result reports
    /// `deduped: true`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[arg(long)]
    pub dedup_prefix: Option<String>,
}

/// Result for `append-inbox`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct AppendInboxResult {
    /// Repo-relative path of the inbox file.
    pub path: String,
    /// Whether the inbox file was created by this invocation.
    pub created: bool,
    /// `true` when `dedup-prefix` matched an existing bullet and no write
    /// happened.
    pub deduped: bool,
}

// -- remove-inbox-item ---------------------------------------------------------

/// Args for `remove-inbox-item`. Removes the first bullet from
/// `{specs-root}/inbox.md` whose text matches `item`. The complement of
/// `append-inbox`; the deterministic surface behind `/gov:groom`'s per-item
/// inbox removal (step 8), which previously edited the file by hand.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct RemoveInboxItemArgs {
    /// The bullet text to remove: the first inbox bullet whose text (after
    /// the `- ` marker and an optional `[ ]`/`[x]` checkbox are stripped),
    /// trimmed, equals this value is removed. Single-line; embedded newlines
    /// are rejected.
    #[arg(long)]
    pub item: String,
}

/// Result for `remove-inbox-item`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct RemoveInboxItemResult {
    /// Repo-relative path of the inbox file.
    pub path: String,
    /// Whether a matching bullet was found and removed. A no-match (or a
    /// missing inbox file) is a clean domain outcome, not an error.
    pub removed: bool,
    /// Number of bullet items remaining in the inbox after the operation.
    pub remaining_count: u32,
}

// -- check-artifacts -----------------------------------------------------------

/// Args for `check-artifacts`. Runs the residual deterministic check
/// families from `/gov:analyze`'s markdown-only reference against one
/// feature (`--all` stays with the caller looping).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct CheckArtifactsArgs {
    /// Feature directory name under the configured spec root.
    #[arg(long)]
    pub feature: String,
}

/// One deterministic artifact finding. Family names and severity tiers
/// mirror `framework/commands/analyze.md`'s markdown-only reference —
/// the primitive mechanizes the documented policy, it introduces none.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ArtifactFinding {
    /// Check family: `artifact-completeness`, `task-consistency`,
    /// `scenario-consistency`, or `review-state-drift`.
    pub family: String,
    /// Severity tier per the reference's assignments: `blocking`
    /// (artifact completeness, task consistency, review state drift) or
    /// `advisory` (scenario consistency).
    pub severity: String,
    /// Human-readable description of the finding.
    pub message: String,
    /// Repo-relative path of the artifact the finding anchors to.
    pub path: String,
}

/// Result for `check-artifacts`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct CheckArtifactsResult {
    /// Feature directory name echoed from the args.
    pub feature: String,
    /// Spec frontmatter `status` the tier classification ran against.
    pub status: String,
    /// Findings across the four families, in family order
    /// (completeness → task consistency → scenario consistency →
    /// review drift).
    pub findings: Vec<ArtifactFinding>,
    /// `true` when no family produced a finding.
    pub clean: bool,
    /// Repo-relative path to the spec file.
    pub path: String,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::{
        AcceptanceCriterion, AnchorReference, CheckRuleIdsArgs, CheckRuleIdsResult, CheckStuckArgs,
        CheckStuckResult, CheckboxToggleResult, Classification, DependencyEdge, DeriveBoundaryArgs,
        DeriveBoundaryResult, Frontmatter, FrontmatterFinding, GateConfirmArgs, GateConfirmResult,
        LintMarkdownArgs, LintMarkdownResult, MarkCriterionArgs, MarkTaskArgs, MarkdownViolation,
        MigrateSessionFileArgs, MigrateSessionFileResult, OpenQuestion, PruneAction, PruneGate,
        PruneMode, PruneSection, PruneTasksArgs, PruneTasksResult, ReadSpecArgs, ReadSpecResult,
        ReadTasksArgs, ReadTasksResult, ResolveAnchorArgs, ResolveAnchorResult, ReviewBlock,
        RuleCitation, RunGeneratorArgs, RunGeneratorResult, SetStatusArgs, SetStatusResult,
        SizeSummary, SpecSection, Subtask, Task, TraverseDepsArgs, TraverseDepsResult,
        ValidateFrontmatterArgs, ValidateFrontmatterResult, WriteSessionArgs, WriteSessionResult,
    };

    fn round_trip<T>(value: &T) -> T
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        let text = serde_json::to_string(value).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    #[test]
    fn read_spec_args_use_kebab_case() {
        let args = ReadSpecArgs {
            feature: "022-deterministic-runtime".into(),
            include_body: true,
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["feature"], "022-deterministic-runtime");
        assert_eq!(value["include-body"], true);
        assert_eq!(round_trip(&args), args);
    }

    #[test]
    fn read_spec_result_round_trip() {
        let result = ReadSpecResult {
            frontmatter: Frontmatter {
                status: "clarified".into(),
                dependencies: vec!["021-runtime-boundary".into()],
                tags: vec![],
                review: Some(ReviewBlock::default()),
            },
            sections: vec![SpecSection {
                heading: "Motivation".into(),
                level: 2,
                body: "…".into(),
            }],
            acceptance_criteria: vec![AcceptanceCriterion {
                checked: false,
                text: "A single binary builds…".into(),
            }],
            open_questions: vec![OpenQuestion { text: "?".into() }],
            path: "specs/022-deterministic-runtime/spec.md".into(),
        };
        let value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert!(value.get("acceptance-criteria").is_some());
        assert!(value.get("open-questions").is_some());
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn read_tasks_round_trip() {
        let result = ReadTasksResult {
            tasks: vec![Task {
                number: "1".into(),
                heading: "Bootstrap".into(),
                subtasks: vec![Subtask {
                    text: "Create Cargo.toml".into(),
                    checked: true,
                }],
                done_when: Some("cargo build succeeds".into()),
                phase: None,
            }],
            path: "specs/022-deterministic-runtime/tasks.md".into(),
        };
        let value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(value["tasks"][0]["done-when"], "cargo build succeeds");
        // `phase: None` must not surface in the JSON — backward-compat for
        // existing consumers that pre-date the phased read-tasks fix.
        assert!(
            !value["tasks"][0].as_object().unwrap().contains_key("phase"),
            "phase: None should serialize as absent, not null"
        );
        assert_eq!(round_trip(&result), result);
        let args = ReadTasksArgs {
            feature: "022-deterministic-runtime".into(),
        };
        assert_eq!(round_trip(&args), args);
    }

    #[test]
    fn read_tasks_phased_task_carries_phase_metadata() {
        let result = ReadTasksResult {
            tasks: vec![Task {
                number: "1".into(),
                heading: "Wire up".into(),
                subtasks: vec![],
                done_when: None,
                phase: Some("Phase A — Bootstrap".into()),
            }],
            path: "specs/022-deterministic-runtime/tasks.md".into(),
        };
        let value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(value["tasks"][0]["phase"], "Phase A — Bootstrap");
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn mark_task_round_trip() {
        let args = MarkTaskArgs {
            feature: "022-deterministic-runtime".into(),
            task_number: "2".into(),
            subtask_index: 0,
            checked: true,
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["task-number"], "2");
        assert_eq!(value["subtask-index"], 0);
        assert_eq!(round_trip(&args), args);

        let result = CheckboxToggleResult {
            previous: false,
            current: true,
            path: "specs/022-deterministic-runtime/tasks.md".into(),
        };
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn prune_tasks_round_trip() {
        let args = PruneTasksArgs {
            feature: "041-task-pruning".into(),
            reset: false,
            force: false,
            apply: true,
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["feature"], "041-task-pruning");
        assert_eq!(value["reset"], false);
        assert_eq!(value["apply"], true);
        assert_eq!(round_trip(&args), args);

        let result = PruneTasksResult {
            mode: PruneMode::KeepPending,
            applied: false,
            gate: PruneGate::NotApplicable,
            status: None,
            nothing_to_prune: false,
            removed_count: 1,
            kept_count: 1,
            size_before: SizeSummary {
                lines: 40,
                bytes: 900,
            },
            size_after: SizeSummary {
                lines: 20,
                bytes: 450,
            },
            sections: vec![
                PruneSection {
                    number: "1".into(),
                    heading: "Schema types".into(),
                    phase: Some("Phase A".into()),
                    classification: Classification::Spent,
                    checkbox_total: 2,
                    checkbox_checked: 2,
                    action: PruneAction::Removed,
                },
                PruneSection {
                    number: "2".into(),
                    heading: "Segmentation".into(),
                    phase: Some("Phase A".into()),
                    classification: Classification::Pending,
                    checkbox_total: 3,
                    checkbox_checked: 1,
                    action: PruneAction::Kept,
                },
            ],
            path: "specs/041-task-pruning/tasks.md".into(),
        };
        let value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(value["mode"], "keep-pending");
        assert_eq!(value["gate"], "not-applicable");
        assert_eq!(value["nothing-to-prune"], false);
        assert_eq!(value["removed-count"], 1);
        assert_eq!(value["size-before"]["lines"], 40);
        assert_eq!(value["sections"][0]["classification"], "spent");
        assert_eq!(value["sections"][0]["action"], "removed");
        assert_eq!(value["sections"][1]["classification"], "pending");
        // `status: None` must serialize as absent, not null.
        assert!(!value.as_object().unwrap().contains_key("status"));
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn mark_criterion_round_trip() {
        let args = MarkCriterionArgs {
            feature: "022-deterministic-runtime".into(),
            criterion_index: 3,
            checked: true,
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["criterion-index"], 3);
        assert_eq!(round_trip(&args), args);
    }

    #[test]
    fn set_status_round_trip() {
        let args = SetStatusArgs {
            feature: "022-deterministic-runtime".into(),
            from: "clarified".into(),
            to: "planned".into(),
        };
        assert_eq!(round_trip(&args), args);
        let result = SetStatusResult {
            previous: "clarified".into(),
            current: "planned".into(),
            path: "specs/022-deterministic-runtime/spec.md".into(),
        };
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn derive_boundary_round_trip() {
        let args = DeriveBoundaryArgs {
            feature: "022-deterministic-runtime".into(),
        };
        assert_eq!(round_trip(&args), args);
        let result = DeriveBoundaryResult {
            boundary: vec![
                "specs/022-deterministic-runtime/**".into(),
                "runtime/**".into(),
            ],
            first_commit: "d398083".into(),
            current_head: "6f0f54e".into(),
        };
        let value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(value["first-commit"], "d398083");
        assert_eq!(value["current-head"], "6f0f54e");
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn check_stuck_round_trip() {
        let args = CheckStuckArgs {
            feature: "022-deterministic-runtime".into(),
            threshold: 10,
        };
        assert_eq!(round_trip(&args), args);
        let result = CheckStuckResult {
            commit_count: 3,
            stuck: false,
            since_sha: "abcdef0".into(),
            threshold: 10,
        };
        let value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(value["commit-count"], 3);
        assert_eq!(value["since-sha"], "abcdef0");
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn validate_frontmatter_round_trip() {
        let args = ValidateFrontmatterArgs {
            path: "specs/022-deterministic-runtime/spec.md".into(),
        };
        assert_eq!(round_trip(&args), args);
        let result = ValidateFrontmatterResult {
            findings: vec![FrontmatterFinding {
                severity: "blocking".into(),
                field: "status".into(),
                message: "unknown status".into(),
            }],
            clean: false,
        };
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn resolve_anchor_round_trip() {
        let args = ResolveAnchorArgs {
            path: "framework/constitution.md".into(),
        };
        assert_eq!(round_trip(&args), args);
        let result = ResolveAnchorResult {
            references: vec![AnchorReference {
                anchor: "runtime-boundary".into(),
                line: 459,
                resolved: true,
            }],
            unresolved: vec![],
        };
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn traverse_deps_round_trip() {
        let args = TraverseDepsArgs {
            feature: "022-deterministic-runtime".into(),
        };
        assert_eq!(round_trip(&args), args);
        let result = TraverseDepsResult {
            dependencies: vec![DependencyEdge {
                feature: "021-runtime-boundary".into(),
                exists: true,
                status: "done".into(),
                compatible: true,
            }],
            compatible: true,
            cycles: Vec::new(),
        };
        assert_eq!(round_trip(&result), result);
        // Cycle-bearing payload round-trips with the new field populated.
        let with_cycles = TraverseDepsResult {
            dependencies: vec![DependencyEdge {
                feature: "100-a".into(),
                exists: true,
                status: "planned".into(),
                compatible: true,
            }],
            compatible: false,
            cycles: vec![vec!["100-a".into(), "101-b".into()]],
        };
        assert_eq!(round_trip(&with_cycles), with_cycles);
    }

    #[test]
    fn check_rule_ids_round_trip() {
        let args = CheckRuleIdsArgs {
            path: "specs/022-deterministic-runtime/spec.md".into(),
            rule_files: vec!["framework/rules/security-backend.md".into()],
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(
            value["rule-files"][0],
            "framework/rules/security-backend.md"
        );
        assert_eq!(round_trip(&args), args);
        let result = CheckRuleIdsResult {
            citations: vec![RuleCitation {
                rule_id: "SEC-AUTH-001".into(),
                found: true,
                deprecated: false,
            }],
            missing: vec![],
            deprecated: vec![],
        };
        let value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(value["citations"][0]["rule-id"], "SEC-AUTH-001");
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn run_generator_round_trip() {
        let args = RunGeneratorArgs {
            script: "scripts/gen-spec-deps.sh".into(),
        };
        assert_eq!(round_trip(&args), args);
        let result = RunGeneratorResult {
            drift: false,
            stdout: "ok\n".into(),
            stderr: String::new(),
            exit_code: 0,
        };
        let value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(value["exit-code"], 0);
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn lint_markdown_round_trip() {
        let args = LintMarkdownArgs {
            paths: vec!["framework/constitution.md".into()],
            fix: false,
        };
        assert_eq!(round_trip(&args), args);
        let result = LintMarkdownResult {
            violations: vec![MarkdownViolation {
                path: "README.md".into(),
                line: 17,
                rule: "MD013".into(),
                message: "Line length".into(),
            }],
            clean: false,
            exit_code: 1,
        };
        let value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(value["exit-code"], 1);
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn gate_confirm_round_trip() {
        let args = GateConfirmArgs {
            gate: "plan-finalize-status".into(),
            prompt: "Advance status from clarified to planned?".into(),
        };
        assert_eq!(round_trip(&args), args);
        let result = GateConfirmResult { confirmed: true };
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn merge_claude_md_round_trip() {
        use super::{MergeClaudeMdArgs, MergeClaudeMdResult};
        let args = MergeClaudeMdArgs {
            path: "CLAUDE.md".into(),
            block: "framework block body".into(),
            marker: Some("govern-managed".into()),
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["block"], "framework block body");
        assert_eq!(value["marker"], "govern-managed");
        assert_eq!(round_trip(&args), args);

        let result = MergeClaudeMdResult {
            path: "CLAUDE.md".into(),
            action: "created".into(),
            marker: "govern-managed".into(),
        };
        let r_value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(r_value["action"], "created");
        assert_eq!(round_trip(&result), result);

        // marker omitted serializes without the field
        let args_no_marker = MergeClaudeMdArgs {
            path: "CLAUDE.md".into(),
            block: "x".into(),
            marker: None,
        };
        let v: serde_json::Value = serde_json::to_value(&args_no_marker).unwrap();
        assert!(!v.as_object().unwrap().contains_key("marker"));
    }

    #[test]
    fn substitute_templates_round_trip() {
        use super::{SubstituteTemplatesArgs, SubstituteTemplatesResult};
        use std::collections::BTreeMap;
        let mut subs = BTreeMap::new();
        subs.insert("project".into(), "anvil".into());
        let args = SubstituteTemplatesArgs {
            source_dir: "/tmp/staging".into(),
            target_dir: "/tmp/project".into(),
            substitutions: subs,
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["substitutions"]["project"], "anvil");
        assert_eq!(value["source-dir"], "/tmp/staging");
        assert_eq!(value["target-dir"], "/tmp/project");
        assert_eq!(round_trip(&args), args);

        let result = SubstituteTemplatesResult {
            target_dir: "/tmp/project".into(),
            files_written: 5,
            substitutions_applied: 12,
            files: vec!["README.md".into()],
        };
        let r_value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(r_value["files-written"], 5);
        assert_eq!(r_value["substitutions-applied"], 12);
        assert_eq!(r_value["target-dir"], "/tmp/project");
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn extract_archive_round_trip() {
        use super::{ExtractArchiveArgs, ExtractArchiveResult};
        let args = ExtractArchiveArgs {
            archive: "/tmp/gvrn.tar.gz".into(),
            dest: "/tmp/out".into(),
            format: Some("tar-gz".into()),
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["archive"], "/tmp/gvrn.tar.gz");
        assert_eq!(value["dest"], "/tmp/out");
        assert_eq!(value["format"], "tar-gz");
        assert_eq!(round_trip(&args), args);

        let result = ExtractArchiveResult {
            dest: "/tmp/out".into(),
            files: vec!["a.txt".into(), "dir/b.txt".into()],
            count: 2,
            format: "tar-gz".into(),
        };
        let r_value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(r_value["count"], 2);
        assert_eq!(r_value["files"][1], "dir/b.txt");
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn merge_managed_block_round_trip() {
        use super::{MergeManagedBlockArgs, MergeManagedBlockResult};
        let args = MergeManagedBlockArgs {
            path: ".gitignore".into(),
            block: ".claude/\nspecs/.cache/".into(),
            marker: Some("govern-managed".into()),
            marker_style: Some("line-prefix".into()),
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["marker-style"], "line-prefix");
        assert_eq!(value["marker"], "govern-managed");
        assert_eq!(round_trip(&args), args);

        // marker_style omitted serializes without the field.
        let args_default_style = MergeManagedBlockArgs {
            path: "CLAUDE.md".into(),
            block: "x".into(),
            marker: None,
            marker_style: None,
        };
        let v: serde_json::Value = serde_json::to_value(&args_default_style).unwrap();
        assert!(!v.as_object().unwrap().contains_key("marker-style"));
        assert!(!v.as_object().unwrap().contains_key("marker"));

        let result = MergeManagedBlockResult {
            path: ".gitignore".into(),
            action: "inserted".into(),
            marker: "govern-managed".into(),
            marker_style: "line-prefix".into(),
            dedup_removed: Some(2),
            dedup_removed_lines: Some(vec![".claude/".into(), "*.sqlite".into()]),
        };
        let r_value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(r_value["marker-style"], "line-prefix");
        assert_eq!(r_value["dedup-removed"], 2);
        assert_eq!(round_trip(&result), result);

        // html-comment shape: dedup fields are absent from JSON when None.
        let html_result = MergeManagedBlockResult {
            path: "CLAUDE.md".into(),
            action: "updated".into(),
            marker: "govern-managed".into(),
            marker_style: "html-comment".into(),
            dedup_removed: None,
            dedup_removed_lines: None,
        };
        let v: serde_json::Value = serde_json::to_value(&html_result).unwrap();
        assert!(!v.as_object().unwrap().contains_key("dedup-removed"));
        assert!(!v.as_object().unwrap().contains_key("dedup-removed-lines"));
    }

    #[test]
    fn merge_permissions_round_trip() {
        use super::{MergePermissionsArgs, MergePermissionsResult};
        let args = MergePermissionsArgs {
            path: ".claude/settings.local.json".into(),
            allow: vec!["Bash(ls *)".into(), "Edit".into()],
            deny: vec!["Bash(rm -rf *)".into()],
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["path"], ".claude/settings.local.json");
        assert_eq!(value["allow"][0], "Bash(ls *)");
        assert_eq!(round_trip(&args), args);

        // A non-Claude host supplies its own settings path; the runtime
        // does not hardcode `.claude/`.
        let auggie_args = MergePermissionsArgs {
            path: ".augment/settings.json".into(),
            allow: vec![],
            deny: vec![],
        };
        let v: serde_json::Value = serde_json::to_value(&auggie_args).unwrap();
        assert_eq!(v["path"], ".augment/settings.json");

        let result = MergePermissionsResult {
            path: ".claude/settings.local.json".into(),
            action: "updated".into(),
            allow_added: 2,
            allow_deduped: 1,
            deny_added: 0,
            deny_deduped: 0,
        };
        let r_value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(r_value["allow-added"], 2);
        assert_eq!(r_value["allow-deduped"], 1);
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn enforce_manifest_round_trip() {
        use super::{EnforceManifestArgs, EnforceManifestResult};
        let args = EnforceManifestArgs {
            directory: ".claude/commands/anvil".into(),
            expected: vec!["status.md".into(), "target.md".into()],
            pinned: vec!["adopter-custom.md".into()],
            recursive: false,
            glob_include: Some("*.md".into()),
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["directory"], ".claude/commands/anvil");
        assert_eq!(value["expected"][0], "status.md");
        assert_eq!(value["glob-include"], "*.md");
        assert_eq!(round_trip(&args), args);

        // glob_include omitted serializes without the field.
        let args_default_glob = EnforceManifestArgs {
            directory: "x".into(),
            expected: vec![],
            pinned: vec![],
            recursive: true,
            glob_include: None,
        };
        let v: serde_json::Value = serde_json::to_value(&args_default_glob).unwrap();
        assert!(!v.as_object().unwrap().contains_key("glob-include"));
        assert_eq!(v["recursive"], true);

        let result = EnforceManifestResult {
            removed: vec!["legacy.md".into()],
            kept: vec!["status.md".into()],
            pinned_kept: vec!["adopter-custom.md".into()],
        };
        let r_value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(r_value["pinned-kept"][0], "adopter-custom.md");
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn apply_manifest_round_trip() {
        use super::{ApplyManifestArgs, ApplyManifestResult, ManifestEntry, ManifestEntryResult};
        use std::collections::BTreeMap;
        let mut subs = BTreeMap::new();
        subs.insert("project".into(), "anvil".into());
        let args = ApplyManifestArgs {
            source_root: "/tmp/staging".into(),
            target_root: "/tmp/project".into(),
            entries: vec![
                ManifestEntry {
                    source: "framework/commands/status.md".into(),
                    dest: "framework/commands/status.md".into(),
                    strategy: "update".into(),
                    keep_literals: None,
                },
                ManifestEntry {
                    source: "govern.md".into(),
                    dest: ".claude/commands/anvil/govern.md".into(),
                    strategy: "update".into(),
                    keep_literals: Some(vec!["project".into(), "cli-config-dir".into()]),
                },
            ],
            pinned: vec!["AGENTS.md".into()],
            substitutions: subs,
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["source-root"], "/tmp/staging");
        assert_eq!(value["target-root"], "/tmp/project");
        assert_eq!(value["entries"][0]["strategy"], "update");
        assert_eq!(value["entries"][1]["keep-literals"][0], "project");
        // keep-literals omitted on the first entry should not serialize.
        assert!(
            value["entries"][0]
                .as_object()
                .unwrap()
                .get("keep-literals")
                .is_none()
        );
        assert_eq!(round_trip(&args), args);

        let result = ApplyManifestResult {
            entries: vec![ManifestEntryResult {
                source: "framework/commands/status.md".into(),
                dest: "framework/commands/status.md".into(),
                action: "created".into(),
            }],
            created: 1,
            updated: 0,
            unchanged: 0,
            skipped_exists: 0,
            skipped_pinned: 1,
            source_missing: 0,
        };
        let r_value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(r_value["skipped-pinned"], 1);
        assert_eq!(r_value["source-missing"], 0);
        assert_eq!(r_value["entries"][0]["action"], "created");
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn fetch_archive_round_trip() {
        use super::{FetchArchiveArgs, FetchArchiveResult};
        let args = FetchArchiveArgs {
            url: "https://example.test/gvrn-0.2.0.tar.gz".into(),
            sha256_url: Some("https://example.test/gvrn-0.2.0.tar.gz.sha256".into()),
            archive: "/tmp/gvrn.tar.gz".into(),
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(
            value["sha256-url"],
            "https://example.test/gvrn-0.2.0.tar.gz.sha256"
        );
        assert_eq!(value["archive"], "/tmp/gvrn.tar.gz");
        assert_eq!(round_trip(&args), args);

        // Absent sha256_url omits the field entirely.
        let args_no_sidecar = FetchArchiveArgs {
            url: "https://example.test/main.tar.gz".into(),
            sha256_url: None,
            archive: "/tmp/main.tar.gz".into(),
        };
        let v: serde_json::Value = serde_json::to_value(&args_no_sidecar).unwrap();
        assert!(!v.as_object().unwrap().contains_key("sha256-url"));
        assert_eq!(round_trip(&args_no_sidecar), args_no_sidecar);

        let result = FetchArchiveResult {
            path: "/tmp/gvrn.tar.gz".into(),
            sha256: "abc123".into(),
            verified: true,
            bytes: 12345,
        };
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn migrate_session_file_round_trip() {
        let args = MigrateSessionFileArgs {
            legacy_path: ".claude/gov-session.json".into(),
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["legacy-path"], ".claude/gov-session.json");
        assert_eq!(round_trip(&args), args);

        // Adopter on a non-Claude host or non-`gov` project name:
        let auggie_args = MigrateSessionFileArgs {
            legacy_path: ".augment/anvil-session.json".into(),
        };
        let v: serde_json::Value = serde_json::to_value(&auggie_args).unwrap();
        assert_eq!(v["legacy-path"], ".augment/anvil-session.json");

        let result = MigrateSessionFileResult {
            source: ".claude/gov-session.json".into(),
            dest: ".govern.session.toml".into(),
            action: "migrated".into(),
            legacy_deleted: true,
        };
        let r_value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(r_value["source"], ".claude/gov-session.json");
        assert_eq!(r_value["dest"], ".govern.session.toml");
        assert_eq!(r_value["action"], "migrated");
        assert_eq!(r_value["legacy-deleted"], true);
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn write_session_round_trip() {
        let args = WriteSessionArgs {
            feature: Some("022-deterministic-runtime".into()),
            path: Some("specs/022-deterministic-runtime".into()),
            scenario: Some("write-session-primitive".into()),
            scenario_path: Some(
                "specs/022-deterministic-runtime/scenarios/write-session-primitive.md".into(),
            ),
            cli_config_dir: None,
            clear: false,
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        // CLI/MCP args remain kebab-case to match every other primitive.
        assert_eq!(value["feature"], "022-deterministic-runtime");
        assert_eq!(value["path"], "specs/022-deterministic-runtime");
        assert_eq!(value["scenario"], "write-session-primitive");
        assert_eq!(
            value["scenario-path"],
            "specs/022-deterministic-runtime/scenarios/write-session-primitive.md"
        );
        assert_eq!(round_trip(&args), args);

        // Host-config write: only `cli-config-dir` set; the target fields
        // are absent.
        let args_host = WriteSessionArgs {
            feature: None,
            path: None,
            scenario: None,
            scenario_path: None,
            cli_config_dir: Some(".opencode".into()),
            clear: false,
        };
        let vh: serde_json::Value = serde_json::to_value(&args_host).unwrap();
        let objh = vh.as_object().unwrap();
        assert!(!objh.contains_key("feature"));
        assert_eq!(vh["cli-config-dir"], ".opencode");
        assert_eq!(round_trip(&args_host), args_host);

        // Absent scenario + scenario-path omit both fields.
        let args_no_scenario = WriteSessionArgs {
            feature: Some("002-target".into()),
            path: Some("specs/002-target".into()),
            scenario: None,
            scenario_path: None,
            cli_config_dir: None,
            clear: false,
        };
        let v: serde_json::Value = serde_json::to_value(&args_no_scenario).unwrap();
        let obj = v.as_object().unwrap();
        assert!(!obj.contains_key("scenario"));
        assert!(!obj.contains_key("scenario-path"));
        assert_eq!(round_trip(&args_no_scenario), args_no_scenario);

        // Clear write: only `clear` set; `clear` serializes as a plain
        // boolean and an absent `clear` key deserializes to `false`
        // (backward compatibility for pre-clear callers).
        let args_clear = WriteSessionArgs {
            feature: None,
            path: None,
            scenario: None,
            scenario_path: None,
            cli_config_dir: None,
            clear: true,
        };
        let vc: serde_json::Value = serde_json::to_value(&args_clear).unwrap();
        assert_eq!(vc["clear"], true);
        assert_eq!(round_trip(&args_clear), args_clear);
        let legacy: WriteSessionArgs = serde_json::from_str("{}").unwrap();
        assert!(!legacy.clear, "absent `clear` defaults to false");

        let result = WriteSessionResult {
            path: ".govern.session.toml".into(),
            created: true,
        };
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn resolve_feature_round_trip() {
        use super::{
            ResolveFeatureArgs, ResolveFeatureOutcome, ResolveFeatureResult, ResolvedScenario,
        };
        let args = ResolveFeatureArgs {
            identifier: "22".into(),
            scenario: Some("scaffolding-primitives".into()),
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["identifier"], "22");
        assert_eq!(value["scenario"], "scaffolding-primitives");
        assert_eq!(round_trip(&args), args);

        // Absent scenario omits the field.
        let bare = ResolveFeatureArgs {
            identifier: "runtime".into(),
            scenario: None,
        };
        let v: serde_json::Value = serde_json::to_value(&bare).unwrap();
        assert!(!v.as_object().unwrap().contains_key("scenario"));

        let resolved = ResolveFeatureResult {
            outcome: ResolveFeatureOutcome::Resolved,
            feature: Some("022-deterministic-runtime".into()),
            path: Some("specs/022-deterministic-runtime".into()),
            status: Some("in-progress".into()),
            candidates: vec![],
            scenario: Some(ResolvedScenario {
                slug: "scaffolding-primitives".into(),
                path: "specs/022-deterministic-runtime/scenarios/scaffolding-primitives.md".into(),
                exists: true,
                section: "Follow-on scenarios".into(),
            }),
        };
        let rv: serde_json::Value = serde_json::to_value(&resolved).unwrap();
        assert_eq!(rv["outcome"], "resolved");
        assert_eq!(rv["scenario"]["exists"], true);
        assert_eq!(rv["scenario"]["section"], "Follow-on scenarios");
        assert_eq!(round_trip(&resolved), resolved);

        // Ambiguous carries the sorted candidate list; the resolved-only
        // fields serialize as absent, not null.
        let ambiguous = ResolveFeatureResult {
            outcome: ResolveFeatureOutcome::Ambiguous,
            feature: None,
            path: None,
            status: None,
            candidates: vec!["001-a-runtime".into(), "002-b-runtime".into()],
            scenario: None,
        };
        let av: serde_json::Value = serde_json::to_value(&ambiguous).unwrap();
        assert_eq!(av["outcome"], "ambiguous");
        assert_eq!(av["candidates"][0], "001-a-runtime");
        let obj = av.as_object().unwrap();
        assert!(!obj.contains_key("feature"));
        assert!(!obj.contains_key("status"));
        assert_eq!(round_trip(&ambiguous), ambiguous);

        let not_found = ResolveFeatureResult {
            outcome: ResolveFeatureOutcome::NotFound,
            feature: None,
            path: None,
            status: None,
            candidates: vec![],
            scenario: None,
        };
        let nv: serde_json::Value = serde_json::to_value(&not_found).unwrap();
        assert_eq!(nv["outcome"], "not-found");
        assert_eq!(round_trip(&not_found), not_found);
    }

    #[test]
    fn create_feature_round_trip() {
        use super::{CreateFeatureArgs, CreateFeatureResult};
        let args = CreateFeatureArgs {
            title: "Deterministic Runtime!".into(),
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["title"], "Deterministic Runtime!");
        assert_eq!(round_trip(&args), args);

        let result = CreateFeatureResult {
            created: true,
            feature: "043-deterministic-runtime".into(),
            path: "specs/043-deterministic-runtime".into(),
            template: Some("framework/templates/spec/spec.md".into()),
        };
        let rv: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(rv["created"], true);
        assert_eq!(rv["template"], "framework/templates/spec/spec.md");
        assert_eq!(round_trip(&result), result);

        // Refusal outcome: template absent from the JSON, not null.
        let refused = CreateFeatureResult {
            created: false,
            feature: "043-deterministic-runtime".into(),
            path: "specs/043-deterministic-runtime".into(),
            template: None,
        };
        let fv: serde_json::Value = serde_json::to_value(&refused).unwrap();
        assert!(!fv.as_object().unwrap().contains_key("template"));
        assert_eq!(round_trip(&refused), refused);
    }

    #[test]
    fn create_plan_artifacts_round_trip() {
        use super::{
            CreatePlanArtifactsArgs, CreatePlanArtifactsResult, PlanArtifact, PlanArtifactAction,
        };
        let args = CreatePlanArtifactsArgs {
            feature: "042-widget".into(),
            include_data_model: true,
            overwrite: false,
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["feature"], "042-widget");
        assert_eq!(value["include-data-model"], true);
        assert_eq!(value["overwrite"], false);
        assert_eq!(round_trip(&args), args);

        // Booleans default false when omitted (host sends only `feature`).
        let minimal: CreatePlanArtifactsArgs =
            serde_json::from_value(serde_json::json!({"feature": "042-widget"})).unwrap();
        assert!(!minimal.include_data_model);
        assert!(!minimal.overwrite);

        let result = CreatePlanArtifactsResult {
            path: "specs/042-widget".into(),
            artifacts: vec![
                PlanArtifact {
                    file: "plan.md".into(),
                    path: "specs/042-widget/plan.md".into(),
                    action: PlanArtifactAction::Created,
                    template: Some("specs/templates/plan.md".into()),
                },
                PlanArtifact {
                    file: "tasks.md".into(),
                    path: "specs/042-widget/tasks.md".into(),
                    action: PlanArtifactAction::Kept,
                    template: None,
                },
            ],
        };
        let rv: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(rv["artifacts"][0]["action"], "created");
        assert_eq!(rv["artifacts"][1]["action"], "kept");
        // `template` is absent from the JSON on kept, not null.
        let kept = rv["artifacts"][1].as_object().unwrap();
        assert!(!kept.contains_key("template"));
        assert_eq!(round_trip(&result), result);

        let replaced: serde_json::Value =
            serde_json::to_value(PlanArtifactAction::Replaced).unwrap();
        assert_eq!(replaced, "replaced");
    }

    #[test]
    fn check_review_gate_round_trip() {
        use super::{CheckReviewGateArgs, CheckReviewGateResult, ReviewGateBlock};
        let args = CheckReviewGateArgs {
            feature: "042-widget".into(),
        };
        assert_eq!(round_trip(&args), args);

        let passed = CheckReviewGateResult {
            passed: true,
            blocked_by: None,
            message: None,
            guidance: None,
            violations: vec![],
        };
        let pv: serde_json::Value = serde_json::to_value(&passed).unwrap();
        // Options are absent from the JSON on pass, not null.
        let obj = pv.as_object().unwrap();
        assert!(!obj.contains_key("blocked-by"));
        assert!(!obj.contains_key("message"));
        assert!(!obj.contains_key("guidance"));
        assert_eq!(round_trip(&passed), passed);

        let blocked = CheckReviewGateResult {
            passed: false,
            blocked_by: Some(ReviewGateBlock::MustViolations),
            message: Some(
                "blocked: spec has 3 MUST violation(s) — see specs/042-widget/review.md".into(),
            ),
            guidance: Some("Resolve the violations and re-run /gov:review".into()),
            violations: vec![],
        };
        let bv: serde_json::Value = serde_json::to_value(&blocked).unwrap();
        assert_eq!(bv["blocked-by"], "must-violations");
        assert_eq!(round_trip(&blocked), blocked);

        let lint: serde_json::Value = serde_json::to_value(ReviewGateBlock::MarkdownLint).unwrap();
        assert_eq!(lint, "markdown-lint");
        let unreviewed: serde_json::Value =
            serde_json::to_value(ReviewGateBlock::NotReviewed).unwrap();
        assert_eq!(unreviewed, "not-reviewed");
    }

    #[test]
    fn append_question_round_trip() {
        use super::{AppendQuestionArgs, AppendQuestionResult};
        let args = AppendQuestionArgs {
            feature: "042-widget".into(),
            question: "Should rate limits be configurable per tenant?".into(),
            scenario: None,
        };
        let av: serde_json::Value = serde_json::to_value(&args).unwrap();
        // Absent scenario is omitted from the JSON, not null.
        assert!(!av.as_object().unwrap().contains_key("scenario"));
        assert_eq!(round_trip(&args), args);

        let scenario_target = AppendQuestionArgs {
            scenario: Some("retry-on-timeout".into()),
            ..args
        };
        let sv: serde_json::Value = serde_json::to_value(&scenario_target).unwrap();
        assert_eq!(sv["scenario"], "retry-on-timeout");
        assert_eq!(round_trip(&scenario_target), scenario_target);

        let appended = AppendQuestionResult {
            path: "specs/042-widget/spec.md".into(),
            appended: true,
            duplicate_of: None,
            section_created: false,
            status_reverted: true,
            previous_status: Some("planned".into()),
        };
        let rv: serde_json::Value = serde_json::to_value(&appended).unwrap();
        assert_eq!(rv["status-reverted"], true);
        assert_eq!(rv["previous-status"], "planned");
        assert!(!rv.as_object().unwrap().contains_key("duplicate-of"));
        assert_eq!(round_trip(&appended), appended);

        let deduped = AppendQuestionResult {
            path: "specs/042-widget/spec.md".into(),
            appended: false,
            duplicate_of: Some("Should rate limits be configurable per tenant?".into()),
            section_created: false,
            status_reverted: false,
            previous_status: None,
        };
        let dv: serde_json::Value = serde_json::to_value(&deduped).unwrap();
        assert_eq!(
            dv["duplicate-of"],
            "Should rate limits be configurable per tenant?"
        );
        assert!(!dv.as_object().unwrap().contains_key("previous-status"));
        assert_eq!(round_trip(&deduped), deduped);
    }

    #[test]
    fn diff_cross_spec_round_trip() {
        use super::{DiffCrossSpecArgs, DiffCrossSpecResult};
        let args = DiffCrossSpecArgs {
            feature: "042-widget".into(),
        };
        assert_eq!(round_trip(&args), args);

        let result = DiffCrossSpecResult {
            first_commit: "abc123".into(),
            current_head: "def456".into(),
            cross_spec_paths: vec!["specs/007-sibling/spec.md".into()],
            inbox_additions: vec!["- security: token logged in plaintext".into()],
        };
        let rv: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(rv["cross-spec-paths"][0], "specs/007-sibling/spec.md");
        assert_eq!(
            rv["inbox-additions"][0],
            "- security: token logged in plaintext"
        );
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn append_inbox_round_trip() {
        use super::{AppendInboxArgs, AppendInboxResult};
        let args = AppendInboxArgs {
            text: "security: token logged in plaintext — src/auth.rs (captured during 022)".into(),
            dedup_prefix: Some("security: token logged".into()),
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(value["dedup-prefix"], "security: token logged");
        assert_eq!(round_trip(&args), args);

        // Absent dedup-prefix omits the field.
        let bare = AppendInboxArgs {
            text: "x".into(),
            dedup_prefix: None,
        };
        let v: serde_json::Value = serde_json::to_value(&bare).unwrap();
        assert!(!v.as_object().unwrap().contains_key("dedup-prefix"));

        let result = AppendInboxResult {
            path: "specs/inbox.md".into(),
            created: false,
            deduped: true,
        };
        let rv: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(rv["deduped"], true);
        assert_eq!(round_trip(&result), result);
    }

    #[test]
    fn check_artifacts_round_trip() {
        use super::{ArtifactFinding, CheckArtifactsArgs, CheckArtifactsResult};
        let args = CheckArtifactsArgs {
            feature: "022-deterministic-runtime".into(),
        };
        assert_eq!(round_trip(&args), args);

        let result = CheckArtifactsResult {
            feature: "022-deterministic-runtime".into(),
            status: "planned".into(),
            findings: vec![ArtifactFinding {
                family: "artifact-completeness".into(),
                severity: "blocking".into(),
                message: "plan.md is required at status 'planned' but does not exist".into(),
                path: "specs/022-deterministic-runtime/plan.md".into(),
            }],
            clean: false,
            path: "specs/022-deterministic-runtime/spec.md".into(),
        };
        let rv: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(rv["findings"][0]["family"], "artifact-completeness");
        assert_eq!(rv["findings"][0]["severity"], "blocking");
        assert_eq!(rv["clean"], false);
        assert_eq!(round_trip(&result), result);
    }
}
