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

/// Read one JSON-line envelope from `reader`. Returns
/// [`std::io::ErrorKind::UnexpectedEof`] when the reader is closed
/// without delivering a line.
///
/// # Errors
///
/// Bubbles I/O errors from `reader`. Malformed JSON surfaces as
/// [`std::io::Error::other`] — the host is responsible for delivering
/// well-formed envelopes per §json-over-stdio-framing.
pub fn read_envelope<R: BufRead>(reader: &mut R) -> std::io::Result<ProtocolMessage> {
    let mut line = String::new();
    let bytes = reader.read_line(&mut line)?;
    if bytes == 0 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::UnexpectedEof,
            "stdin closed before next envelope",
        ));
    }
    serde_json::from_str(line.trim())
        .map_err(|err| std::io::Error::other(format!("malformed envelope: {err}: {line:?}")))
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
    fn read_envelope_rejects_malformed_json() {
        let mut reader = Cursor::new("not-json\n".to_string());
        let err = read_envelope(&mut reader).unwrap_err();
        assert!(err.to_string().contains("malformed envelope"));
    }
}
