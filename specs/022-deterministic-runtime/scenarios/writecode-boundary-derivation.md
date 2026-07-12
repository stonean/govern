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

*None — all resolved.*

## Resolved Questions

**Implementation finding (2026-07-12) — boundary-format mismatch, reframed this scenario.** `derive-boundary` emitted the **exact paths** of files changed since the spec dir's first commit (e.g. `runtime/src/main.rs`), plus the single `specs/{feature}/**` glob. But the writeCode enforcement treats a non-glob boundary entry as an **exact-path match** (`/**` = any descendant, `/*` = direct children, otherwise exact). writeCode's job is to **create/edit files**, including *new* files not yet in git history — and a new file (`runtime/src/foo.rs`) does not exact-match any previously-changed path. That is precisely why the `implement-basic` fixture seeded the broad glob `runtime/**` rather than relying on the derivation. So binding the derived boundary to `write-boundary` as-is would have rejected every new-file `create` edit — the derivation was the wrong *shape* for enforcement, not just absent. The resolutions below follow from that finding.

- **Boundary format** (the fork the implementation finding above surfaced: emit directory globs from `derive-boundary`, or teach the writeCode matcher that an exact path permits its directory?). **Resolved: `derive-boundary` emits directory-zone globs.** Each changed path contributes its parent directory as `{dir}/**` (`runtime/src/main.rs` → `runtime/src/**`), deduped and sorted; a root-level changed file stays an exact path, since its zone would be `**` — everything. The matcher-side alternative was rejected because it makes boundary entries lie: an entry that reads as one file would silently grant its whole directory, retroactively widening every seeded exact path too. The glob emission keeps the matcher grammar honest — what the boundary says is what is granted — and puts the widening where it is visible, in the derived artifact.
- **Precedence when both a non-empty derivation and a seed are present: union, not override.** The walker merges `derive-boundary`'s `boundary` into the `write-boundary` enforcement key as seeded ∪ derived (a targeted merge exception alongside the create-feature/resolve-feature retargets; sorted for deterministic payloads). A seed is a deliberate host/user grant the derivation must never revoke — the "derived narrower boundary wins" edge case above would have history revoke an explicit grant — and on a fresh feature, whose derivation holds only the spec glob, the seed is what admits the first out-of-spec edit. With neither seed nor non-spec history, enforcement stays fail-closed: the first out-of-spec writeCode edit halts with `out-of-boundary-edit`, surfaced per §implement-phase rather than written anywhere.
- The `implement-basic` parity fixture stages a **two-commit** history (fixed-time signatures keep the shas golden-stable): the initial commit is the first spec-dir commit; a second commit touches `runtime/src/`, so the derivation yields `runtime/src/**` and admits the canned writeCode edit. The fixture's seeded `write-boundary` is **removed** — the re-blessed golden's enforcement key is populated purely by the derivation, proving the wiring end-to-end.
