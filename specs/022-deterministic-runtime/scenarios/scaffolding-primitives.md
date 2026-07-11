---
section: "Follow-on scenarios"
---

# Scaffolding-primitives

## Context

The 2026-07-11 coverage review found four clusters of deterministic scaffold work hand-rolled in prose across commands, each with either a shipped-but-unused primitive or no primitive at all:

- **Session writes**: specify.md's session step claims "the runtime exposes no session-shaped primitive for this step" — stale since `write-session` shipped in 0.10.0 — and hand-rolls the cli-config-dir-preserving write; target.md's `--clear` hand-rolls a clearing variant that `write-session` cannot express (it has no clear mode).
- **Feature resolution**: target.md's specs-dir scan with NNN/partial-name matching and its scenario frontmatter read have no primitive ("no runtime primitive iterates the specs directory" per its own prose), so every session's first command starts with LLM-walked directory iteration.
- **Feature creation**: specify.md's step 1 has the host pre-compute the next NNN number, derive the slug, create the directory, and copy the template — all deterministic, no primitive.
- **Inbox appends**: log.md, implement.md's auto-capture rule, and the bootstrap security audit each hand-roll the same atomic append-to-`specs/inbox.md` (the bootstrap variant with its own dedup-by-prefix). Additionally, plan.md and specify.md carry plain ask-for-approval steps while `gate-confirm` exists and prune.md proves the pattern; and `gen-spec-deps.sh` is named raw in several command files while only analyze.md routes it through `run-generator`.

## Behavior

- `write-session` gains a `clear` mode that removes the target while preserving `cli-config-dir`; target.md `--clear` and specify.md's session step invoke `write-session` and the stale no-primitive claim is deleted.
- A new `resolve-feature` primitive scans the configured specs root and resolves a feature by exact number, zero-padded number, or unique partial slug — returning the directory name, path, status, and (when a scenario slug is supplied) the scenario file's frontmatter and existence. target.md invokes it; ambiguity and no-match remain host-mediated prompts.
- A new `create-feature` primitive computes the next feature number, derives the kebab-case slug, creates `specs/{NNN-slug}/`, and copies the spec template into it (atomic, mode-preserving); specify.md invokes it and the LLM fills the body via `writeSpecBody` as today.
- A new `append-inbox` primitive appends one bullet to `specs/inbox.md` (creating the file from its template when missing) with optional dedup-by-prefix; log.md invokes it, implement.md's auto-capture names it, and the bootstrap audit's append points at the same contract.
- plan.md and specify.md approval gates invoke `gate-confirm`; command files that name `gen-spec-deps.sh` raw route it through `run-generator`.
- All new primitives are wired at every site (schema Args/Result, primitive module, mod.rs, MCP server, PRIMITIVE_NAMES, interpreter dispatch, CLI enum, `runtime-tools.txt`, data-model entries, regenerated configure permission blocks) per the AGENTS.md six-site rule.

## Edge Cases

- `resolve-feature` with an ambiguous partial match returns the candidate list as a domain outcome; choosing stays with the user through the host.
- `create-feature` refuses when the derived directory already exists (no overwrite path).
- `append-inbox` with dedup finds an existing bullet by prefix and reports `deduped: true` instead of appending twice.
- The markdown-only path for each rewritten step keeps the current prose as the documented fallback — including the cli-config-dir preservation rule, which must survive verbatim for hosts without the runtime.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
