---
section: "Follow-on scenarios"
---

# Runtime-review-019-hardening

## Context

The 2026-07-11 follow-up review of the 0.18.0 runtime (after the nine-MUST/twenty-one-SHOULD remediation) surfaced a partially-resolved security finding, a regression the 0.18.0 parser change introduced, and a set of newly-found input-validation and correctness gaps. Groomed here as one hardening pass (shipped in gvrn 0.19.0):

- **SSRF was only half-closed**: `fetch-archive` validated the initial URL but used the default `reqwest` client, which follows up to ten redirects with no re-validation — a single `302` to `http://169.254.169.254/…` defeated both the https-only rule and the internal-range denial.
- **Parse regression**: the 0.18.0 two-primitive hard-error rejected `framework/bootstrap/govern.md` step 6 (which named `merge-managed-block` + `write-session`), so `gvrn exec govern` no longer parsed — and the parseability lint scanned only `framework/commands/*.md`, so CI missed it.
- **`feature`-argument traversal**: the writers `set-status` / `mark-task` / `mark-criterion` / `prune-tasks` / `write-review` and the read-only feature primitives joined the MCP-supplied `feature` into a path with no containment check, an out-of-repo read/write escape.
- **`write-review` frontmatter injection**: `reviewed-at` / `reviewed-against` / `diff-base` / `feature` / `scenario` / `skipped-passes` were spliced raw into `review.md` and the spec `review:` block, so a newline could inject a spoofed top-level key (e.g. `status: done`).
- **Exec-path correctness**: `gvrn exec target <feature>` kept the stale session target (the seeded-key guard blocked `resolve-feature`'s resolved value); the writeCode content contract (`create` needs `content`, `edit` needs `patch`/`content`) was documented but unenforced; operational-error exits (command-not-found, unreadable file, walker I/O) emitted no terminal `error` envelope, breaking the 1–127 clean-band contract; the parser opened Instructions on a text-less heading, dropped a stray extension marker, and mistook a marker quoted in a code span for a live seam.
- **Lesser hardening**: `run-generator` / `lint-markdown` ran uncontained path/flag arguments; `discover-rule-files` accepted unvalidated `detected-surfaces`; `mark-task` / `read-tasks` treated a dot-less prose heading as a task; `write_atomic` narrowed an existing file's mode to `0600`; `merge-managed-block`'s `html-comment` style churned `CLAUDE.md` on CRLF checkouts and left its `marker` unvalidated.

## Behavior

- `fetch-archive` re-runs the full scheme/internal-range screen on every redirect hop via a custom `reqwest` redirect policy capped at ten hops; the internal-range denial also covers `0.0.0.0/8` and `100.64.0.0/10`.
- `framework/bootstrap/govern.md` step 6 is split into two single-primitive steps so it parses under the two-primitive hard-error, and `scripts/lint-procedure-parseability.sh` parses `framework/bootstrap/*.md` alongside `framework/commands/*.md`.
- Every primitive that joins the MCP-supplied `feature` into a path validates it with `validate_no_traversal`.
- `write-review` single-line-validates the frontmatter scalar fields before any write; multi-line prose body fields are unaffected.
- `gvrn exec target` lets a `resolve-feature` `resolved` result override the seeded `feature`/`path`; `validate_response` rejects a `create` edit with no `content` and an `edit` edit with neither `patch` nor `content`; the operational-error exit paths emit a terminal `error` envelope carrying the runtime version; the parser opens Instructions only on a heading that emits the text `Instructions`, attaches a between-steps marker to the next step, and ignores a marker quoted in a code span.
- `run-generator` bounds its `script` to the repo and `lint-markdown` rejects a `-`-leading `paths` entry; `discover-rule-files` validates `detected-surfaces` members; `mark-task` / `read-tasks` require the trailing `.` on a task heading; `write_atomic` re-applies an existing destination's mode on Unix; `merge-managed-block` normalizes `\r` before its unchanged-compare and rejects a newline- or `-->`-bearing `marker`.

## Edge Cases

- The markdown-only path is unchanged; `GVRN_FETCH_ALLOW_INSECURE_HOSTS` still exempts named hosts (initial URL and every redirect hop).
- An empty derived boundary and the DNS-rebinding TOCTOU are explicitly out of scope here (see [writecode-boundary-derivation](writecode-boundary-derivation.md) and the inbox item, respectively).

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None — this scenario documents work already shipped in gvrn 0.19.0.*
