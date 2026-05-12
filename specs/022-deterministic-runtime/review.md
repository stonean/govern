---
spec: 022-deterministic-runtime
scenario: govern-bootstrap
reviewed-at: 2026-05-12T00:00:00Z
reviewed-against: 6fc0acc
diff-base: 6fc0acc~9
must-violations: 0
should-violations: 4
low-confidence: 2
skipped-passes: []
---

# Review — 022-deterministic-runtime (scenario: govern-bootstrap)

## Summary

Scenario `govern-bootstrap` adds four primitives (`fetch-archive`,
`extract-archive`, `substitute-templates`, `merge-claude-md`), extends
`gvrn exec`'s command-resolution surface to `framework/bootstrap/`,
rewrites the bootstrap procedure under the parseable conventions, and
ships a chain integration test plus unit coverage for each primitive.

No MUST violations: the shipped rule catalogs (`security-backend.md`,
`security-frontend.md`, `configuration.md`) target web-app patterns
(auth/sessions, XSS, cross-module config) that don't fire against
CLI/MCP primitive code. The runtime's own threat model (path
traversal in archive extraction, sha256-verified downloads) is
addressed in-code without rule-driven flagging.

Four SHOULD findings concern operational robustness and code reuse;
two low-confidence findings flag potential edge cases worth a second
look before the runtime ships v0.2.0 to real adopters. **Spec is
non-blocking and may advance to `done` after `/gov:validate`.**

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

### SHOULD: timeout-missing — fetch-archive has no HTTP timeout configured

- **File**: `runtime/src/primitives/fetch_archive.rs:74-88`
- **Pass**: efficiency
- **Rule**: (no shipped rule; framework convention from the
  CLI-baseline note in `AGENTS.md` Tech Stack — "fast cold-start,
  sensible exit codes")
- **Finding**: `reqwest::blocking::get(url)` uses reqwest's default
  client, which has **no connect or read timeout**. A slow or
  unresponsive URL will block the primitive indefinitely; the
  procedure has no way to recover short of process kill. For the
  bootstrap installer in particular, an adopter behind a flaky network
  could see `/govern` appear to hang.
- **Auto-fixable**: no (requires choosing a sensible default — 30s
  connect, 60s overall, configurable via `--timeout`?)
- **Suggested fix**: build a `reqwest::blocking::Client` once with
  `Client::builder().connect_timeout(Duration::from_secs(30)).timeout(Duration::from_secs(120)).build()`,
  cache it in a `OnceLock`, and use it for both `fetch_bytes` and
  `fetch_text`. Surface timeouts as `PrimitiveError::Http`.

### SHOULD: file-mode-not-preserved — extract-archive drops permission bits

- **File**: `runtime/src/primitives/extract_archive.rs:139-147` (tar.gz path), `:180-191` (zip path)
- **Pass**: quality (confidence 95)
- **Rule**: (no shipped rule; functional defect against the bootstrap's
  acceptance criteria — extracted scripts must be runnable)
- **Finding**: both `extract_tar_gz` and `extract_zip` write via
  `File::create` followed by `io::copy` and never call
  `fs::set_permissions` on the resulting file.
  The tar entry's `header().mode()` and the zip entry's `unix_mode()` are
  ignored. For the bootstrap path, this means executable scripts in
  `scripts/` (e.g., `scripts/gen-spec-deps.sh`) land with `0o644` instead
  of `0o755` and the pre-commit hook can't execute them — the adopter
  project bootstraps broken.
- **Auto-fixable**: yes (mechanical: `set_permissions` after the file
  closes; cfg-gate on `unix` for the bit set)
- **Suggested fix**: in `extract_tar_gz`, after the `io::copy`, call
  `entry.header().mode().ok()` and (on Unix) apply it via
  `fs::set_permissions(&safe, fs::Permissions::from_mode(mode))`. Same
  shape for `extract_zip` using `entry.unix_mode()`. Add a unit test
  that builds a tarball with `set_mode(0o755)`, extracts it, and asserts
  the extracted file's mode round-trips.

### SHOULD: duplicated-resolve-path — four primitives reimplement the same helper

- **File**: `runtime/src/primitives/fetch_archive.rs:62-68`,
  `runtime/src/primitives/extract_archive.rs:59-66`,
  `runtime/src/primitives/substitute_templates.rs:90-97`,
  `runtime/src/primitives/merge_claude_md.rs:71-78`
- **Pass**: reuse
- **Rule**: AGENTS.md `Workflow` — extract shared helpers when the same
  shape appears more than twice
- **Finding**: each of the four new primitives defines a private
  `resolve_path(repo, p)` function with identical logic ("if absolute
  return as-is, else `repo.join(p)`"). Other primitives in the same
  module already do this inline; the four bootstrap primitives forked
  the pattern into copy-paste helpers. Extract a single
  `pub(crate) fn resolve_path(repo: &Path, p: &str) -> PathBuf` in
  `primitives/mod.rs` next to `read_text` / `write_atomic_bytes` and
  call it from each primitive.
- **Auto-fixable**: yes
- **Suggested fix**: add the helper to `primitives/mod.rs` and remove
  the four local copies; no behavior change.

### SHOULD: redundant-drain — extract_zip drains entries that std::io::copy already consumed

- **File**: `runtime/src/primitives/extract_archive.rs:188-190`
- **Pass**: simplicity
- **Rule**: (no shipped rule; framework convention against dead code)
- **Finding**: after `std::io::copy(&mut entry, &mut out)` reads the
  zip entry to EOF, the subsequent `let _ = entry.read_to_end(&mut buf)`
  is a no-op (entry is already exhausted). The comment claims it's
  needed to advance the cursor, but `zip::ZipArchive::by_index(i)`
  reseeks per call — there's no shared cursor to advance. Remove the
  drain and its comment.
- **Auto-fixable**: yes (delete three lines)
- **Suggested fix**: drop lines 188-190 of `extract_archive.rs`.

## Low-confidence findings

### LOW: silent-truncation — fetch-archive surfaces oversized responses as a sha mismatch

- **File**: `runtime/src/primitives/fetch_archive.rs:84-90`
- **Pass**: quality (confidence 65)
- **Finding**: `response.take(MAX_FETCH_BYTES).read_to_end(&mut buf)`
  silently truncates the body when the response exceeds 256 MiB. The
  truncated body fails sha256 verification later, so the user sees a
  `ChecksumMismatch` error rather than "exceeded MAX_FETCH_BYTES." For
  the bootstrap workflow this is unlikely to fire (framework tarballs
  are ~50 KiB compressed), but the error message would mislead
  whoever does hit it.
- **Suggested fix**: check `response.content_length()` before
  reading; if present and over the limit, fail fast with a new
  `PrimitiveError::ArchiveTooLarge { expected, limit }` variant.
  Lower-priority if MAX_FETCH_BYTES never matters in practice.

### LOW: marker-substring-match — merge-claude-md uses unanchored substring search

- **File**: `runtime/src/primitives/merge_claude_md.rs:96-128`
- **Pass**: quality (confidence 50)
- **Finding**: `text.find(begin)` matches the BEGIN marker anywhere in
  the file — including inside fenced code blocks where an adopter
  might be quoting the marker for documentation purposes. If a code
  example happens to embed `<!-- BEGIN govern-managed -->` and `<!-- END govern-managed -->`,
  the primitive treats those quoted markers as the managed region and
  replaces the wrong block. Unlikely in practice but possible.
- **Suggested fix**: low-priority. If it ever matters, anchor markers
  to lines that contain *only* the marker (e.g., scan line-by-line and
  match on stripped equality rather than substring).

## Waived findings

*None.*

## Skipped passes

*None — all five passes ran.*

## Notes

- The shipped security rule files (`security-backend.md`,
  `security-frontend.md`) cover web-application threats that don't
  apply to a CLI / MCP server. The runtime's threat model — archive
  path traversal, supply-chain integrity (sha256 verification of
  downloads), unbounded resource use — is addressed in-code (see
  `safe_join` in `extract_archive.rs` and the `MAX_FETCH_BYTES` cap in
  `fetch_archive.rs`). A future spec adding a `framework/rules/security-cli.md`
  would let `/gov:review` flag these patterns automatically.
- The govern-basic parity fixture is recorded as deferred in
  `tasks.md`'s sub-task 26.8 note; the back-half chain test in
  `runtime/tests/exec_subprocess.rs` covers the post-fetch pipeline
  end-to-end. The full-procedure parity check waits on mock-HTTP
  support in the parity harness.
