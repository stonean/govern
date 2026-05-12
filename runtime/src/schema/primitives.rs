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

/// Parsed spec frontmatter.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct Frontmatter {
    /// Pipeline status (e.g., "clarified", "planned", "in-progress", "done").
    pub status: String,
    /// Dependency feature names.
    #[serde(default)]
    pub dependencies: Vec<String>,
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
    /// Overall compatibility (logical AND across edges).
    pub compatible: bool,
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
    /// Whether the lint produced no violations.
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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema, clap::Args)]
#[serde(rename_all = "kebab-case")]
pub struct FetchArchiveArgs {
    /// URL of the archive (`.tar.gz`, `.zip`, etc.).
    #[arg(long)]
    pub url: String,
    /// URL of the sha256 sidecar file (matching the `shasum -a 256` format —
    /// one or more lines of `<hex>  <filename>`).
    #[arg(long)]
    pub sha256_url: String,
    /// Local destination path for the downloaded archive.
    #[arg(long)]
    pub dest: String,
}

/// Result for `fetch-archive`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct FetchArchiveResult {
    /// Repo-relative or absolute path where the archive was written.
    pub path: String,
    /// Lowercase hex sha256 of the downloaded archive (matches the sidecar).
    pub sha256: String,
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::{
        AcceptanceCriterion, AnchorReference, CheckRuleIdsArgs, CheckRuleIdsResult, CheckStuckArgs,
        CheckStuckResult, CheckboxToggleResult, DependencyEdge, DeriveBoundaryArgs,
        DeriveBoundaryResult, Frontmatter, FrontmatterFinding, GateConfirmArgs, GateConfirmResult,
        LintMarkdownArgs, LintMarkdownResult, MarkCriterionArgs, MarkTaskArgs, MarkdownViolation,
        OpenQuestion, ReadSpecArgs, ReadSpecResult, ReadTasksArgs, ReadTasksResult,
        ResolveAnchorArgs, ResolveAnchorResult, ReviewBlock, RuleCitation, RunGeneratorArgs,
        RunGeneratorResult, SetStatusArgs, SetStatusResult, SpecSection, Subtask, Task,
        TraverseDepsArgs, TraverseDepsResult, ValidateFrontmatterArgs, ValidateFrontmatterResult,
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
            }],
            path: "specs/022-deterministic-runtime/tasks.md".into(),
        };
        let value: serde_json::Value = serde_json::to_value(&result).unwrap();
        assert_eq!(value["tasks"][0]["done-when"], "cargo build succeeds");
        assert_eq!(round_trip(&result), result);
        let args = ReadTasksArgs {
            feature: "022-deterministic-runtime".into(),
        };
        assert_eq!(round_trip(&args), args);
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
        };
        assert_eq!(round_trip(&result), result);
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
    fn fetch_archive_round_trip() {
        use super::{FetchArchiveArgs, FetchArchiveResult};
        let args = FetchArchiveArgs {
            url: "https://example.test/gvrn-0.2.0.tar.gz".into(),
            sha256_url: "https://example.test/gvrn-0.2.0.tar.gz.sha256".into(),
            dest: "/tmp/gvrn.tar.gz".into(),
        };
        let value: serde_json::Value = serde_json::to_value(&args).unwrap();
        assert_eq!(
            value["sha256-url"],
            "https://example.test/gvrn-0.2.0.tar.gz.sha256"
        );
        assert_eq!(value["dest"], "/tmp/gvrn.tar.gz");
        assert_eq!(round_trip(&args), args);

        let result = FetchArchiveResult {
            path: "/tmp/gvrn.tar.gz".into(),
            sha256: "abc123".into(),
            bytes: 12345,
        };
        assert_eq!(round_trip(&result), result);
    }
}
