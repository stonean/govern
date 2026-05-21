---
section: "LLM extension points"
---

# Writecode-payload-bundling

## Context

Spec 022's "LLM extension points" section declares the `writeCode` request payload as `{task description, plan-relevant files, write boundary}` and the `writeSpecBody` request as including optional `existing-content` for re-runs on partially-filled files. The runtime schema in `runtime/src/schema/extensions.rs` mirrors these shapes precisely. However, the interpreter at `runtime/src/interpreter/mod.rs:179` emits each `llm-request` with `request: Value::Object(self.context.clone())` — the walker context contains only scalar fields seeded by primitives (task-number, write-boundary, threshold, etc.). The rich list-shaped fields (`plan-relevant-files`, `constitution-excerpts`, `existing-content`) are left empty; the host's LLM does its own context gathering before producing a response. The implementation has not caught up to the design intent.

The per-task loop of `/gov:implement` compounds the cost: across N tasks in one feature, the same constitution excerpts and plan-relevant files re-enter the host's prompt context N times. Without a deliberately cache-anchored payload shape, every task pays the full token cost for content that does not change between tasks.

## Behavior

1. **Populate `writeCode.plan-relevant-files`.** Before emitting the `llm-request` envelope for a `writeCode` extension point, the interpreter parses the targeted feature's `plan.md` Affected Files section, reads each listed repo-relative file from disk, and emits each as `{path, content}` in the request. Files listed in the plan but absent from disk (planned-new files) are omitted from the array, not errored.

2. **Populate `writeCode.constitution-excerpts`.** Before emitting, the interpreter parses the running slash command file's `Reference: §<anchor>, §<anchor>` line under Scope Boundaries, resolves each anchor via the existing `resolve-anchor` primitive, and emits each resolved section body as a string in the array. Command files with no `Reference:` line yield an empty array.

3. **Reorder `WriteCodeRequest` for cache anchoring.** The struct field order in `runtime/src/schema/extensions.rs` becomes: `constitution-excerpts`, `plan-relevant-files`, `write-boundary`, `task`. The stable prefix (the first three fields) is contiguous and front; the per-task variable suffix (`task`) is last. Serialized JSON respects this order via `serde`'s declaration-order serialization.

4. **Cache-breakpoint contract.** Spec 022's LLM extension points section gains a one-paragraph contract: hosts SHOULD place a prompt-cache anchor immediately after `write-boundary` and before `task` in serialized `writeCode` request payloads. The contract is advisory — hosts that do not implement prompt caching produce correct results, just at higher token cost per task. Independent host integrations (Claude Code, Auggie) converge on the same anchor position by following the contract.

5. **Populate `writeSpecBody.existing-content`.** When `/gov:specify` or `/gov:plan` re-runs on a partially-filled spec or plan section, the interpreter reads the current section body from disk and emits it in the `existing-content` field. Empty sections emit `None` (matching the current schema default).

6. **Read-side secret-exfiltration guard.** The `plan-relevant-files` read path adds a new guard analogous to `derive-boundary`'s write guard. Before inlining a file's contents, the interpreter refuses files matching common secret-bearing patterns: `.env`, `.env.*`, `*-secrets.*`, `credentials*`. The guard also respects the repo's `.gitignore`. A matched path triggers a structured `secret-exfiltration-blocked` error envelope; the procedure halts and the plan author resolves by removing or renaming the entry in Affected Files.

7. **Parity discipline.** Every change in this scenario carries a `runtime/tests/parity/` test proving the markdown-only walker and the runtime walker produce equivalent state mutations against a fixture for `/gov:implement` and `/gov:plan`. The new bundled fields are exercised against fixtures with realistic plan tables, command `Reference:` lines, and re-run states.

## Edge Cases

- **Missing or malformed Affected Files table** in `plan.md`: `plan-relevant-files` emits as `[]`; the procedure continues. The host's LLM falls back to its existing context-gathering behavior.
- **Listed file absent from disk** (planned-new file or rename): omit from the array; do not error.
- **Command file has no `Reference:` line** (legacy command files yet to be rewritten under 022's conventions): `constitution-excerpts` emits as `[]`.
- **Secret-pattern match in Affected Files**: halt the procedure with `secret-exfiltration-blocked`. The author resolves by editing the plan; the runtime does not provide an override flag in v1.
- **Host has no prompt-cache anchoring** (third-party hosts): the bundle still works correctly; the host pays full token cost per task. The cache contract is SHOULD, not MUST.
- **Out-of-scope LLM extensions**: `assessSpecQuality` is not modified — its request payload is already complete today.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
