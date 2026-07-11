---
section: "Follow-on scenarios"
---

# Host-protocol-conformance

## Context

The 2026-07-11 runtime review found three seams where the binary's behavior diverges from the documented host contract (this spec's acceptance criteria and the data model's envelope documentation):

- A command-file parse failure under `gvrn exec` prints to stderr and exits 2 without emitting any `error` JSON envelope — no code, no runtime version, no version-mismatch note — though the versioning-enforcement resolution and a checked acceptance criterion require exactly that, and the exit-code contract (1–127 = clean operational error with terminal `error` message) forbids a message-less non-zero exit.
- Inbound stdio handling contradicts the data model's robustness rule ("the runtime ignores any other inbound JSON shape — it logs to stderr and continues waiting"): a wrong-type envelope or mismatched request-id halts the walk with `protocol-mismatch`, and a malformed or blank JSON line propagates a raw I/O error to exit 74 with no terminal envelope.
- The `assessSpecQuality` extension point never constructs its documented typed request (`spec-path`, `spec-content`, `rule{id, verification, severity}`); `build_extension_request` has no arm for it, so hosts receive a raw walker-context dump for `/gov:analyze`'s headline extension point. `AssessSpecQualityRequest` is dead code.

## Behavior

The binary honors the protocol contract as documented:

- `gvrn exec` emits a terminal `error` envelope on command-file parse failure — descriptive message, the runtime version, and a note that a framework/runtime version mismatch is a possible cause — before exiting non-zero.
- Unknown, wrong-type, mismatched-request-id, or unparseable inbound lines while suspended are logged to stderr and skipped; the runtime continues waiting for a valid response. Only stdin EOF while awaiting a response is an operational error.
- `build_extension_request` constructs the typed `assessSpecQuality` request matching the data model's shape, mirroring the `writeCode` builder discipline.

## Edge Cases

- A late `llm-response` for a superseded request-id is ignored, not fatal.
- Blank keepalive lines between envelopes are ignored.
- The interpreter module doc's claim that the runtime "panics on malformed JSON on stdin" is corrected to describe the ignore rule.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
