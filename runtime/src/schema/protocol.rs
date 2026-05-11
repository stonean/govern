//! JSON-over-stdio protocol envelope and message types.
//!
//! Line-delimited JSON; one complete object per line, terminated by `\n`. The
//! envelope below is the closed protocol surface — adding a discriminator
//! variant requires a runtime major-version bump per §runtime-boundary.

#![allow(clippy::module_name_repetitions)]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// File-location annotation attached to error envelopes.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct ErrorLocation {
    /// Repo-relative file path.
    pub file: String,
    /// 1-based line.
    pub line: u32,
    /// 1-based column.
    pub col: u32,
}

/// The protocol envelope. Each variant serializes as a JSON object with a
/// `"type"` discriminator and the variant's fields alongside it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum ProtocolMessage {
    /// Runtime → host: invoke an LLM extension point and await response.
    LlmRequest {
        /// Extension-point identifier (e.g., "writeCode").
        #[serde(rename = "extension-point")]
        extension_point: String,
        /// Opaque correlator the host echoes back in `llm-response`.
        #[serde(rename = "request-id")]
        request_id: String,
        /// Extension-point request payload (shape depends on `extension-point`).
        request: Value,
    },
    /// Host → runtime: response payload for an open `llm-request`.
    LlmResponse {
        /// Matches an open `llm-request`.
        #[serde(rename = "request-id")]
        request_id: String,
        /// Extension-point response payload.
        response: Value,
    },
    /// Runtime → host: surface a pipeline gate to the user.
    GateConfirm {
        /// Named gate (e.g., "plan-finalize-status").
        gate: String,
        /// Opaque correlator the host echoes back in `gate-response`.
        #[serde(rename = "request-id")]
        request_id: String,
        /// Prompt shown to the user via the host.
        prompt: String,
    },
    /// Host → runtime: response to an open `gate-confirm`.
    GateResponse {
        /// Matches an open `gate-confirm`.
        #[serde(rename = "request-id")]
        request_id: String,
        /// User's decision.
        confirmed: bool,
    },
    /// Runtime → host: non-blocking informational signal.
    Progress {
        /// Human-readable progress text.
        message: String,
        /// Step number being executed (e.g., "3.1"), if applicable.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        step: Option<String>,
        /// Primitive currently dispatched, if applicable.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        primitive: Option<String>,
    },
    /// Runtime → host: procedure finished cleanly. Followed by exit code 0.
    Complete {
        /// Result payload (shape depends on the command).
        result: Value,
        /// Runtime binary version (`CARGO_PKG_VERSION`).
        #[serde(rename = "runtime-version")]
        runtime_version: String,
    },
    /// Runtime → host: procedure halted on operational error.
    Error {
        /// Machine-readable error code (e.g., "parse-error").
        code: String,
        /// Human-readable description.
        message: String,
        /// Runtime binary version (`CARGO_PKG_VERSION`).
        #[serde(rename = "runtime-version")]
        runtime_version: String,
        /// Source location, when available.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        location: Option<ErrorLocation>,
    },
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::{ErrorLocation, ProtocolMessage};
    use serde_json::json;

    #[test]
    fn round_trip_llm_request() {
        let original = ProtocolMessage::LlmRequest {
            extension_point: "writeCode".into(),
            request_id: "abc-1".into(),
            request: json!({"task": {"number": "1"}}),
        };
        let text = serde_json::to_string(&original).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["type"], "llm-request");
        assert_eq!(value["extension-point"], "writeCode");
        let parsed: ProtocolMessage = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn round_trip_llm_response() {
        let original = ProtocolMessage::LlmResponse {
            request_id: "abc-1".into(),
            response: json!({"passed": true, "finding": null}),
        };
        let text = serde_json::to_string(&original).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["type"], "llm-response");
        assert_eq!(value["request-id"], "abc-1");
        let parsed: ProtocolMessage = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn round_trip_gate_confirm_and_response() {
        let confirm = ProtocolMessage::GateConfirm {
            gate: "plan-finalize-status".into(),
            request_id: "g-1".into(),
            prompt: "Advance status from clarified to planned?".into(),
        };
        let confirm_text = serde_json::to_string(&confirm).unwrap();
        let confirm_value: serde_json::Value = serde_json::from_str(&confirm_text).unwrap();
        assert_eq!(confirm_value["type"], "gate-confirm");
        let confirm_parsed: ProtocolMessage = serde_json::from_str(&confirm_text).unwrap();
        assert_eq!(confirm_parsed, confirm);

        let response = ProtocolMessage::GateResponse {
            request_id: "g-1".into(),
            confirmed: true,
        };
        let response_text = serde_json::to_string(&response).unwrap();
        let response_value: serde_json::Value = serde_json::from_str(&response_text).unwrap();
        assert_eq!(response_value["type"], "gate-response");
        let response_parsed: ProtocolMessage = serde_json::from_str(&response_text).unwrap();
        assert_eq!(response_parsed, response);
    }

    #[test]
    fn round_trip_progress_omits_optional_fields() {
        let msg = ProtocolMessage::Progress {
            message: "walking task 1".into(),
            step: None,
            primitive: None,
        };
        let value: serde_json::Value = serde_json::to_value(&msg).unwrap();
        let object = value.as_object().unwrap();
        assert_eq!(object["type"], "progress");
        assert!(!object.contains_key("step"));
        assert!(!object.contains_key("primitive"));

        let with_fields = ProtocolMessage::Progress {
            message: "walking task 1".into(),
            step: Some("3.1".into()),
            primitive: Some("mark-task".into()),
        };
        let text = serde_json::to_string(&with_fields).unwrap();
        let parsed: ProtocolMessage = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed, with_fields);
    }

    #[test]
    fn round_trip_complete() {
        let original = ProtocolMessage::Complete {
            result: json!({"ok": true}),
            runtime_version: env!("CARGO_PKG_VERSION").into(),
        };
        let text = serde_json::to_string(&original).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["type"], "complete");
        assert!(value.get("runtime-version").is_some());
        let parsed: ProtocolMessage = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn round_trip_error_with_location() {
        let original = ProtocolMessage::Error {
            code: "parse-error".into(),
            message: "unexpected token".into(),
            runtime_version: env!("CARGO_PKG_VERSION").into(),
            location: Some(ErrorLocation {
                file: "framework/commands/status.md".into(),
                line: 12,
                col: 4,
            }),
        };
        let text = serde_json::to_string(&original).unwrap();
        let value: serde_json::Value = serde_json::from_str(&text).unwrap();
        assert_eq!(value["type"], "error");
        assert_eq!(value["code"], "parse-error");
        let parsed: ProtocolMessage = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed, original);
    }
}
