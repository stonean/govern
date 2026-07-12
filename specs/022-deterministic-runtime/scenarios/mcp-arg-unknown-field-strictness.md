---
section: "Follow-on scenarios"
---

# Mcp-arg-unknown-field-strictness

## Context

No primitive `Args` struct sets `#[serde(deny_unknown_fields)]`, so an MCP caller that misspells an optional field has it silently dropped and the primitive runs with that field's default — flipping behavior for `fix`, `apply`, `force`, `recursive`, and the `sha256_url` verification-skip. Every field is kebab-case-renamed from a snake_case Rust name, which makes the snake_case misspelling (`sha256_url` for `sha256-url`, `glob_include` for `glob-include`, …) the *likely* caller error. `schemars` only emits `additionalProperties: false` when `deny_unknown_fields` is present, so a strict MCP client can't catch it either.

The blanket fix is not available: the subprocess interpreter binds every primitive's args from a clone of the **entire** walker context (a deliberate superset — `dispatch_primitive` passes one merged map and each primitive ignores the keys it doesn't need), so adding `deny_unknown_fields` to the shared `Args` structs would reject every primitive on the exec path. Strictness has to distinguish the MCP boundary (a clean, primitive-scoped args object) from the exec boundary (an intentional superset).

## Behavior

An MCP tool call whose arguments contain a field outside the target primitive's known set is rejected with a clear error naming the unknown field, rather than silently defaulting it. The subprocess interpreter's superset-context binding is unaffected — it continues to ignore the context keys a given primitive does not consume.

The mechanism is a per-primitive field allowlist applied at the MCP surface (e.g. validating the incoming object's keys against the primitive's `Args` field names before deserialization, or a wrapper type that is strict on the MCP path and lenient on the exec path) — not a blanket attribute on the shared `Args` structs.

## Edge Cases

- The exec path must still accept the full seeded context (session keys, prior `llm:*` echoes, threaded primitive results) without error.
- A field that is legitimately optional and simply omitted is not an error.
- The allowlist is derived, not hand-maintained, so a new `Args` field cannot drift out of sync (per the never-depend-on-diligence design principle).

## Open Questions

- Derive the allowlist from the `Args` type (e.g. via the schema `schemars` already produces) versus a second strict wrapper type — which avoids a hand-maintained list while keeping the exec path lenient?

**Implementation note (2026-07-12):** the MCP tools take `Parameters<Args>` and rmcp deserializes leniently into the typed struct before the tool body runs, so the unknown-field information is already gone at the method boundary, and the `Args` types are shared with the exec path (so `deny_unknown_fields` on them is out). Strict rejection needs a custom `Parameters`-style wrapper that validates the raw incoming object's keys against the schema before deserializing while still advertising the tool's input schema — which is precisely the rmcp param layer the inbox's **rmcp 1.x → 2.x migration** reworks. Building this wrapper against 1.x would be discarded at that migration, so this scenario should land **with** the rmcp 2.x port rather than before it.

## Resolved Questions

*None yet.*
