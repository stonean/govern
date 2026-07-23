---
spec: 042-consolidate-govern-per-project-files-under-govern-directory
reviewed-at: 2026-07-23T00:50:37Z
reviewed-against: 50cc0702cf621c3766668c66275a83c47c0c6455
diff-base: 2b32d415558b5483d523711dd16669ee0566fc10
must-violations: 0
should-violations: 0
low-confidence: 1
captured-issues: 0
skipped-passes: []
---

# Review — 042-consolidate-govern-per-project-files-under-govern-directory

## Summary

Post-fix run: the QUAL-REUSE SHOULD from the 64b926c pass is resolved — `config_path` now derives from `config_display_name` (50cc070), so the new-wins choice lives once and the read path and provenance tag cannot disagree on the resolution rule; behavior unit-proven identical across all four presence cases. The tasks 14–15 delta plus release prep otherwise stands as reviewed: no new attack surface (display literals, doc comments, fixed-constant path helper), no new input handling, network calls, or secrets. 0 MUST, 0 SHOULD; 1 low-confidence note retained (probe-to-use race — read and display still resolve at separate moments at the call sites, though the choice logic is now single-sourced; mitigated by the serial pipeline and atomic writes). No issues captured to the inbox in the window. Not blocking.

## MUST violations (blocking)

*None.*

## SHOULD violations (advisory)

*None.*

## Low-confidence findings

### LOW-CONFIDENCE: BE-RACE-001 — resolver existence-probe → use window races a concurrent migration

- **File**: `runtime/src/schema/paths.rs:58-120`
- **Rule**: Shared mutable state reachable from more than one concurrent execution context MUST be protected by a synchronization mechanism — a lock, an atomic primitive, single-owner/actor confinement, or serialized access; unsynchronized concurrent read-write is a data race.
- **Finding**: Carried forward from the 8a770e8 run: the resolvers probe existence and return a path the caller later opens, and `discover-rule-files` / `dashboard` resolve the config once for reading and again for the provenance tag — two temporal probes, so a config file created or removed between read and render could tag a file other than the one read (the choice *logic* is now single-sourced in `config_display_name` after 50cc070, but the probes still run at separate moments). Mitigated by design: the pipeline is serial per constitution §concurrent-features, the migration runs only inside /govern, writes are atomic tempfile+rename, and the notice renders only when a config was successfully read. Recorded low-confidence for visibility, not as a confirmed defect.
- **Auto-fixable**: no

## Waived findings

*None.*

## Captured issues

*None.*

## Skipped passes

*None.*
