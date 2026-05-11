//! `gate-confirm` — surface a pipeline gate and await the user's decision.
//!
//! Behavior differs by surface (see §runtime-boundary):
//!
//! - **Subprocess interpreter / CLI**: emit a [`ProtocolMessage::GateConfirm`]
//!   envelope on `writer`, then read one JSON line from `reader` and parse
//!   it as a [`ProtocolMessage::GateResponse`] with a matching `request-id`.
//!   The response's `confirmed` bool is the result. Implemented by
//!   [`run_blocking`].
//! - **MCP surface**: the tool returns `{ "gate", "prompt", "request-id" }`
//!   unchanged; the MCP client routes the prompt to the user and calls
//!   back via a separate tool. Implemented by [`prompt_payload`] (task 6
//!   wires this into the MCP server's tool description).
//!
//! The `request-id` is supplied by the caller — the interpreter generates
//! one per step, and the CLI binding generates a fresh one per invocation.
//! Keeping id allocation outside the primitive avoids shared mutable state
//! and lets correlate-back logic live where it belongs (the orchestrator).

use std::io::{BufRead, Write};
use std::sync::atomic::{AtomicU64, Ordering};

use crate::primitives::{PrimitiveError, Result};
use crate::schema::primitives::{GateConfirmArgs, GateConfirmResult};
use crate::schema::protocol::ProtocolMessage;

/// Run `gate-confirm` against an explicit reader/writer pair using the
/// caller-supplied `request_id` as the correlation token. The CLI binding
/// passes `io::stdin().lock()` and `io::stdout().lock()`; tests pass byte
/// buffers and fixed ids.
///
/// # Errors
///
/// Returns [`PrimitiveError::Io`] on read/write failures or when the
/// response is not a well-formed JSON envelope. Returns a wrapped I/O
/// error when the response has a mismatched `request-id` or arrives as a
/// type other than `gate-response`.
pub fn run_blocking<R, W>(
    args: &GateConfirmArgs,
    request_id: &str,
    reader: &mut R,
    writer: &mut W,
) -> Result<GateConfirmResult>
where
    R: BufRead,
    W: Write,
{
    let envelope = ProtocolMessage::GateConfirm {
        gate: args.gate.clone(),
        request_id: request_id.into(),
        prompt: args.prompt.clone(),
    };
    let serialized = serde_json::to_string(&envelope).map_err(|err| PrimitiveError::Io {
        path: std::path::PathBuf::from("<stdout>"),
        source: std::io::Error::other(err),
    })?;
    writeln!(writer, "{serialized}").map_err(|source| PrimitiveError::Io {
        path: std::path::PathBuf::from("<stdout>"),
        source,
    })?;
    writer.flush().map_err(|source| PrimitiveError::Io {
        path: std::path::PathBuf::from("<stdout>"),
        source,
    })?;

    let mut line = String::new();
    let bytes = reader
        .read_line(&mut line)
        .map_err(|source| PrimitiveError::Io {
            path: std::path::PathBuf::from("<stdin>"),
            source,
        })?;
    if bytes == 0 {
        return Err(PrimitiveError::Io {
            path: std::path::PathBuf::from("<stdin>"),
            source: std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "stdin closed before gate-response arrived",
            ),
        });
    }
    let parsed: ProtocolMessage =
        serde_json::from_str(line.trim()).map_err(|err| PrimitiveError::Io {
            path: std::path::PathBuf::from("<stdin>"),
            source: std::io::Error::other(err),
        })?;
    match parsed {
        ProtocolMessage::GateResponse {
            request_id: response_id,
            confirmed,
        } if response_id == request_id => Ok(GateConfirmResult { confirmed }),
        ProtocolMessage::GateResponse {
            request_id: other, ..
        } => Err(PrimitiveError::Io {
            path: std::path::PathBuf::from("<stdin>"),
            source: std::io::Error::other(format!(
                "gate-response request-id mismatch: expected {request_id}, got {other}",
            )),
        }),
        _ => Err(PrimitiveError::Io {
            path: std::path::PathBuf::from("<stdin>"),
            source: std::io::Error::other("expected gate-response envelope"),
        }),
    }
}

/// Build the prompt payload returned by the MCP tool surface. The MCP tool
/// does not block; it returns this payload, and the client routes the
/// prompt to the user and calls a follow-up tool to return the user's
/// decision. The shape mirrors the `gate-confirm` envelope minus `type`.
#[must_use]
pub fn prompt_payload(args: &GateConfirmArgs, request_id: &str) -> GatePromptPayload {
    GatePromptPayload {
        gate: args.gate.clone(),
        prompt: args.prompt.clone(),
        request_id: request_id.into(),
    }
}

/// Generate a fresh per-process `request-id` (`gate-<N>`) suitable for the
/// CLI binding. Higher-level interpreters typically derive ids from their
/// own walker state instead of using this helper.
pub fn fresh_request_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    format!("gate-{n}")
}

/// MCP-surface response: an opaque `request-id` plus the args echoed back.
/// The MCP client is responsible for routing `prompt` to the user and
/// returning the answer on a separate tool call.
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct GatePromptPayload {
    /// Named gate (echoed from args).
    pub gate: String,
    /// Prompt to surface to the user (echoed from args).
    pub prompt: String,
    /// Correlator the client must echo on the follow-up tool call.
    pub request_id: String,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::io::Cursor;

    fn args() -> GateConfirmArgs {
        GateConfirmArgs {
            gate: "plan-finalize-status".into(),
            prompt: "Advance status from clarified to planned?".into(),
        }
    }

    #[test]
    fn emits_envelope_and_reads_matching_response() {
        let response = "{\"type\":\"gate-response\",\"request-id\":\"req-1\",\"confirmed\":true}\n";
        let mut writer: Vec<u8> = Vec::new();
        let mut reader = Cursor::new(response.to_string());
        let result = run_blocking(&args(), "req-1", &mut reader, &mut writer).unwrap();
        assert!(result.confirmed);

        let emitted = String::from_utf8(writer).unwrap();
        let envelope: serde_json::Value = serde_json::from_str(emitted.trim()).unwrap();
        assert_eq!(envelope["type"], "gate-confirm");
        assert_eq!(envelope["gate"], "plan-finalize-status");
        assert_eq!(envelope["request-id"], "req-1");
    }

    #[test]
    fn user_denial_returns_confirmed_false() {
        let response =
            "{\"type\":\"gate-response\",\"request-id\":\"req-2\",\"confirmed\":false}\n";
        let mut writer: Vec<u8> = Vec::new();
        let mut reader = Cursor::new(response.to_string());
        let result = run_blocking(&args(), "req-2", &mut reader, &mut writer).unwrap();
        assert!(!result.confirmed);
    }

    #[test]
    fn mismatched_request_id_is_error() {
        let response =
            "{\"type\":\"gate-response\",\"request-id\":\"wrong-id\",\"confirmed\":true}\n";
        let mut writer: Vec<u8> = Vec::new();
        let mut reader = Cursor::new(response.to_string());
        let err = run_blocking(&args(), "expected-id", &mut reader, &mut writer).unwrap_err();
        match err {
            PrimitiveError::Io { source, .. } => {
                assert!(source.to_string().contains("request-id mismatch"));
            }
            other => panic!("expected Io error, got {other:?}"),
        }
    }

    #[test]
    fn closed_stdin_is_error() {
        let mut writer: Vec<u8> = Vec::new();
        let mut reader = Cursor::new(String::new());
        let err = run_blocking(&args(), "req-3", &mut reader, &mut writer).unwrap_err();
        match err {
            PrimitiveError::Io { source, .. } => {
                assert_eq!(source.kind(), std::io::ErrorKind::UnexpectedEof);
            }
            other => panic!("expected Io error, got {other:?}"),
        }
    }

    #[test]
    fn prompt_payload_echoes_args() {
        let p = prompt_payload(&args(), "req-9");
        assert_eq!(p.gate, "plan-finalize-status");
        assert!(p.prompt.starts_with("Advance status"));
        assert_eq!(p.request_id, "req-9");
    }

    #[test]
    fn fresh_request_id_yields_unique_ids() {
        let a = fresh_request_id();
        let b = fresh_request_id();
        assert_ne!(a, b);
        assert!(a.starts_with("gate-"));
        assert!(b.starts_with("gate-"));
    }
}
