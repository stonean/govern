---
spec-ref: "013-text-first-artifacts — Principles"
tags: []
---

# Markdown Derived Views

## Context

013's text-first principle declares that *"structured derived views (SQLite caches, generated graph data, JSON indexes) are permitted only as gitignored build artifacts that consumers regenerate on demand."* The examples are non-markdown. The rule's intent is to keep binary or structured-noise artifacts out of git, where they don't diff cleanly and review tools don't understand them. But the rule's literal text would also gitignore *markdown* derived views, like `code-locations.md` produced by `/gov:implement`. Markdown derived views are different in shape: they diff cleanly in PRs, humans review them like any other markdown, and forcing regeneration before reading would break the agent's ability to use the artifact for cross-session resumption. The 000 scenario `code-location-index` resolved by committing the artifact, which exposes this gap and makes the clarification load-bearing.

## Behavior

- The constitution distinguishes markdown derived views from non-markdown derived views.
- **Markdown derived views** (e.g., `code-locations.md`) **may be committed** to git when their diffs are valuable to humans — review at PR time, onboarding orientation, refactoring impact analysis, or session-resumption context for the agent. Committing is permitted, not required; adopters may still gitignore a particular markdown derived view if they prefer.
- **Non-markdown derived views** (SQLite caches, JSON indexes, generated graph data, binary artifacts) **must remain gitignored** and be regenerated on demand by their consumers — unchanged from the current rule.
- The §text-first-artifacts section of the constitution is updated with one paragraph (or modified bullet) that draws this distinction. Source-of-truth artifacts (markdown specs, plans, etc.) continue to be the default; the clarification only affects the "structured derived view" carve-out.

## Edge Cases

- A markdown derived view that an adopter prefers not to commit — they `.gitignore` it locally; the constitution permits but doesn't require commit.
- A markdown file that's both source-of-truth AND derived (e.g., a partially-hand-edited file with auto-generated sections) — still treated as source-of-truth; the carve-out only applies to fully-derived artifacts.
- A non-markdown text format that diffs cleanly (TOML, YAML, plain text) — out of scope for this scenario. If a future case demands committing such an artifact, that's a separate constitutional discussion.

## Open Questions

*All open questions resolved during the parent scenario's clarify pass — see [code-location-index resolved questions](../../000-slash-commands/scenarios/code-location-index.md#resolved-questions).*

## Resolved Questions
