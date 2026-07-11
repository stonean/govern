//! Stdio framing helpers for protocol messages.
//!
//! The JSON-over-stdio protocol is line-delimited: every envelope is a
//! single complete JSON object terminated by `\n`. These helpers are the
//! single place where serialization, flushing, and EOF semantics live so
//! the [`crate::interpreter::Walker`] and the `runtime exec` driver stay
//! decoupled from the framing choice.

use std::io::{BufRead, Write};

use crate::schema::protocol::ProtocolMessage;

/// Serialize `message` as a single JSON line and flush. Returns an I/O
/// error if serialization or the underlying write fails.
///
/// # Errors
///
/// Bubbles I/O errors from `writer`. Serialization errors are converted
/// to [`std::io::Error::other`].
pub fn write_envelope<W: Write>(writer: &mut W, message: &ProtocolMessage) -> std::io::Result<()> {
    let serialized = serde_json::to_string(message)
        .map_err(|err| std::io::Error::other(format!("failed to serialize envelope: {err}")))?;
    writeln!(writer, "{serialized}")?;
    writer.flush()
}

/// Read the next well-formed JSON-line envelope from `reader`.
///
/// Per the protocol's robustness rule (data-model §JSON-over-stdio: "the
/// runtime ignores any other inbound JSON shape — it logs to stderr and
/// continues waiting"), a blank keepalive line or a line that fails to
/// parse as an envelope is logged to stderr and skipped; reading
/// continues until a valid envelope arrives. Only EOF while awaiting an
/// envelope is an operational error.
///
/// # Errors
///
/// Bubbles I/O errors from `reader`. Returns
/// [`std::io::ErrorKind::UnexpectedEof`] when the reader closes before
/// delivering a valid envelope.
pub fn read_envelope<R: BufRead>(reader: &mut R) -> std::io::Result<ProtocolMessage> {
    loop {
        let mut line = String::new();
        let bytes = reader.read_line(&mut line)?;
        if bytes == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "stdin closed before next envelope",
            ));
        }
        let trimmed = line.trim();
        if trimmed.is_empty() {
            eprintln!("runtime: ignoring blank stdin line while awaiting an envelope");
            continue;
        }
        match serde_json::from_str(trimmed) {
            Ok(message) => return Ok(message),
            Err(err) => {
                eprintln!("runtime: ignoring unparseable stdin line ({err}): {trimmed:?}");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::io::Cursor;

    #[test]
    fn round_trip_progress_envelope() {
        let msg = ProtocolMessage::Progress {
            message: "hello".into(),
            step: Some("1".into()),
            primitive: None,
        };
        let mut writer: Vec<u8> = Vec::new();
        write_envelope(&mut writer, &msg).unwrap();
        let mut reader = Cursor::new(writer);
        let parsed = read_envelope(&mut reader).unwrap();
        assert_eq!(parsed, msg);
    }

    #[test]
    fn read_envelope_surfaces_eof_on_empty_reader() {
        let mut reader = Cursor::new(String::new());
        let err = read_envelope(&mut reader).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
    }

    #[test]
    fn read_envelope_skips_malformed_and_blank_lines_until_valid() {
        let valid = "{\"type\":\"gate-response\",\"request-id\":\"g-1\",\"confirmed\":true}\n";
        let mut reader = Cursor::new(format!("not-json\n\n{{\"half\": \n{valid}"));
        let parsed = read_envelope(&mut reader).unwrap();
        assert_eq!(
            parsed,
            ProtocolMessage::GateResponse {
                request_id: "g-1".into(),
                confirmed: true,
            }
        );
    }

    #[test]
    fn read_envelope_surfaces_eof_after_only_malformed_lines() {
        // Malformed lines are skipped, not fatal; the reader then closing
        // without a valid envelope is the operational error.
        let mut reader = Cursor::new("not-json\n".to_string());
        let err = read_envelope(&mut reader).unwrap_err();
        assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
    }
}
