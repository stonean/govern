---
section: "Follow-on scenarios"
---

# Clarify-command-acceleration

## Context

This is the spec's first-scheduled follow-on scenario, deferred at the initial release for its multi-turn ABI and never shipped: `/gov:clarify` remains on the legacy-prose allowlist with zero primitive references, even though its deterministic scaffold maps entirely onto shipped primitives — the gate branch on (status, open-question count) is exactly `read-spec`'s output, dependency readiness is `traverse-deps`, the `gen-spec-deps.sh` safety net is `run-generator`, the validation gate is a zero-open-question count plus `lint-markdown`, and the draft→clarified flip is `set-status`. The question-resolution loop (the semantic core) maps to the `askClarifyQuestion` extension point, whose typed request builder ships in the extension-request-hygiene scenario. On the host side the multi-turn shape is ordinary: each open question is one `llm-request`/`llm-response` round trip that the host mediates through the user.

## Behavior

`/gov:clarify`'s Instructions are rewritten to the parseable conventions: numbered steps invoking `read-spec` (gate branch), `traverse-deps` (dependency readiness), `run-generator` (gen-spec-deps dry-run safety net), the `askClarifyQuestion` extension marker for the per-question loop (one round trip per open question; the host shows the question, returns the user's answer; the LLM applies the resolution to the spec body), `lint-markdown` plus a zero-open-question check for the validation gate, and `set-status` (draft → clarified) behind the user-approval gate. The file parses cleanly, leaves `legacy-prose-commands.txt`, and the markdown-only reference walks the same contract with host file tools.

## Edge Cases

- A spec with zero open questions short-circuits to the status-advance gate without any extension round trip.
- A `clarified`/`planned` target reports there is nothing to clarify, as today.
- Spec-body edits applying each answer remain LLM work on both paths — no primitive writes prose.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
