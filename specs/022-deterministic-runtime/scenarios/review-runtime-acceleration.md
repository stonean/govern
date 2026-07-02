---
section: "Follow-on scenarios"
---

# Review-runtime-acceleration — primitives + `performReview` for `/gov:review`

## Context

Spec 022 §Follow-on scenarios (under §Slash command rewiring) enumerates three deferred command rewrites, in order: `/gov:clarify`, `/gov:review`, `/gov:groom`. This scenario realizes the second — `/gov:review` — and introduces the `performReview` extension point named in §LLM extension points ("Deferred to scenarios on this spec").

022 originally judged review's runtime value "small (predominantly LLM work)". That framing undercounts the deterministic bookkeeping `framework/commands/review.md` carries: at 604 lines it is the largest command file, walks entirely in prose, and invokes zero primitives. The five review passes are genuinely LLM work — but everything around them is mechanical: rule-file discovery, `.govern.toml` parsing, waiver arithmetic, scope/diff-base computation, and report scaffolding. That is exactly the "LLM-walked mechanical work" this spec exists to eliminate. On the MCP path the agent loads all 604 lines and re-executes that bookkeeping by hand every invocation; on the exec path the runtime should own it. This scenario captures that pushback and specifies the primitives.

It also folds in two prose-convention refinements surfaced while reviewing the command set for token cost (both governed by §Per-rewrite checklist and enforced by the existing parseability check), plus a runtime content-ingestion convention surfaced while authoring this scenario (single-payload params).

## Behavior

### Primitives

Four new primitives, current `gvrn` convention (bare verb-noun names; MCP tools under the `mcp__gvrn__` prefix; CLI `gvrn` subcommands). Each is deterministic, independently testable, atomic (tempfile + rename) for the writing member, and registered in `framework/runtime-tools.txt`.

1. **`discover-rule-files`** — own review Behavior step 5 end-to-end. List the rule-file directory (`framework/rules/` here, `specs/rules/` in adopters); classify each basename by suffix (`-backend` / `-frontend` / `-cross` / unrecognized); apply the `[rules] surfaces` selection (valid list keeps listed surfaces plus every `-cross`; `[]` = cross-only; unset = derive from the detected stack; degenerate value fails fast); then apply the `[[review.disabled-rule-files]]` filter. Returns the selected rule-file set plus the ordered stdout notice lines the command must emit — the five existing forms verbatim (`disabled-rule-file:`, `disabled-rule-file (no-op):`, `unknown disabled-rule-file:`, `malformed disabled-rule-file at …`, `duplicate disabled-rule-file:`) followed by the closing `loading rule files: …` line.

2. **`process-waivers`** — walk `review.waivers` before findings are counted. For each entry: **apply** (file exists at anchor AND rule still fires) → excluded from `must-violations`, listed under Waived findings; **expire** (file gone or rule no longer fires) → drop on the next write and emit `waiver expired: …`; **do-not-extend** to a different file; **malformed** (missing field) → skip and warn, never auto-prune; **duplicate** on the (rule, file) pair → first applies, each dup warns. The anchor is the (rule, file) pair only — line numbers are not part of it. Returns applied / expired / warning sets.

3. **`compute-review-scope`** — resolve `diff-base` (the commit where the spec became `in-progress`, or the `--since` override), the file scope (plan `Affected Files` unioned with files modified since `diff-base`, whichever set is larger), and the inbox additions in the window (a `git diff` of `specs/inbox.md` from `diff-base` to HEAD). Returns the scope, the base sha, and the captured-issues list.

4. **`write-review`** — write `specs/NNN/review.md` (frontmatter plus the fixed section skeleton) and update the spec frontmatter `review:` block (`last-run`, `reviewed-against`, `must-violations`, `should-violations`, `blocking`). Applies the deterministic cross-pass dedup (highest-severity-wins on matching rule-id, file, and overlapping range) before counting. The empty-scope case (no implementation files) is a `write-review` branch that emits the 0-findings, `blocking: false` report — not a prose special-case. Blocking is true when `must-violations` exceeds zero; the exit code (0 or 1) is derivable from the returned counts. It consumes the pass findings as a single structured `findings` array (plus small scalar fields) and renders the report itself — it does not accept the report body as several per-section prose params (see the content-ingestion convention below).

### Extension point

**`performReview`** — one single-shot request/response per pass (five passes: security, reuse, quality, efficiency, simplicity). Request: the in-scope files plus the rules loaded for that pass. Response: findings (rule-id, severity, file, line-range, confidence, explanation). No multi-turn interaction — the single-shot pattern proven in the initial release; the multi-turn ABI is `/gov:clarify`'s concern. The tech-stack **alignment judgment** ("does the documented stack match the code") stays LLM/host — only the `[review] tech-stack-verified` gate read is deterministic.

### Prose-convention tightening (#3 / #4 — applies to every rewritten command)

- **Host-integration boilerplate stated once.** The ~1.1 KB "For agent runtimes" blockquote is byte-identical across the command files (modulo one example name). Its content is host-integration knowledge — bare-to-prefixed name mapping, lazy `ToolSearch` schema fetch, the no-shell-utilities rule, the two-paths-share-a-contract statement — whose canonical home is `constitution.md` §runtime-boundary (already the source of truth cited by `framework/runtime-tools.txt`). Move it there once; each command opens its Instructions with a one-line pointer. `/gov:target` already loads the constitution once per session, so the reference costs no extra read; markdown-only adopters keep the full contract in the constitution (§runtime-boundary principle 3 intact).
- **Redundant per-step tails dropped.** The parenthetical restating the MCP wire name duplicates the `Invoke` verb (the mapping rule already derives the wire name from the backticked primitive), and the trailing "Otherwise, follow the markdown-only path" restates the one two-paths sentence. Strip both. The parseability parser and `lint-tool-coverage.sh` key on the backticked primitive name in the `Invoke` step, not on the tails, so removal keeps both checks green.

### Content-ingestion convention (one payload, not multiple large params)

LLM-authored content crosses the runtime boundary as **one payload**, never as several large sibling string params. Two shapes satisfy it: a single prose `body` string (free-form authored text) or a single structured array (per-item records the runtime renders). This is both a robustness rule and a design constraint discovered while authoring this scenario:

- **Why.** The host's MCP parameter encoder drops a field when several large multi-line string params ride together in one tool call — reproduced deterministically against `create-scenario` (three large params `context` / `behavior` / `edge-cases`; the middle field vanished with `missing field 'behavior'`), while a single param of equal-or-greater size round-trips cleanly. gvrn itself is not at fault — its CLI accepts the same inputs verbatim — and the runtime cannot fix the host encoder, so it removes the trigger. Decomposition also buys no determinism (splitting prose into sections is the LLM's job, done in-context, not something the runtime computes) and it matches the `exec` extension-point ABI, where `writeSpecBody` / `writeCode` / `performReview` already pass content as single payloads.
- **`write-review`** consumes the pass findings as a single `findings` array (plus small scalar fields) and renders `review.md` itself — no per-section prose params. Designing it this way from the start is the point: a new content-writing primitive must not ship with the fragility this convention names.
- **Retrofit `create-scenario`** — collapse `context` + `behavior` + `edge-cases` into a single `body` string (the assembled `## Context` … `## Edge Cases` markdown); gvrn keeps the `section:` frontmatter, H1-from-slug, atomic write, conflict refusal, and the auto-appended Open / Resolved Questions scaffolding. Breaking arg-shape change → a `gvrn` version bump plus an update to `amend.md`'s scenario-branch prose (`create-scenario` invocation).
- **Token angle.** This is waste-avoidance, not a baseline reduction — the authored content is irreducible — but a dropped field forces the agent to re-emit multiple kilobytes on retry (or, worse, silently corrupts the write). Removing the trigger removes the retry tax.

### Tests (comprehensive — concrete and testable per the constitution; never depend on human diligence)

Every new primitive ships a `#[cfg(test)]` module in the norm of the existing ones (`append_task.rs` has 21 tests, `create_scenario.rs` 10): each Behavior branch and each Edge Case below is a named test with fixture inputs and asserted outputs. Minimum coverage:

- **`discover-rule-files`** — one test per suffix class (`-backend` / `-frontend` / `-cross` / unrecognized); `[rules] surfaces` as a valid list, `[]` (cross-only), unset (derive-from-stack), and each degenerate value (unrecognized member; a mixed valid-plus-invalid list failing on the invalid member; non-list type); each `[[review.disabled-rule-files]]` outcome (drop-and-notice, no-op notice, unknown warning, malformed warning, duplicate warning); and the notice **ordering** (disabled lines, then `loading rule files: …`). Assert the exact notice strings — they are contract.
- **`process-waivers`** — apply; expire (file gone); expire (rule no longer fires); do-not-extend to another file; malformed (missing field → skip-and-warn, not pruned); duplicate (first applies, dup warns); code-moved-within-file (anchor is the `(rule, file)` pair, not the line → still applies).
- **`compute-review-scope`** — diff-base from the status-to-`in-progress` commit; `--since` override; scope = the larger of (plan `Affected Files`, files-modified-since); inbox additions in the window; empty scope. Exercised against a temporary git fixture repo.
- **`write-review`** — empty-scope 0-findings / `blocking: false` report; cross-pass dedup (highest-severity-wins on rule-id + file + overlapping range); `blocking` true when `must-violations` exceeds zero; `skipped-passes` recorded under the dimension flags; the frontmatter `review:` block fields; single-`findings`-array ingestion.
- **`create-scenario` retrofit** — the single-`body` argument renders the same section structure the current three-argument tests assert; conflict refusal, atomic write, slug/path validation, and the Open / Resolved Questions scaffolding are unchanged (the existing 10 tests are updated, not dropped).
- **`performReview` ABI** — with a scripted / mock host: exactly one `llm-request` per non-skipped pass, no request for a skipped pass, and response findings flowing into `write-review`.
- **Prose conventions (#3 / #4)** — the parseability check and `lint-tool-coverage.sh` pass against every rewritten command file; assert the boilerplate blockquote is replaced by the one-line pointer and that no `(MCP: …)` or "Otherwise, follow the markdown-only path" tails remain.

**Testability boundaries (stated so no futile tests are written).** Two things are *not* gvrn-unit-testable and must not be faked: (1) the host MCP encoder dropping a field is host behavior the runtime cannot exercise — gvrn tests only that the single-payload contract renders correctly; (2) `performReview`'s semantic finding quality is an LLM seam (same bracket as `assessSpecQuality` / `writeCode`) — only its request/response wiring is asserted. Everything else above is deterministic and fully asserted.

## Edge Cases

- **Empty review scope** — `write-review` emits the 0-findings, `blocking: false`, exit-0 report; the "nothing to review yet" behavior is preserved as a primitive branch, not a prose special-case.
- **`[rules] surfaces` degenerate value** — `discover-rule-files` fails fast with the existing `CFG-ENV-003`-style operational error (message text unchanged from `review.md`); a list mixing valid and invalid members fails on the invalid member.
- **`surfaces = []` vs unset** — the empty list is valid and means cross-only; `discover-rule-files` must not conflate it with the unset (derive-from-stack) case.
- **Waiver code-movement** — because the anchor is the (rule, file) pair, `process-waivers` does not expire a waiver when the offending code moves within the file. Malformed and duplicate waivers are surfaced, never auto-pruned — they are operator state.
- **Boilerplate-dedup drift** — a command referencing a not-yet-migrated constitution section is caught at PR time by `/gov:analyze`'s `resolve-anchor` check (an unresolved §name), so the dedup cannot silently break the markdown-only path.
- **Namespacing** — the new primitives use the current `gvrn` convention, superseding the `gov-rt:` strings in the older `ask-consolidation` scenario.
- **`--all` review** — `discover-rule-files`, `compute-review-scope`, and `write-review` operate on one feature at a time; the `--all` loop stays in the command, invoking the primitives per targeted feature. No primitive iterates the feature set.
- **Dimension-restricting flags** (`--security` / `--simplicity` / `--quality`) — `performReview` is not invoked for a skipped pass; `write-review` records the skipped dimensions in `skipped-passes` and omits them from the counts.
- **`--fix` mode** — fix *application* stays LLM/host (it edits code); after the affected passes re-run, `write-review` performs a second write to update the post-fix counts. The primitive supports being called twice in one invocation.
- **Idempotency** — `write-review` is a pure function of findings + rules + code: re-running review on an unchanged target reproduces an identical `review.md` apart from `reviewed-at` / `reviewed-against`.

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Should the markdown-only-reference detail be relocated out of the command files (referenced, loaded only on the markdown-only path) rather than inlined in every command?** No — keep it inline. The `exec` surface already removes the entire command file from the agent's context, so the inline detail's token cost bites only on the transitional MCP path; the investment belongs in host `exec` adoption, not restructuring the command files. Relocating would reopen the "sibling `procedure.md`" drift class that 022 rejected on purpose (twin maintenance; markdown-only adopters tracking two files) — a high bar for a framework that leads with drift-prevention. The safe MCP-path savings are already captured by the #3/#4 tightening in this scenario's Behavior (dedupe the boilerplate to the constitution, drop the redundant per-step tails), which move no source of truth. Revisit only if a strategic host is pinned to the MCP path long-term and the #3/#4 trims prove insufficient; the narrower fallback then is to relocate only exhaustive primitive-internal detail into the primitives' own contracts (not a new sibling file), keeping the numbered procedure inline.

- **Why primitives for review when 022 judged the value "small"?** Because "small" conflated the semantic passes (genuinely LLM) with the surrounding bookkeeping (mechanical). The passes stay LLM via `performReview`; the bookkeeping moves to `discover-rule-files` / `process-waivers` / `compute-review-scope` / `write-review`. Net: the agent-visible review procedure drops from ~604 prose lines to the extension-point steps plus brief scaffolding.
- **`performReview` granularity — one call per pass, per file, or multi-turn?** One single-shot call per pass (five total). It matches the initial-release single-shot pattern and avoids the multi-turn ABI, which is `/gov:clarify`'s job to introduce first. Cross-pass dedup (highest-severity-wins) is deterministic and lands in `write-review`, not in the extension point.
- **Why fold #3/#4 into this scenario rather than a separate one?** Both are command-prose-convention refinements governed by §Per-rewrite checklist, and the review rewrite is the concrete occasion to apply them across the command set. A standalone prose-only scenario would be near-empty; the review rewrite already touches every convention they name.
