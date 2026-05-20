---
spec: 023-govern-refinement
scenario: configure-dedup-permissions
reviewed-at: 2026-05-19T00:00:00Z
reviewed-against: e71bd410a7d8d6bdad82188bdc16a2af85ee945a
diff-base: e71bd410a7d8d6bdad82188bdc16a2af85ee945a
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 023-govern-refinement (configure-dedup-permissions scenario)

## Summary

Reviewed task #19's implementation — wiring `/configure` to the new `merge-permissions` primitive that landed on spec 022. The work is markdown-only: three slash-command source files edited and one generated file refreshed.

Scope:

- `framework/bootstrap/configure/claude.md` — Scope Boundaries inverted (the prohibition "do NOT … deduplicate, reorder, or rewrite" becomes "remove exact-match duplicates from `permissions.allow` and `permissions.deny`; do NOT reorder or rewrite non-duplicate entries"). Instructions step 1 invokes `merge-permissions` (MCP: `merge-permissions`) to install the canonical allow/deny sets and dedup. Steps 2/3 enumerate the canonical sets (data, not policy). Step 4 retains the host-side `additionalDirectories` handling that the primitive deliberately does not touch.
- `framework/bootstrap/configure/auggie.md` — preamble note documents that the new primitive is Claude-format only today; Auggie's structurally different `toolPermissions` array is host-walked until a future scenario decides whether to add a format argument to `merge-permissions` or introduce a separate Auggie-format primitive. Step 2's wording updated from "do not duplicate existing ones" (only blocks duplicate addition) to "no exact-match duplicate ... survives the run" (also removes pre-existing duplicates).
- `.claude/commands/gov/configure.md` — regenerated from the Claude source via `scripts/gen-claude-commands.sh`. Reflects the new instructions byte-for-byte.
- `specs/023-govern-refinement/tasks.md` — task #19 subtask checkbox flipped to `[x]`.

Stack: text-first markdown (no code paths added in this task). Loaded rule files: `configuration-cross.md`, `security-backend.md`, `api-backend.md`. None of the BE-API or BE-AUTHN/AUTHZ/etc. triggers fire — this task adds zero Rust, zero HTTP endpoints, zero new constants or env vars. The dedup behavior itself was reviewed under spec 022's `framework-list-dedup` review against `runtime/src/primitives/merge_permissions.rs`; this review is exclusively about the source-of-truth slash-command prose.

Five-dimension review:

- **Security**: no code surface added; the primitive's path-handling and JSON-parsing were reviewed under spec 022. The prose changes don't introduce new boundaries.
- **Reuse**: prose changes; no code duplication concerns. The Auggie open question is correctly cross-referenced to the `framework-list-dedup` scenario file rather than re-stated inline.
- **Quality**: the prose correctly describes the primitive's contract (created/updated/unchanged action, per-array counts, atomic write, byte-for-byte preservation of untouched fields). Step 4's two-write sequence (`merge-permissions` writes; then host re-reads to add `additionalDirectories`) is documented explicitly so the LLM doesn't accidentally clobber the primitive's output.
- **Efficiency**: not applicable to source prose.
- **Simplicity**: the new Instructions are longer than what they replaced (a single "Ensure X contains Y" sentence). The extra length is load-bearing — it names the primitive, describes the contract, and provides a markdown-only fallback for hosts without the runtime. No premature abstraction or dead branches.

**Result**: 0 MUST, 0 SHOULD, 0 low-confidence. `blocking: no`.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Waived findings

_None._

## Skipped passes

_None._
