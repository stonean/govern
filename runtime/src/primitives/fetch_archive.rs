//! `fetch-archive` — download an archive and, optionally, verify its
//! sha256 against a sidecar file.
//!
//! The procedural use case is the `/govern` bootstrap installer
//! (scenario `govern-bootstrap` on spec 022). The framework operates
//! live-on-main, so the bootstrap fetches GitHub's auto-generated
//! source tarball (`/archive/refs/heads/main.tar.gz`), which has no
//! companion sha256 sidecar. Other procedural callers (release-asset
//! installers, future runtime auto-update) do have sidecars and want
//! verification.
//!
//! Behavior:
//!
//! - `sha256_url = Some(_)`: download the archive, fetch the sidecar,
//!   parse its leading hex token, compare against the computed sha. A
//!   mismatch raises [`PrimitiveError::ChecksumMismatch`] *before* the
//!   archive is written — nothing lands on disk. The error's `path`
//!   names the destination the archive would have been written to
//!   (kept for message compatibility and retry context); that file is
//!   never created.
//! - `sha256_url = None`: download the archive, compute the sha256,
//!   return it in the result with `verified: false`. The host can
//!   compare against a known-good digest out-of-band if it cares.
//!
//! Either way the result includes the computed digest and a
//! `verified: bool` so the host knows whether to trust it.
//!
//! Response bodies are capped at `MAX_FETCH_BYTES`. A body that exceeds
//! the cap raises an operational error naming the cap and the archive is
//! not written — never a silent truncation, which would otherwise carry
//! a self-consistent computed sha and only surface as corruption later
//! in `extract-archive`. A body of exactly the cap size succeeds.
//!
//! The primitive is blocking — it spins reqwest's blocking client per
//! call rather than dragging an async runtime through the primitive
//! surface. Network failures map to [`PrimitiveError::Http`] (transport)
//! or [`PrimitiveError::HttpStatus`] (non-2xx).
//!
//! The sidecar format matches `shasum -a 256` output: one or more lines
//! shaped `<hex>  <filename>`. Only the first hex digest is consulted;
//! filename column is informational.

use std::io::Read;
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use crate::primitives::{PrimitiveError, Result, resolve_path, write_atomic_bytes};
use crate::schema::primitives::{FetchArchiveArgs, FetchArchiveResult};

/// Execute the `fetch-archive` primitive.
///
/// # Errors
///
/// - [`PrimitiveError::Http`] / [`PrimitiveError::HttpStatus`] on network failure.
/// - [`PrimitiveError::Io`] on local filesystem failures writing the archive,
///   or when the response body exceeds the `MAX_FETCH_BYTES` fetch cap
///   (the error message names the cap; the archive is not written).
/// - [`PrimitiveError::MalformedSidecar`] when the sidecar's first hex token isn't a valid 64-char sha256.
/// - [`PrimitiveError::ChecksumMismatch`] when the computed sha differs from
///   the sidecar. Raised before the archive is written; the error's `path`
///   is the intended destination, which is never created.
pub fn run(args: &FetchArchiveArgs, repo: &Path) -> Result<FetchArchiveResult> {
    let dest = resolve_path(repo, &args.archive);

    let body = fetch_bytes(&args.url)?;
    let computed = sha256_hex(&body);

    let verified = match &args.sha256_url {
        Some(sidecar_url) => {
            let sidecar = fetch_text(sidecar_url)?;
            let expected = parse_sidecar_hex(&sidecar, sidecar_url)?;
            if computed != expected {
                // Fail before write: `path` carries the *intended*
                // destination (kept for error-message compatibility and
                // retry context), but no file is created there.
                return Err(PrimitiveError::ChecksumMismatch {
                    path: dest,
                    expected,
                    actual: computed,
                });
            }
            true
        }
        None => false,
    };

    let bytes = u64::try_from(body.len()).unwrap_or(u64::MAX);
    write_atomic_bytes(&dest, &body)?;

    Ok(FetchArchiveResult {
        path: dest.to_string_lossy().into_owned(),
        sha256: computed,
        verified,
        bytes,
    })
}

fn fetch_bytes(url: &str) -> Result<Vec<u8>> {
    let response = reqwest::blocking::get(url).map_err(|source| PrimitiveError::Http {
        url: url.into(),
        source,
    })?;
    let status = response.status();
    if !status.is_success() {
        return Err(PrimitiveError::HttpStatus {
            url: url.into(),
            status: status.as_u16(),
        });
    }
    read_capped(response, MAX_FETCH_BYTES, url)
}

/// Read at most `cap` bytes from `reader`, erroring when the stream holds
/// more. Reads `cap + 1` bytes so an over-cap body is *detected* and
/// rejected with an error naming the cap, rather than silently truncated
/// (a truncated archive would carry a self-consistent computed sha and
/// only surface as corruption later). A stream of exactly `cap` bytes
/// succeeds.
fn read_capped(reader: impl Read, cap: u64, url: &str) -> Result<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();
    reader
        .take(cap.saturating_add(1))
        .read_to_end(&mut buf)
        .map_err(|source| PrimitiveError::Io {
            path: PathBuf::from(url),
            source,
        })?;
    if u64::try_from(buf.len()).unwrap_or(u64::MAX) > cap {
        return Err(PrimitiveError::Io {
            path: PathBuf::from(url),
            source: std::io::Error::other(format!(
                "response body exceeds the fetch cap of {cap} bytes; archive not written"
            )),
        });
    }
    Ok(buf)
}

fn fetch_text(url: &str) -> Result<String> {
    let bytes = fetch_bytes(url)?;
    String::from_utf8(bytes).map_err(|err| PrimitiveError::MalformedSidecar {
        url: url.into(),
        reason: format!("not valid UTF-8: {err}"),
    })
}

/// Parse the first `<hex>  <filename>` line of a `shasum -a 256` sidecar.
///
/// # Errors
///
/// Returns [`PrimitiveError::MalformedSidecar`] when the body has no parsable
/// line or the leading token isn't a 64-char lowercase-hex string.
pub(crate) fn parse_sidecar_hex(body: &str, url: &str) -> Result<String> {
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let token = trimmed.split_whitespace().next().unwrap_or("");
        if is_sha256_hex(token) {
            return Ok(token.to_ascii_lowercase());
        }
        return Err(PrimitiveError::MalformedSidecar {
            url: url.into(),
            reason: format!("leading token `{token}` is not 64-char lowercase hex"),
        });
    }
    Err(PrimitiveError::MalformedSidecar {
        url: url.into(),
        reason: "sidecar body is empty or has no non-comment line".into(),
    })
}

fn is_sha256_hex(token: &str) -> bool {
    token.len() == 64
        && token
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c) || ('A'..='F').contains(&c))
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hasher.finalize();
    let mut out = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut out, "{byte:02x}");
    }
    out
}

/// Cap the per-fetch body size to ~256 MiB. Framework release tarballs
/// are well under 50 MiB; this ceiling defends against accidentally
/// streaming an unbounded URL into memory. A body exceeding the cap is
/// an operational error (see [`read_capped`]), never a silent truncation.
const MAX_FETCH_BYTES: u64 = 256 * 1024 * 1024;

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use std::io::Cursor;

    use super::{is_sha256_hex, parse_sidecar_hex, read_capped, sha256_hex};

    #[test]
    fn sha256_hex_matches_known_vector() {
        // sha256("abc") = ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad
        assert_eq!(
            sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn parse_sidecar_extracts_leading_hex() {
        let sidecar =
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  release.tar.gz\n";
        let got = parse_sidecar_hex(sidecar, "http://t").unwrap();
        assert_eq!(
            got,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn parse_sidecar_lowercases_hex() {
        let sidecar =
            "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD  release.tar.gz";
        let got = parse_sidecar_hex(sidecar, "http://t").unwrap();
        assert!(got.chars().all(|c| !c.is_ascii_uppercase()));
    }

    #[test]
    fn parse_sidecar_skips_comments_and_blanks() {
        let sidecar = "# this is a comment\n\nba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad  release.tar.gz\n";
        let got = parse_sidecar_hex(sidecar, "http://t").unwrap();
        assert!(got.starts_with("ba7816"));
    }

    #[test]
    fn parse_sidecar_rejects_short_token() {
        let err = parse_sidecar_hex("abc123  release.tar.gz", "http://t").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("malformed"), "got: {msg}");
    }

    #[test]
    fn parse_sidecar_rejects_empty_body() {
        let err = parse_sidecar_hex("", "http://t").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("empty"), "got: {msg}");
    }

    #[test]
    fn read_capped_accepts_body_under_cap() {
        let got = read_capped(Cursor::new(vec![7u8; 4]), 8, "http://t").unwrap();
        assert_eq!(got, vec![7u8; 4]);
    }

    #[test]
    fn read_capped_accepts_body_of_exactly_cap_size() {
        let got = read_capped(Cursor::new(vec![7u8; 8]), 8, "http://t").unwrap();
        assert_eq!(got.len(), 8);
    }

    #[test]
    fn read_capped_rejects_body_exceeding_cap_naming_the_cap() {
        let err = read_capped(Cursor::new(vec![7u8; 9]), 8, "http://t").unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("fetch cap of 8 bytes"),
            "error must name the cap; got: {msg}"
        );
    }

    #[test]
    fn is_sha256_hex_recognizes_valid_digests() {
        assert!(is_sha256_hex(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        ));
        assert!(is_sha256_hex(
            "BA7816BF8F01CFEA414140DE5DAE2223B00361A396177A9CB410FF61F20015AD"
        ));
        assert!(!is_sha256_hex("ba7816bf"));
        assert!(!is_sha256_hex(
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015agz"
        ));
    }
}
