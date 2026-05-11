//! Initial-release LLM extension-point payload schemas.
//!
//! Mirrors the request/response shapes in
//! `specs/022-deterministic-runtime/data-model.md`. The runtime emits these
//! as the `request` field of `llm-request` envelopes and validates incoming
//! `llm-response` payloads against them.

#![allow(clippy::module_name_repetitions)]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct WriteCodeRequest {
    /// Task being implemented.
    pub task: WriteCodeTask,
    /// Files the plan named as relevant for this task.
    pub plan_relevant_files: Vec<PlanRelevantFile>,
    /// Runtime write boundary (glob patterns and concrete paths).
    pub write_boundary: Vec<String>,
    /// Constitution excerpts the runtime determined are relevant.
    pub constitution_excerpts: Vec<String>,
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::{
        AssessSpecQualityFinding, AssessSpecQualityRequest, AssessSpecQualityResponse,
        AssessSpecQualityRule, FindingLocation, PlanRelevantFile, WriteCodeAction, WriteCodeEdit,
        WriteCodeRequest, WriteCodeResponse, WriteCodeTask, WriteSpecBodyRequest,
        WriteSpecBodyResponse,
    };

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
}
