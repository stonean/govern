---
section: "Command Set"
---

# Dashboard-dependencies-column

## Context

The 000 spec describes `status` as a "read-only dashboard of all specs, their status, artifacts, **dependencies**, and next actions" (Behavior → Command Set → Utility commands), and the acceptance criterion says the command "displays a dashboard table." The current rendering procedure in `framework/commands/status.md` step 3 lists columns `Feature | Status | Plan | Tasks | Data-model | Scenarios | Next Action` — dependencies are not in the table.

The data is already available: the `dashboard` primitive (per [022's dashboard-primitive scenario](../../022-deterministic-runtime/scenarios/dashboard-primitive.md)) returns each spec's `dependencies` array in its per-spec payload. Only the renderer changes.

The user-visible gap: when multiple specs are non-done, the user has no glance-level view of dependency order — they cannot tell which spec to start on without opening each spec file individually. The spec's promise of dependencies in the dashboard isn't being kept.

A second, related rendering defect surfaces in the same table: the session-target marker. Step 3 says "Mark the row matching the session target with a leading `>>`," which produces a first cell like `| >> 022-deterministic-runtime |`. Several markdown renderers (observed 2026-05-23 in the agent's own paste-back of `/gov:status` output) strip the leading pipe on that row, dropping it out of the table grid; the rendered output visibly breaks alignment exactly on the row the marker is meant to highlight.

## Behavior

`framework/commands/status.md` step 3 (table render) changes in two ways:

1. **Add a Dependencies column** between Scenarios and Next Action. Cell content: comma-separated NNN prefixes (the leading three digits of each entry in `specs[].dependencies`, sorted ascending), or `—` when the array is empty.

   Example:

   - 024-rule-loader depends on 020-code-review and 023-govern-refinement → cell shows `020, 023`.
   - 000-slash-commands has no deps → cell shows `—`.

2. **Replace the `>>` row-prefix marker** with one that survives markdown renderers. Wrap the target row's Feature cell in `**…**` (bold) — e.g., `| **022-deterministic-runtime** | ... |`. Bold is a well-supported inline span that does not interact with table-cell parsing the way a leading `>>` does. When no session target is set (`dashboard` returns `session-target: null`), no row is bolded.

No primitive change required — both changes are in the rendering procedure; the data is already in the payload.

## Edge Cases

- **Empty `dependencies` array** → cell shows `—`, matching the existing convention used by Data-model and similar columns.
- **`dependencies` entry whose target spec doesn't exist** (drift) → cell shows the NNN prefix as-recorded; `validate` catches dangling refs.
- **No session target set** → no row is bolded; the table renders normally with no marker. The "No session target" preamble line above the table is unchanged.
- **Session target names a slug not in the dashboard inventory** (e.g., stale session file pointing at a deleted feature) → no row matches, so no row is bolded. The caller continues to surface the stale target through the preamble line as today; the table render is silent about it.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
