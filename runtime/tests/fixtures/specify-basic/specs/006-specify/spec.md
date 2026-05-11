---
status: draft
dependencies: []
---

# 006 — Specify Fixture

A fixture for the `/gov:specify` parity test. The host pre-creates this
directory and the empty spec.md (mirroring the template-copy step) so
the runtime walker can lint the file and confirm with the user. The
writeSpecBody response fills the body content in the host's working
copy; this committed fixture stays at the minimal template shape so
the parity test reads it deterministically.

## Motivation

Placeholder body — the writeSpecBody extension supplies the real body
content at host time.

## Acceptance Criteria

- [ ] `runtime exec specify` walks the procedure to completion.

## Open Questions

*None — all resolved.*
