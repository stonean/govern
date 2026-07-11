//! LLM extension-point payload schemas.
//!
//! Mirrors the request/response shapes in
//! `specs/022-deterministic-runtime/data-model.md`: the three
//! initial-release points (`assessSpecQuality`, `writeCode`,
//! `writeSpecBody`), `performReview`, and the follow-on points —
//! `askClarifyQuestion` and `routeInboxItem`, whose typed shapes ship
//! ahead of their scenarios per the extension-request-hygiene scenario,
//! plus `verifyCriteria`, the implement-completion-gate scenario's
//! criterion-verification seam. The runtime emits these as the `request`
//! field of `llm-request` envelopes and validates incoming `llm-response`
//! payloads against them.

#![allow(clippy::module_name_repetitions)]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

// -- assessSpecQuality -------------------------------------------------------

/// Verification request for one rule.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct AssessSpecQualityRule {
    /// Rule ID (e.g., "QUAL-CLARITY-001").
    pub id: String,
    /// Verification phrase from the rule's definition.
    pub verification: String,
    /// Severity tier ("must", "should", "info").
    pub severity: String,
}

/// Request payload for `assessSpecQuality`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct AssessSpecQualityRequest {
    /// Repo-relative path to the spec file under review.
    pub spec_path: String,
    /// Full spec contents.
    pub spec_content: String,
    /// Rule whose Verification clause is being assessed.
    pub rule: AssessSpecQualityRule,
}

/// Location annotation inside a finding.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct FindingLocation {
    /// Section heading the finding refers to.
    pub section: String,
    /// 1-based line in the spec.
    pub line: u32,
}

/// One finding emitted by `assessSpecQuality`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct AssessSpecQualityFinding {
    /// Severity tier echoed from the rule.
    pub severity: String,
    /// Rule ID the finding belongs to.
    pub rule_id: String,
    /// Where the finding applies in the spec.
    pub location: FindingLocation,
    /// Human-readable description.
    pub message: String,
}

/// Response payload for `assessSpecQuality`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct AssessSpecQualityResponse {
    /// Whether the rule passed.
    pub passed: bool,
    /// The finding when `passed` is false; `None` when `passed` is true.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finding: Option<AssessSpecQualityFinding>,
}

// -- writeCode ---------------------------------------------------------------

/// Task description payload (mirrors the read-tasks shape).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct WriteCodeTask {
    /// Top-level task number.
    pub number: String,
    /// Task heading.
    pub heading: String,
    /// Sub-item texts.
    pub subtasks: Vec<String>,
}

/// One plan-relevant file the LLM may need to read.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct PlanRelevantFile {
    /// Repo-relative file path.
    pub path: String,
    /// File contents at request time.
    pub content: String,
}

/// Request payload for `writeCode`.
///
/// Field order is the **cache-anchor contract** documented in spec 022's
/// `LLM extension points` section: the stable prefix
/// (`constitution-excerpts`, `plan-relevant-files`, `write-boundary`) is
/// contiguous and front so a host can place a prompt-cache breakpoint
/// immediately before `task`. The per-task variable suffix (`task`) is
/// last. Reordering the fields here changes the on-wire field order;
/// hosts integrating against the protocol rely on this layout.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct WriteCodeRequest {
    /// Constitution excerpts the runtime determined are relevant.
    pub constitution_excerpts: Vec<String>,
    /// Files the plan named as relevant for this task.
    pub plan_relevant_files: Vec<PlanRelevantFile>,
    /// Runtime write boundary (glob patterns and concrete paths).
    pub write_boundary: Vec<String>,
    /// Task being implemented.
    pub task: WriteCodeTask,
}

/// Edit-action discriminator on a `writeCode` edit.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum WriteCodeAction {
    /// Create a new file (`content` is required).
    Create,
    /// Modify an existing file (`patch` or `content` is required).
    Edit,
    /// Delete an existing file.
    Delete,
}

/// One edit emitted by the LLM.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct WriteCodeEdit {
    /// Repo-relative target path; must fall within the request's
    /// `write-boundary`.
    pub path: String,
    /// Edit action.
    pub action: WriteCodeAction,
    /// Full file content for `create` edits, or replacement content for
    /// content-mode `edit` edits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Unified diff for patch-mode `edit` edits.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub patch: Option<String>,
}

/// Response payload for `writeCode`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct WriteCodeResponse {
    /// Edits to apply, in order.
    pub edits: Vec<WriteCodeEdit>,
    /// One-line summary surfaced in `progress` messages.
    pub summary: String,
}

// -- writeSpecBody -----------------------------------------------------------

/// Request payload for `writeSpecBody`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct WriteSpecBodyRequest {
    /// Repo-relative path to the template the section is being filled into.
    pub template_path: String,
    /// Full template contents.
    pub template_content: String,
    /// Section heading being filled.
    pub section: String,
    /// User-provided feature description supplied at the slash command.
    pub feature_description: String,
    /// Existing section content (when re-running on a partially-filled file).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub existing_content: Option<String>,
}

/// Response payload for `writeSpecBody`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct WriteSpecBodyResponse {
    /// Filled-in section content (markdown).
    pub content: String,
    /// Section heading echoed from the request.
    pub section: String,
}

// -- performReview -----------------------------------------------------------

/// One in-scope file a review pass reads.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ReviewScopeFile {
    /// Repo-relative file path.
    pub path: String,
    /// File contents at request time.
    pub content: String,
}

/// One rule file loaded for the pass (basename + full text).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ReviewRuleFile {
    /// Rule-file basename (e.g., "security-backend.md").
    pub name: String,
    /// Full rule-file contents.
    pub content: String,
}

/// Request payload for `performReview` — one single-shot request per pass
/// (five passes: security, reuse, quality, efficiency, simplicity).
///
/// `scope-files` is identical across every pass of a run, so it leads the
/// payload as the cache-stable prefix; `rule-files` and `pass` vary per pass
/// and trail it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct PerformReviewRequest {
    /// In-scope files under review (same set across all passes).
    pub scope_files: Vec<ReviewScopeFile>,
    /// Rule files loaded for this pass.
    pub rule_files: Vec<ReviewRuleFile>,
    /// Pass name: `security` / `reuse` / `quality` / `efficiency` /
    /// `simplicity`.
    pub pass: String,
}

/// Response payload for `performReview`. The `findings` array flows directly
/// into `write-review`'s `findings` input — the walker accumulates each pass's
/// findings across the run (see [`crate::interpreter`]).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct PerformReviewResponse {
    /// Findings from this pass, in the shape `write-review` consumes.
    pub findings: Vec<crate::schema::primitives::ReviewFinding>,
}

// -- askClarifyQuestion --------------------------------------------------------

/// One open question presented to the user (host-mediated, one
/// request/response round trip per question).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ClarifyQuestion {
    /// Question text, verbatim from the spec's Open Questions list.
    pub text: String,
    /// Spec section the question is attributed to; omitted when the
    /// walker cannot attribute it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub section: Option<String>,
}

/// Request payload for `askClarifyQuestion` (reserved by the
/// clarify-command-acceleration scenario; the typed builder ships ahead
/// of it per extension-request-hygiene so the point never falls back to
/// a raw walker-context dump).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct AskClarifyQuestionRequest {
    /// Repo-relative path to the spec under clarification.
    pub spec_path: String,
    /// Full spec contents.
    pub spec_content: String,
    /// The open question this round trip resolves.
    pub question: ClarifyQuestion,
}

/// Response payload for `askClarifyQuestion` — the user's answer,
/// mediated by the host. Applying the resolution to the spec body
/// remains LLM work per the clarify scenario.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct AskClarifyQuestionResponse {
    /// The user's resolution, verbatim.
    pub answer: String,
}

// -- routeInboxItem ------------------------------------------------------------

/// One feature spec the inbox router may match an item against.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct RouteInboxSpec {
    /// Feature directory basename (`NNN-slug`).
    pub feature: String,
    /// Frontmatter `status` (drives the done → in-progress reopen
    /// consent on a scenario route); empty when unreadable.
    pub status: String,
}

/// Request payload for `routeInboxItem` (reserved by the
/// groom-command-acceleration scenario; typed builder ships ahead of it).
/// Kept minimal: the item under decision, the closed route vocabulary
/// (the groom decision tree's leaves, in walk order), and the specs the
/// router may match — enough for the routing decision without a
/// walker-context dump.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct RouteInboxItemRequest {
    /// Raw inbox item text under decision.
    pub item_text: String,
    /// Closed route vocabulary, in decision-tree walk order:
    /// `rule`, `spec`, `scenario`, `chore`, `discard`.
    pub routes: Vec<String>,
    /// Feature specs the router may target, sorted by slug.
    pub available_specs: Vec<RouteInboxSpec>,
}

/// Route decision discriminator — the groom decision tree's leaves.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum InboxRoute {
    /// Cross-cutting concern: promote to (or amend) a rule file.
    Rule,
    /// No covering spec exists: direct the user to `/gov:specify`.
    Spec,
    /// Durable behavioral requirement under an existing spec: create a
    /// scenario (plus task append and possible done → in-progress reopen).
    Scenario,
    /// Project maintenance with no feature home: stays in the inbox.
    Chore,
    /// Not actionable: remove from the inbox.
    Discard,
}

/// Response payload for `routeInboxItem` — the routing decision.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct RouteInboxItemResponse {
    /// Chosen route from the request's `routes` vocabulary.
    pub route: InboxRoute,
    /// Matched feature slug when the route targets an existing spec.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature: Option<String>,
    /// Optional prose justification the host may surface in the
    /// per-item confirmation prompt.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

// -- verifyCriteria ------------------------------------------------------------

/// One acceptance criterion presented for verification. Mirrors the
/// `read-spec` criteria listing; `index` is the 0-based body-order
/// position `mark-criterion` addresses.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct VerifyCriterion {
    /// 0-based criterion index, ordered as in the spec body.
    pub index: u32,
    /// Criterion text, verbatim from the Acceptance Criteria checkbox.
    pub text: String,
    /// Checkbox state at request time.
    pub checked: bool,
}

/// Request payload for `verifyCriteria` (implement-completion-gate
/// scenario): `/gov:implement`'s completion gate sends one request
/// carrying every acceptance criterion; the LLM judges each criterion
/// against the implementation — the verification stays semantic.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct VerifyCriteriaRequest {
    /// Repo-relative path to the spec under verification.
    pub spec_path: String,
    /// Full spec contents.
    pub spec_content: String,
    /// Acceptance criteria to verify, in body order.
    pub criteria: Vec<VerifyCriterion>,
}

/// One per-criterion verdict in a `verifyCriteria` response.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct VerifyCriterionResult {
    /// Criterion index echoed from the request.
    pub index: u32,
    /// Whether the criterion is met by the implementation.
    pub met: bool,
    /// Optional prose surfaced in the completion report (a failing
    /// criterion's note explains the failure).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

/// Response payload for `verifyCriteria` — one verdict per criterion.
/// Each `met: true` verdict drives one `mark-criterion` call; a
/// `met: false` verdict leaves its checkbox unchecked and is reported,
/// never batch-marked.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct VerifyCriteriaResponse {
    /// Per-criterion verdicts.
    pub results: Vec<VerifyCriterionResult>,
}

// -- validation --------------------------------------------------------------

/// Validation errors raised by [`validate_response`] and
/// [`validate_write_code_boundary`].
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// The extension identifier is not known to this runtime version.
    #[error("unknown extension point `{0}`")]
    UnknownExtension(String),
    /// The response payload did not match the expected schema.
    #[error("schema-mismatch in `{identifier}`: {source}")]
    Schema {
        /// Extension identifier the response was checked against.
        identifier: String,
        /// Underlying serde error (field path + reason).
        #[source]
        source: serde_json::Error,
    },
    /// A `writeCode` edit targeted a path outside the request's
    /// `write-boundary`, or used a form that is rejected before pattern
    /// matching (absolute, or containing `.` / `..` / empty segments) per
    /// the write-boundary-path-normalization scenario.
    #[error("out-of-boundary-edit: path `{path}` {reason}")]
    OutOfBoundary {
        /// Offending edit path.
        path: String,
        /// Boundary patterns the path was checked against.
        boundary: Vec<String>,
        /// Why the path was rejected: `is not within {boundary:?}` for a
        /// clean path matching no pattern, or a description naming the
        /// traversal segment / absolute prefix that disqualified the path
        /// before matching.
        reason: String,
    },
}

/// Deserialize `response` into the response type for `identifier`. Returns
/// the parsed value (boxed via [`Value`] for callers that prefer to keep
/// working with JSON), or a [`ValidationError::Schema`] when the payload
/// is malformed.
///
/// # Errors
///
/// Returns [`ValidationError::UnknownExtension`] when `identifier` is not
/// `assessSpecQuality`, `writeCode`, `writeSpecBody`, `performReview`,
/// `askClarifyQuestion`, `routeInboxItem`, or `verifyCriteria`; otherwise
/// [`ValidationError::Schema`] when deserialization fails.
pub fn validate_response(identifier: &str, response: &Value) -> Result<(), ValidationError> {
    macro_rules! check {
        ($ty:ty) => {{
            serde_json::from_value::<$ty>(response.clone())
                .map(|_| ())
                .map_err(|source| ValidationError::Schema {
                    identifier: identifier.into(),
                    source,
                })
        }};
    }
    match identifier {
        "assessSpecQuality" => check!(AssessSpecQualityResponse),
        "writeCode" => check!(WriteCodeResponse),
        "writeSpecBody" => check!(WriteSpecBodyResponse),
        "performReview" => check!(PerformReviewResponse),
        "askClarifyQuestion" => check!(AskClarifyQuestionResponse),
        "routeInboxItem" => check!(RouteInboxItemResponse),
        "verifyCriteria" => check!(VerifyCriteriaResponse),
        other => Err(ValidationError::UnknownExtension(other.into())),
    }
}

/// Check every edit path in a parsed `writeCode` response against the
/// `boundary` patterns. Returns the first offending path as
/// [`ValidationError::OutOfBoundary`]. A boundary entry ending in `/**`
/// matches any descendant; an entry ending in `/*` matches direct
/// children; any other entry is an exact-path match.
///
/// Before any pattern is consulted, each edit path is screened lexically
/// per the write-boundary-path-normalization scenario: absolute paths and
/// paths containing `.` / `..` / empty segments are rejected outright,
/// because raw prefix matching would let `runtime/../framework/x` satisfy
/// `runtime/**`. Boundary patterns themselves are trusted (they come from
/// `derive-boundary`); only response paths are suspect.
///
/// # Errors
///
/// See above.
pub fn validate_write_code_boundary(
    response: &WriteCodeResponse,
    boundary: &[String],
) -> Result<(), ValidationError> {
    for edit in &response.edits {
        if let Some(reason) = boundary_rejection_reason(&edit.path) {
            return Err(ValidationError::OutOfBoundary {
                path: edit.path.clone(),
                boundary: boundary.to_vec(),
                reason,
            });
        }
        if !path_in_boundary(&edit.path, boundary) {
            return Err(ValidationError::OutOfBoundary {
                path: edit.path.clone(),
                boundary: boundary.to_vec(),
                reason: format!("is not within {boundary:?}"),
            });
        }
    }
    Ok(())
}

/// Lexical pre-check applied to each edit path before pattern matching.
/// Returns a human-readable reason — naming the offending segment or
/// prefix — when the path is absolute (POSIX `/`, backslash-leading, or a
/// Windows drive prefix) or contains a `.` / `..` / empty segment; `None`
/// when the path is a clean repo-relative path safe to match. Paths are
/// rejected rather than normalized silently — the LLM is asked for clean
/// repo-relative paths.
///
/// The segment screen splits on **both** `/` and `\`: the release ships
/// the `x86_64-pc-windows-msvc` target, where `\` is a path separator, so
/// `runtime/a\..\..\x` would otherwise slip a `..` traversal past a `/`-only
/// split (its lone backslash-laden segment is not literally `..`) and
/// escape a `runtime/**` boundary. Treating `\` as a separator here keeps
/// the screen in step with the leading-`\` and `C:` Windows forms already
/// rejected above.
fn boundary_rejection_reason(path: &str) -> Option<String> {
    if path.starts_with('/') || path.starts_with('\\') {
        return Some("is absolute — provide clean repo-relative paths".into());
    }
    let mut chars = path.chars();
    if let (Some(drive), Some(':')) = (chars.next(), chars.next())
        && drive.is_ascii_alphabetic()
    {
        return Some(format!(
            "starts with drive prefix `{drive}:` — provide clean repo-relative paths"
        ));
    }
    for segment in path.split(['/', '\\']) {
        if segment == "." || segment == ".." {
            return Some(format!(
                "contains traversal segment `{segment}` — provide clean repo-relative paths"
            ));
        }
        if segment.is_empty() {
            return Some("contains an empty segment — provide clean repo-relative paths".into());
        }
    }
    None
}

fn path_in_boundary(path: &str, boundary: &[String]) -> bool {
    boundary
        .iter()
        .any(|pattern| matches_pattern(path, pattern))
}

fn matches_pattern(path: &str, pattern: &str) -> bool {
    if let Some(prefix) = pattern.strip_suffix("/**") {
        return path == prefix || path.starts_with(&format!("{prefix}/"));
    }
    if let Some(prefix) = pattern.strip_suffix("/*") {
        if let Some(rest) = path.strip_prefix(&format!("{prefix}/")) {
            return !rest.contains('/');
        }
        return false;
    }
    if pattern == "**" {
        return true;
    }
    path == pattern
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::{
        AskClarifyQuestionRequest, AskClarifyQuestionResponse, AssessSpecQualityFinding,
        AssessSpecQualityRequest, AssessSpecQualityResponse, AssessSpecQualityRule,
        ClarifyQuestion, FindingLocation, InboxRoute, PerformReviewRequest, PerformReviewResponse,
        PlanRelevantFile, ReviewRuleFile, ReviewScopeFile, RouteInboxItemRequest,
        RouteInboxItemResponse, RouteInboxSpec, VerifyCriteriaRequest, VerifyCriteriaResponse,
        VerifyCriterion, VerifyCriterionResult, WriteCodeAction, WriteCodeEdit, WriteCodeRequest,
        WriteCodeResponse, WriteCodeTask, WriteSpecBodyRequest, WriteSpecBodyResponse,
    };
    use crate::schema::primitives::ReviewFinding;

    fn round_trip<T>(value: &T) -> T
    where
        T: serde::Serialize + serde::de::DeserializeOwned,
    {
        let text = serde_json::to_string(value).unwrap();
        serde_json::from_str(&text).unwrap()
    }

    #[test]
    fn assess_spec_quality_round_trip() {
        let request = AssessSpecQualityRequest {
            spec_path: "specs/022-deterministic-runtime/spec.md".into(),
            spec_content: "# spec".into(),
            rule: AssessSpecQualityRule {
                id: "QUAL-CLARITY-001".into(),
                verification: "Acceptance criteria are concrete and testable".into(),
                severity: "must".into(),
            },
        };
        let value: serde_json::Value = serde_json::to_value(&request).unwrap();
        assert_eq!(
            value["spec-path"],
            "specs/022-deterministic-runtime/spec.md"
        );
        assert_eq!(value["spec-content"], "# spec");
        assert_eq!(round_trip(&request), request);

        let response = AssessSpecQualityResponse {
            passed: false,
            finding: Some(AssessSpecQualityFinding {
                severity: "must".into(),
                rule_id: "QUAL-CLARITY-001".into(),
                location: FindingLocation {
                    section: "Acceptance Criteria".into(),
                    line: 213,
                },
                message: "criterion 8 is not testable".into(),
            }),
        };
        let r_value: serde_json::Value = serde_json::to_value(&response).unwrap();
        assert_eq!(r_value["finding"]["rule-id"], "QUAL-CLARITY-001");
        assert_eq!(round_trip(&response), response);

        let passed = AssessSpecQualityResponse {
            passed: true,
            finding: None,
        };
        let p_value: serde_json::Value = serde_json::to_value(&passed).unwrap();
        assert!(!p_value.as_object().unwrap().contains_key("finding"));
        assert_eq!(round_trip(&passed), passed);
    }

    #[test]
    fn write_code_round_trip() {
        let request = WriteCodeRequest {
            task: WriteCodeTask {
                number: "3".into(),
                heading: "Implement read-spec primitive".into(),
                subtasks: vec!["Parse frontmatter".into()],
            },
            plan_relevant_files: vec![PlanRelevantFile {
                path: "runtime/src/primitives/read_spec.rs".into(),
                content: String::new(),
            }],
            write_boundary: vec![
                "runtime/**".into(),
                "specs/022-deterministic-runtime/**".into(),
            ],
            constitution_excerpts: vec!["§runtime-boundary…".into()],
        };
        let value: serde_json::Value = serde_json::to_value(&request).unwrap();
        assert_eq!(value["write-boundary"][0], "runtime/**");
        assert_eq!(
            value["plan-relevant-files"][0]["path"],
            "runtime/src/primitives/read_spec.rs"
        );
        assert_eq!(round_trip(&request), request);

        let response = WriteCodeResponse {
            edits: vec![
                WriteCodeEdit {
                    path: "runtime/src/primitives/read_spec.rs".into(),
                    action: WriteCodeAction::Create,
                    content: Some("// stub".into()),
                    patch: None,
                },
                WriteCodeEdit {
                    path: "runtime/src/primitives/mod.rs".into(),
                    action: WriteCodeAction::Edit,
                    content: None,
                    patch: Some("--- a/...\n+++ b/...".into()),
                },
            ],
            summary: "Implemented read-spec primitive".into(),
        };
        let r_value: serde_json::Value = serde_json::to_value(&response).unwrap();
        assert_eq!(r_value["edits"][0]["action"], "create");
        assert_eq!(r_value["edits"][1]["action"], "edit");
        assert_eq!(round_trip(&response), response);
    }

    #[test]
    fn validate_response_rejects_missing_required_field() {
        use super::{ValidationError, validate_response};
        let response = serde_json::json!({
            // `passed` is required by AssessSpecQualityResponse — leave it out
            "finding": {
                "severity": "must",
                "rule-id": "QUAL-CLARITY-001",
                "location": { "section": "Foo", "line": 1 },
                "message": "..."
            }
        });
        let err = validate_response("assessSpecQuality", &response).unwrap_err();
        match err {
            ValidationError::Schema { identifier, source } => {
                assert_eq!(identifier, "assessSpecQuality");
                assert!(source.to_string().contains("passed"));
            }
            other => panic!("expected Schema, got {other:?}"),
        }
    }

    #[test]
    fn validate_response_rejects_unexpected_enum_value() {
        use super::{ValidationError, validate_response};
        let response = serde_json::json!({
            "edits": [
                {
                    "path": "runtime/src/foo.rs",
                    "action": "rename",
                    "content": null,
                    "patch": null
                }
            ],
            "summary": "rename a file"
        });
        let err = validate_response("writeCode", &response).unwrap_err();
        match err {
            ValidationError::Schema { identifier, source } => {
                assert_eq!(identifier, "writeCode");
                assert!(
                    source.to_string().contains("rename") || source.to_string().contains("variant")
                );
            }
            other => panic!("expected Schema, got {other:?}"),
        }
    }

    #[test]
    fn validate_response_accepts_well_formed_payload() {
        use super::validate_response;
        let response = serde_json::json!({
            "passed": true
        });
        validate_response("assessSpecQuality", &response).unwrap();
    }

    #[test]
    fn validate_response_rejects_unknown_extension() {
        use super::{ValidationError, validate_response};
        let response = serde_json::json!({});
        let err = validate_response("notAnExtension", &response).unwrap_err();
        assert!(matches!(err, ValidationError::UnknownExtension(_)));
    }

    #[test]
    fn write_code_boundary_rejects_out_of_boundary_path() {
        use super::{ValidationError, validate_write_code_boundary};
        let response = WriteCodeResponse {
            edits: vec![WriteCodeEdit {
                path: "framework/constitution.md".into(),
                action: WriteCodeAction::Edit,
                content: Some("malicious".into()),
                patch: None,
            }],
            summary: "edit constitution".into(),
        };
        let boundary = vec![
            "runtime/**".into(),
            "specs/022-deterministic-runtime/**".into(),
        ];
        let err = validate_write_code_boundary(&response, &boundary).unwrap_err();
        // The pre-normalization message shape is preserved for clean paths
        // that simply match no pattern.
        assert!(err.to_string().contains("is not within"));
        match err {
            ValidationError::OutOfBoundary { path, .. } => {
                assert_eq!(path, "framework/constitution.md");
            }
            other => panic!("expected OutOfBoundary, got {other:?}"),
        }
    }

    /// One-edit `writeCode` response targeting `path`, for the boundary
    /// path-screening tests below.
    fn single_edit_response(path: &str) -> WriteCodeResponse {
        WriteCodeResponse {
            edits: vec![WriteCodeEdit {
                path: path.into(),
                action: WriteCodeAction::Edit,
                content: Some("x".into()),
                patch: None,
            }],
            summary: "edit".into(),
        }
    }

    #[test]
    fn write_code_boundary_rejects_dot_dot_traversal_inside_prefix() {
        // `runtime/../framework/x` starts with `runtime/`, so raw prefix
        // matching would accept it against `runtime/**` — the exact bypass
        // the write-boundary-path-normalization scenario closes.
        use super::{ValidationError, validate_write_code_boundary};
        let boundary = vec!["runtime/**".to_string()];
        let err = validate_write_code_boundary(
            &single_edit_response("runtime/../framework/x"),
            &boundary,
        )
        .unwrap_err();
        assert!(err.to_string().contains("traversal segment `..`"));
        match err {
            ValidationError::OutOfBoundary { path, .. } => {
                assert_eq!(path, "runtime/../framework/x");
            }
            other => panic!("expected OutOfBoundary, got {other:?}"),
        }
    }

    #[test]
    fn write_code_boundary_rejects_redundant_dot_segment() {
        // Lexically in-boundary but not clean — rejected rather than
        // normalized silently.
        use super::{ValidationError, validate_write_code_boundary};
        let boundary = vec!["runtime/**".to_string()];
        let err = validate_write_code_boundary(&single_edit_response("./runtime/x"), &boundary)
            .unwrap_err();
        assert!(err.to_string().contains("traversal segment `.`"));
        assert!(matches!(err, ValidationError::OutOfBoundary { .. }));
    }

    #[test]
    fn write_code_boundary_rejects_absolute_path() {
        use super::{ValidationError, validate_write_code_boundary};
        let boundary = vec!["runtime/**".to_string(), "**".to_string()];
        let err = validate_write_code_boundary(&single_edit_response("/etc/passwd"), &boundary)
            .unwrap_err();
        assert!(err.to_string().contains("is absolute"));
        assert!(matches!(err, ValidationError::OutOfBoundary { .. }));
    }

    #[test]
    fn write_code_boundary_rejects_windows_drive_and_empty_segments() {
        use super::validate_write_code_boundary;
        let boundary = vec!["runtime/**".to_string()];
        let drive = validate_write_code_boundary(&single_edit_response("C:/runtime/x"), &boundary)
            .unwrap_err();
        assert!(drive.to_string().contains("drive prefix `C:`"));
        let empty = validate_write_code_boundary(&single_edit_response("runtime//x"), &boundary)
            .unwrap_err();
        assert!(empty.to_string().contains("empty segment"));
    }

    #[test]
    fn write_code_boundary_rejects_backslash_traversal_segments() {
        // `runtime/a\..\..\x` starts with `runtime/` and its only
        // `/`-delimited tail segment (`a\..\..\x`) is not literally `..`,
        // so a `/`-only split would wave it through against `runtime/**` —
        // then `\` resolves as a separator on the Windows release target
        // and the `..` components escape the boundary. Splitting on both
        // separators catches the traversal.
        use super::{ValidationError, validate_write_code_boundary};
        let boundary = vec!["runtime/**".to_string()];
        let err =
            validate_write_code_boundary(&single_edit_response("runtime/a\\..\\..\\x"), &boundary)
                .unwrap_err();
        assert!(err.to_string().contains("traversal segment `..`"));
        match err {
            ValidationError::OutOfBoundary { path, .. } => {
                assert_eq!(path, "runtime/a\\..\\..\\x");
            }
            other => panic!("expected OutOfBoundary, got {other:?}"),
        }
    }

    #[test]
    fn write_code_boundary_still_accepts_clean_in_boundary_path() {
        use super::validate_write_code_boundary;
        let boundary = vec!["runtime/**".to_string()];
        validate_write_code_boundary(&single_edit_response("runtime/src/foo.rs"), &boundary)
            .unwrap();
    }

    #[test]
    fn write_code_boundary_accepts_in_boundary_paths() {
        use super::validate_write_code_boundary;
        let response = WriteCodeResponse {
            edits: vec![
                WriteCodeEdit {
                    path: "runtime/src/foo.rs".into(),
                    action: WriteCodeAction::Create,
                    content: Some("// hi".into()),
                    patch: None,
                },
                WriteCodeEdit {
                    path: "specs/022-deterministic-runtime/tasks.md".into(),
                    action: WriteCodeAction::Edit,
                    content: Some("...".into()),
                    patch: None,
                },
            ],
            summary: "ok".into(),
        };
        let boundary = vec![
            "runtime/**".into(),
            "specs/022-deterministic-runtime/**".into(),
        ];
        validate_write_code_boundary(&response, &boundary).unwrap();
    }

    #[test]
    fn boundary_pattern_double_star_matches_descendants() {
        use super::matches_pattern;
        assert!(matches_pattern("runtime/src/foo.rs", "runtime/**"));
        assert!(matches_pattern("runtime", "runtime/**"));
        assert!(!matches_pattern("framework/foo.md", "runtime/**"));
    }

    #[test]
    fn boundary_pattern_single_star_matches_direct_children_only() {
        use super::matches_pattern;
        assert!(matches_pattern("runtime/foo.rs", "runtime/*"));
        assert!(!matches_pattern("runtime/src/foo.rs", "runtime/*"));
    }

    #[test]
    fn boundary_pattern_exact_match() {
        use super::matches_pattern;
        assert!(matches_pattern("runtime/Cargo.toml", "runtime/Cargo.toml"));
        assert!(!matches_pattern("runtime/Cargo.lock", "runtime/Cargo.toml"));
    }

    #[test]
    fn write_code_request_serializes_with_cache_anchor_field_order() {
        // Locks the §LLM extension points cache-anchor contract: the stable
        // prefix (constitution-excerpts, plan-relevant-files, write-boundary)
        // must serialize contiguously and front; the per-task variable
        // suffix (task) must be last. Hosts that drop a prompt-cache
        // breakpoint between `write-boundary` and `task` rely on this layout.
        let request = WriteCodeRequest {
            constitution_excerpts: vec!["a".into()],
            plan_relevant_files: vec![PlanRelevantFile {
                path: "p".into(),
                content: "c".into(),
            }],
            write_boundary: vec!["runtime/**".into()],
            task: WriteCodeTask {
                number: "1".into(),
                heading: "h".into(),
                subtasks: vec![],
            },
        };
        let text = serde_json::to_string(&request).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        let keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(
            keys,
            vec![
                "constitution-excerpts",
                "plan-relevant-files",
                "write-boundary",
                "task",
            ]
        );
    }

    #[test]
    fn write_spec_body_round_trip() {
        let request = WriteSpecBodyRequest {
            template_path: "framework/templates/spec/spec.md".into(),
            template_content: "# {{ heading }}".into(),
            section: "Motivation".into(),
            feature_description: "Deterministic runtime".into(),
            existing_content: None,
        };
        let value: serde_json::Value = serde_json::to_value(&request).unwrap();
        assert_eq!(value["template-path"], "framework/templates/spec/spec.md");
        assert_eq!(value["feature-description"], "Deterministic runtime");
        assert!(!value.as_object().unwrap().contains_key("existing-content"));
        assert_eq!(round_trip(&request), request);

        let response = WriteSpecBodyResponse {
            content: "## Motivation\n\nA runtime…".into(),
            section: "Motivation".into(),
        };
        assert_eq!(round_trip(&response), response);
    }

    #[test]
    fn perform_review_round_trip() {
        let request = PerformReviewRequest {
            scope_files: vec![ReviewScopeFile {
                path: "runtime/src/main.rs".into(),
                content: "fn main() {}".into(),
            }],
            rule_files: vec![ReviewRuleFile {
                name: "security-backend.md".into(),
                content: "# Security".into(),
            }],
            pass: "security".into(),
        };
        let value: serde_json::Value = serde_json::to_value(&request).unwrap();
        // Cache-stable prefix leads: scope-files, then rule-files, then pass.
        let keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, vec!["scope-files", "rule-files", "pass"]);
        assert_eq!(value["scope-files"][0]["path"], "runtime/src/main.rs");
        assert_eq!(round_trip(&request), request);

        let response = PerformReviewResponse {
            findings: vec![ReviewFinding {
                rule: "SEC-BE-014".into(),
                severity: "must".into(),
                file: "runtime/src/main.rs".into(),
                line_range: "1-1".into(),
                confidence: "high".into(),
                ..ReviewFinding::default()
            }],
        };
        let r_value: serde_json::Value = serde_json::to_value(&response).unwrap();
        assert_eq!(r_value["findings"][0]["rule"], "SEC-BE-014");
        assert_eq!(r_value["findings"][0]["line-range"], "1-1");
        assert_eq!(round_trip(&response), response);
    }

    #[test]
    fn validate_response_accepts_perform_review_and_defaults_finding_extras() {
        use super::validate_response;
        // The 6 core fields the performReview contract names; the render extras
        // (summary / finding / rule-text / auto-fixable / suggested-fix) default.
        let response = serde_json::json!({
            "findings": [
                {
                    "rule": "SIM-001",
                    "severity": "should",
                    "file": "runtime/src/lib.rs",
                    "line-range": "10-20",
                    "confidence": "low"
                }
            ]
        });
        validate_response("performReview", &response).unwrap();
    }

    #[test]
    fn validate_response_rejects_malformed_perform_review() {
        use super::{ValidationError, validate_response};
        // `findings` must be an array of finding objects, not a bare string.
        let response = serde_json::json!({ "findings": "oops" });
        let err = validate_response("performReview", &response).unwrap_err();
        match err {
            ValidationError::Schema { identifier, .. } => assert_eq!(identifier, "performReview"),
            other => panic!("expected Schema, got {other:?}"),
        }
    }

    #[test]
    fn ask_clarify_question_round_trip() {
        let request = AskClarifyQuestionRequest {
            spec_path: "specs/022-deterministic-runtime/spec.md".into(),
            spec_content: "# spec".into(),
            question: ClarifyQuestion {
                text: "Should retries back off exponentially or linearly?".into(),
                section: Some("Open Questions".into()),
            },
        };
        let value: serde_json::Value = serde_json::to_value(&request).unwrap();
        // Typed field order: spec-path, spec-content, question.
        let keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, vec!["spec-path", "spec-content", "question"]);
        assert_eq!(value["question"]["section"], "Open Questions");
        assert_eq!(round_trip(&request), request);

        // `section` is omitted, not null, when unattributed.
        let bare = AskClarifyQuestionRequest {
            question: ClarifyQuestion {
                text: "q".into(),
                section: None,
            },
            ..request
        };
        let b_value: serde_json::Value = serde_json::to_value(&bare).unwrap();
        assert!(
            !b_value["question"]
                .as_object()
                .unwrap()
                .contains_key("section")
        );

        let response = AskClarifyQuestionResponse {
            answer: "Exponential, capped at 60s.".into(),
        };
        assert_eq!(round_trip(&response), response);
    }

    #[test]
    fn route_inbox_item_round_trip() {
        let request = RouteInboxItemRequest {
            item_text: "Bug: retry loop never backs off".into(),
            routes: ["rule", "spec", "scenario", "chore", "discard"]
                .iter()
                .map(ToString::to_string)
                .collect(),
            available_specs: vec![RouteInboxSpec {
                feature: "021-webhook-delivery".into(),
                status: "done".into(),
            }],
        };
        let value: serde_json::Value = serde_json::to_value(&request).unwrap();
        let keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, vec!["item-text", "routes", "available-specs"]);
        assert_eq!(
            value["available-specs"][0]["feature"],
            "021-webhook-delivery"
        );
        assert_eq!(round_trip(&request), request);

        let response = RouteInboxItemResponse {
            route: InboxRoute::Scenario,
            feature: Some("021-webhook-delivery".into()),
            reason: Some("Durable edge case.".into()),
        };
        let r_value: serde_json::Value = serde_json::to_value(&response).unwrap();
        assert_eq!(r_value["route"], "scenario");
        assert_eq!(round_trip(&response), response);
    }

    #[test]
    fn verify_criteria_round_trip() {
        let request = VerifyCriteriaRequest {
            spec_path: "specs/022-deterministic-runtime/spec.md".into(),
            spec_content: "# spec".into(),
            criteria: vec![
                VerifyCriterion {
                    index: 0,
                    text: "The walker completes.".into(),
                    checked: false,
                },
                VerifyCriterion {
                    index: 1,
                    text: "Out-of-boundary edits are rejected.".into(),
                    checked: true,
                },
            ],
        };
        let value: serde_json::Value = serde_json::to_value(&request).unwrap();
        // Typed field order: spec-path, spec-content, criteria.
        let keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, vec!["spec-path", "spec-content", "criteria"]);
        assert_eq!(value["criteria"][0]["index"], 0);
        assert_eq!(value["criteria"][1]["checked"], true);
        assert_eq!(round_trip(&request), request);

        let response = VerifyCriteriaResponse {
            results: vec![
                VerifyCriterionResult {
                    index: 0,
                    met: true,
                    note: None,
                },
                VerifyCriterionResult {
                    index: 1,
                    met: false,
                    note: Some("boundary rejection has no covering test yet".into()),
                },
            ],
        };
        let r_value: serde_json::Value = serde_json::to_value(&response).unwrap();
        // `note` is omitted, not null, when absent.
        assert!(
            !r_value["results"][0]
                .as_object()
                .unwrap()
                .contains_key("note")
        );
        assert_eq!(r_value["results"][1]["met"], false);
        assert_eq!(round_trip(&response), response);
    }

    #[test]
    fn validate_response_accepts_minimal_verify_criteria_payload() {
        use super::validate_response;
        validate_response(
            "verifyCriteria",
            &serde_json::json!({ "results": [ { "index": 0, "met": true } ] }),
        )
        .unwrap();
        // An empty verdict list is well-formed (a template-state spec has
        // zero criteria).
        validate_response("verifyCriteria", &serde_json::json!({ "results": [] })).unwrap();
    }

    #[test]
    fn validate_response_rejects_malformed_verify_criteria() {
        use super::{ValidationError, validate_response};
        // A verdict missing the required `met` field is a schema mismatch.
        let err = validate_response(
            "verifyCriteria",
            &serde_json::json!({ "results": [ { "index": 0 } ] }),
        )
        .unwrap_err();
        match err {
            ValidationError::Schema { identifier, source } => {
                assert_eq!(identifier, "verifyCriteria");
                assert!(source.to_string().contains("met"));
            }
            other => panic!("expected Schema, got {other:?}"),
        }
    }

    #[test]
    fn validate_response_accepts_minimal_clarify_and_route_payloads() {
        use super::validate_response;
        validate_response(
            "askClarifyQuestion",
            &serde_json::json!({ "answer": "Use exponential backoff." }),
        )
        .unwrap();
        // `feature` and `reason` are optional on a route decision.
        validate_response("routeInboxItem", &serde_json::json!({ "route": "chore" })).unwrap();
    }

    #[test]
    fn validate_response_rejects_unknown_inbox_route() {
        use super::{ValidationError, validate_response};
        // The route vocabulary is closed — a value outside the decision
        // tree's leaves is a schema mismatch, not a silent passthrough.
        let err = validate_response("routeInboxItem", &serde_json::json!({ "route": "shrug" }))
            .unwrap_err();
        match err {
            ValidationError::Schema { identifier, .. } => assert_eq!(identifier, "routeInboxItem"),
            other => panic!("expected Schema, got {other:?}"),
        }
    }
}
