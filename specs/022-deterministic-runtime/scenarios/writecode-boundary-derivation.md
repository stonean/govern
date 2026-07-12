---
section: "Follow-on scenarios"
---

# Writecode-boundary-derivation

## Context

On the `/gov:implement` exec path the writeCode validator enforces edits against the `write-boundary` context key, but nothing derives into that key. `derive-boundary` (step 2) emits its result under `boundary`; only the informational `first-commit` / `current-head` are threaded into the writeCode request. So `write-boundary` is populated only by a session seed — with no seed, every edit is rejected (fail-closed), and a freshly derived boundary is never used for enforcement.

A naive fix (bind `derive-boundary`'s `boundary` → `write-boundary`) breaks the `implement-basic` parity fixture: it is a single-commit git repo, so `git diff <first-commit>..HEAD` is empty and `derive-boundary` yields an empty (or edit-excluding) boundary. The fixture hand-seeds `["specs/004-implement/**", "runtime/**"]` precisely to give the writeCode enforcement a realistic boundary the canned edits satisfy. An empty-guarded override still fails, because the single-commit derivation is non-empty-but-wrong for the canned edits, so the golden re-blesses to an out-of-boundary error and the parity success assert fails. The fix therefore needs both a precedence decision and a fixture that exercises a real derivation.

## Behavior

The write boundary the runtime derives during `/gov:implement` populates the key the writeCode validator enforces on, so enforcement uses what the run actually computed rather than depending on a pre-seeded `write-boundary`. A seeded boundary remains the fallback when the derivation is empty (a spec dir with no changes since its first commit), so the fail-closed case is a deliberate seed rather than an accident.

The `implement-basic` parity fixture gains a multi-commit history (an initial commit, then a commit touching the feature's spec dir and the edited paths) so `derive-boundary` produces a non-empty boundary that matches the canned writeCode edits; the golden is re-blessed against it.

## Edge Cases

- Empty derivation → fall back to any seeded `write-boundary`; with neither, enforcement stays fail-closed (no edit permitted) and the walk halts with `out-of-boundary-edit` rather than writing anywhere.
- A seed that is *wider* than the derivation: the derived (narrower, authoritative) boundary wins.
- The markdown-only path is unaffected — the host derives and enforces the boundary itself per the prose.

## Open Questions

- Precedence when both a non-empty derivation and a seed are present: does the derivation always win, or only when the two disagree in a way worth surfacing?

**Implementation finding (2026-07-12) — boundary-format mismatch, reframes this scenario.** `derive-boundary` emits the **exact paths** of files changed since the spec dir's first commit (`derive_boundary.rs:61`, e.g. `runtime/src/main.rs`), plus the single `specs/{feature}/**` glob. But the writeCode enforcement treats a non-glob boundary entry as an **exact-path match** (`/**` = any descendant, `/*` = direct children, otherwise exact). writeCode's job is to **create/edit files**, including *new* files not yet in git history — and a new file (`runtime/src/foo.rs`) does not exact-match any previously-changed path. That is precisely why the `implement-basic` fixture seeds the broad glob `runtime/**` rather than relying on the derivation. So binding the derived boundary to `write-boundary` as-is would reject every new-file `create` edit — the derivation is the wrong *shape* for enforcement, not just absent. Also note the derivation is never empty (it always contains the spec glob), so the "empty fallback" framing above does not apply.

Resolving this needs a design decision, not a fixture tweak: either (a) `derive-boundary` emits **directory globs** for changed paths (`runtime/src/foo.rs` → `runtime/**` or `runtime/src/**`) so new files in touched directories are permitted — which changes `derive-boundary`'s contract and its exact-path tests and any other consumer; or (b) writeCode enforcement treats an exact-path boundary entry as permitting its containing directory — a `path_in_boundary` semantic change. This is deferred pending that decision rather than shipped with the wrong shape. It also means the current exec-path writeCode flow depends on a seeded `write-boundary` that nothing writes, so a real `/gov:implement` exec run without a seed is fail-closed on new files — the deeper gap this scenario should ultimately close.

## Resolved Questions

*None yet.*
