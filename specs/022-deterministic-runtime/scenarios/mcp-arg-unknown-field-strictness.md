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

*None — the allowlist-derivation question is resolved below.*

## Resolved Questions

- **Derive the allowlist from the `Args` type versus a second strict wrapper type?** Resolved 2026-07-12 in favor of deriving from the schema `schemars` already produces, enforced at the **router** rather than the param layer. `GovRuntimeServer::new` runs a `reject_unknown_fields` pass (a sibling of `strip_nonstandard_formats`): for each built route it reads the field names from `route.attr.input_schema["properties"]` and wraps `route.call` to reject any incoming argument key outside that set with an `invalid_params` error naming the field, before the macro's lenient `Parameters` deserialize runs. No `Args` struct is touched and none uses `#[serde(flatten)]`, so `properties` enumerates every known field and the allowlist can never drift from the type. The exec path never constructs this router, so its superset-context binding stays lenient. The custom `Parameters`-style wrapper (the other candidate) proved unnecessary.

**Implementation note (2026-07-12, resolved):** the earlier prediction that this needed the rmcp param layer to be reworked did not hold — the **rmcp 1.x → 2.x migration landed source-compatible**. `Parameters<Args>`, `ToolRouter` / `ToolRoute` (`pub map` / `pub attr`), and `model::Tool` (`input_schema` / `output_schema`) are all preserved across the major, so the bump was a no-op at the tool call sites and no custom `Parameters` wrapper was required. Strictness is enforced one level up, at the router, as described above. Landed together with the rmcp 2.2.0 bump. Tests: `mcp_surface_rejects_unknown_argument_field` (MCP surface rejects the `snake_case` misspelling) and `interpreter::tests::exec_path_ignores_unknown_argument_key` (exec surface still ignores it).
