---
section: "Follow-on scenarios"
---

# Ask-consolidation — primitives for `/amend`'s scenario branch

## Context

Spec [023 — `govern` Refinement](../../023-govern-refinement/spec.md) consolidates `/elaborate` into `/amend`. The merged `/amend` classifies its input as a question or a scenario; the scenario branch creates a `scenarios/{slug}.md` file under the targeted feature, appends a linked task to that feature's `tasks.md`, and on a `done` spec flips status via the existing `set-status` primitive.

Two of those operations have no primitive today:

- Creating a scenario file from the template with `section` frontmatter and body content filled in. The existing `substitute-templates` primitive is shaped for bulk template→destination tree copy (the bootstrap pattern) and is overshaped for a single-file write.
- Appending a new task block to `tasks.md`. The existing `mark-task` primitive flips an existing checkbox; it does not append.

Falling back to host-side `Edit` calls for these two operations was considered and rejected — it breaks the spec 022 pattern ("every mechanical step is a primitive") and bypasses the atomic-write semantics that state-modifying primitives provide.

## Behavior

Two new primitives. Each is independently testable, atomic via tempfile + rename, and exposed on both surfaces (CLI subcommand and MCP tool under the `gov-rt:` namespace).

1. **`create-scenario`** — write a scenario file from the scenario template with frontmatter and body content populated.
   - Args: `feature-path: String` (e.g., `specs/042-foo`), `slug: String` (the scenario slug, no extension), `section: String` (the `section:` frontmatter value), `context: String` (Context section body), `behavior: String` (Behavior section body), `edge-cases: Option<String>` (when present, populates the Edge Cases section; when absent, the section is omitted).
   - Behavior: resolve the scenario template at `framework/templates/spec/scenario.md`, substitute the supplied values, write to `{feature-path}/scenarios/{slug}.md`. Create the scenarios subdirectory if absent. Atomic via tempfile-in-parent + `persist` rename. Refuse with a clean operational error if the destination already exists.
   - Result: `{ "created": "{feature-path}/scenarios/{slug}.md" }` on success; operational error on slug conflict or write failure.

2. **`append-task`** — append a new numbered task block to `tasks.md`.
   - Args: `feature-path: String`, `title: String` (the task title, e.g., "Implement scenario: ask-consolidation"), `done-when: String` (the "Done when" condition body), `body: Option<Vec<String>>` (optional list of checkbox sub-items; when absent, the primitive emits a single default `- [ ] Implement the behavior described in scenarios/{slug}.md` line — but the caller controls the body shape).
   - Behavior: read `{feature-path}/tasks.md`, count existing `## NNN.` headings to compute the next task number, append a new section block. If `tasks.md` does not exist, create it with the heading `# {NNN} — {Feature Name} Tasks` derived from the feature's spec title (or a minimal heading when the title can't be read). Atomic via tempfile-in-parent + `persist` rename.
   - Result: `{ "appended": { "task-number": N, "path": "{feature-path}/tasks.md" } }`. Operational error on write failure or unparseable existing tasks file.

Both primitives compose with existing primitives at the procedure level — `/amend`'s scenario branch in 023's `framework/commands/amend.md` rewrite will call `create-scenario` then `append-task` then (on a `done` spec) `set-status` to reopen, then `write-session` rewrites `.govern.session.toml` to point at the new scenario.

The CLI surfaces follow the same shape as existing write primitives:

```text
gvrn create-scenario --feature specs/042-foo --slug retry-on-timeout \
  --section "Network failure handling" \
  --context "Connections to the upstream may time out..." \
  --behavior "On timeout, the client retries up to three times..."

gvrn append-task --feature specs/042-foo \
  --title "Implement scenario: retry-on-timeout" \
  --done-when "the scenario's described behavior is correctly implemented and tested."
```

Both register under the canonical `gov-rt:` namespace (`gov-rt:create-scenario`, `gov-rt:append-task`) and join the existing tool list in `framework/runtime-tools.txt`. The pre-commit hook's MCP allow-list generator (added in spec 023 task 1) flows the new tool names through to both `framework/bootstrap/configure/claude.md` and `framework/bootstrap/configure/auggie.md` automatically on the same commit that updates `runtime-tools.txt`.

## Edge Cases

- **`create-scenario` with a slug that already exists** — refuse with a clean operational error: `scenario already exists: {feature-path}/scenarios/{slug}.md`. The caller (host or `/amend`'s scenario branch) decides whether to surface the conflict to the user or to retry under a different slug.
- **`create-scenario` against a feature path that does not exist** — refuse with `feature path does not exist: {feature-path}`. Do not create the parent feature directory; `/specify` owns feature creation.
- **`create-scenario` with no Edge Cases content** — omit the `## Edge Cases` section entirely from the rendered file. Matches the template's "remove this section if none apply" guidance.
- **`append-task` against a feature with no existing `tasks.md`** — create the file with the heading `# {NNN} — {Feature Name} Tasks`. When the feature's `spec.md` is unreadable (frontmatter parse failure, missing file), fall back to a minimal heading `# Tasks` and continue; do not halt on a soft failure to derive the feature name.
- **`append-task` with task numbers that skip values in the existing file** — use `max(existing) + 1` rather than `count + 1`, so a tasks file with `## 1.`, `## 3.` produces a new `## 4.` rather than overwriting `## 3.`.
- **Cross-platform tempfile-rename semantics** — same as the existing write primitives. `NamedTempFile` in the parent directory + `persist` rename. On Windows, fall back to the documented best-effort path (matching `mark-task` / `set-status` behavior) and surface the rename error if the platform rejects atomic semantics.
- **Concurrent invocation against the same `tasks.md`** — the second writer's `persist` step fails when the destination has been replaced underneath it. Surface as an operational error with the same shape as `mark-task`'s concurrent-edit error; the caller re-reads and retries.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

- **Why two primitives and not one combined `create-scenario-with-task`?** Single responsibility. `create-scenario` writes a scenario file; `append-task` extends a tasks file. They compose at the procedure level for the `/amend` scenario branch, and each is independently useful (a future `/groom` integration, for example, may want `append-task` without creating a scenario). Combining them would couple two failure modes that callers want to handle separately.
- **Why not extend `substitute-templates` to also handle single-file writes?** Rejected — `substitute-templates`'s contract is "overwrite the destination tree from a source tree with substitutions applied," a single-strategy bulk copy. Conflating it with single-file scenario creation muddies the abstraction. `create-scenario` is the more specific primitive that names what it's for; `substitute-templates` stays as-is.
- **Should the new primitives bump `gvrn` to a major version?** No — they are additive (no existing primitive contract changes), so a minor bump is appropriate (same pattern as 0.2 → 0.3 when the apply-manifest scenario shipped). The next `gvrn` release that ships these primitives moves `gvrn` to 0.4.0.
- **Should the primitives know about `/amend`'s classifier heuristic?** No. The classifier lives in `framework/commands/amend.md` as prose the LLM applies (per spec 023's resolved question on classification mechanism). The primitives operate one layer below — they perform the writes whichever route the classifier selects.
