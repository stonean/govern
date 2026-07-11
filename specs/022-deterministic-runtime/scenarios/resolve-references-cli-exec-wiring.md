---
section: "Follow-on scenarios"
---

# Resolve-references-cli-exec-wiring

## Context

`resolve-references` is exposed as an MCP tool and listed in `framework/runtime-tools.txt` — whose header declares that bare names are also the `gvrn &lt;name&gt;` CLI subcommand identifiers — but it is absent from the parser's `PRIMITIVE_NAMES`, the interpreter's `dispatch_primitive` match, and the CLI `Command` enum. `gvrn resolve-references` is an unknown subcommand, the exec surface cannot dispatch it, and backticking the name inside any command's Instructions would hard-fail the parseability check. This is precisely the six-site wiring gap the AGENTS.md gotcha (recorded 2026-06-14 for this same primitive) warns about: MCP exposure alone passes `cargo test` and the tool-list superset check, so the gap stays invisible. It violates this spec's contract that each primitive has both a CLI subcommand and an MCP tool. Surfaced in the 2026-07-11 runtime review.

## Behavior

`resolve-references` is wired at all remaining sites: a `Command` enum variant and dispatch in `main.rs` (CLI subcommand), a `dispatch_primitive` arm in the interpreter (exec surface), and an entry in the parser's `PRIMITIVE_NAMES` (so rewritten command prose may backtick it). The parser doc comment's claim that `PRIMITIVE_NAMES` mirrors `TOOL_NAMES` is true again, and a test guards the three lists against future divergence.

## Edge Cases

- The longer-term fix — generating the five parallel name registries from a single source — is tracked in the inbox as consolidation work, not this scenario.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
