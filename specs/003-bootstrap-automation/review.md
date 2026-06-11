---
spec: 003-bootstrap-automation
reviewed-at: 2026-06-11T01:57:04Z
reviewed-against: fc3a832ebbb2c9cca9aedcd6703a9c0be0081271
diff-base: 9847647bc7c165d26dff07317c6a865a49f18457
must-violations: 0
should-violations: 0
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 003-bootstrap-automation

## Summary

Reviewed the code added under the `curl-sh-installer` scenario: `install.sh`
(the one-line installer), `scripts/audit/installer-registry-parity.sh` (audit
Family 14), and the supporting edits to `scripts/audit/run-all.sh`,
`check-zero.sh`, and `cross-doc-consistency.sh`. **No MUST violations and no
SHOULD violations — the spec is validly `done`.** The loaded security/api/config
rule files target application-backend concerns (auth, API schemas, secret
handling) that this build/CLI tooling does not engage; per the security pass's
authoritative-rule-set constraint, no security findings were manufactured for
the `curl | sh` pattern (govern has no published release artifacts to checksum —
it is live-on-main).

The three advisory SHOULD findings from the prior run (2026-06-11T01:48Z) have
been resolved — see **Resolved since prior run** below.

## MUST violations (blocking)

None.

## SHOULD violations (advisory)

None.

## Resolved since prior run

- **simplicity — installer autodetect** (was `install.sh:23-31`). Removed. Agent
  resolution is now the positional argument or a `claude` default
  (`agent="${1:-claude}"`); the undocumented `GOVERN_AGENT` env override was
  dropped in the same pass for the same reason.
- **simplicity — `agy` alias accepted but undocumented** (was `install.sh:53`).
  Removed. The Antigravity arm is now `antigravity)`, so the installer's accepted
  agent names are exactly the §Agent Registry keys.
- **reuse — frontmatter-strip awk "duplication"** (was `install.sh:62`).
  Withdrawn as a mischaracterization: the `awk` literal lives only in
  `install.sh`. `govern.md` describes the frontmatter strip in prose (no literal
  awk), and the README's former copy was removed during the README rewrite.
  There is a single copy, so there is nothing to de-duplicate.

## Low-confidence findings

None.

## Waived findings

None.

## Captured issues (pending /gov:groom)

None.

## Skipped passes

None.
</content>
