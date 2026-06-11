---
spec: 003-bootstrap-automation
reviewed-at: 2026-06-11T01:48:34Z
reviewed-against: b9982910c3120ed67b63b90a7bb702a88de29403
diff-base: 9847647bc7c165d26dff07317c6a865a49f18457
must-violations: 0
should-violations: 3
low-confidence: 0
captured-issues: 0
skipped-passes: []
---

# Review — 003-bootstrap-automation

## Summary

Reviewed the code added under the `curl-sh-installer` scenario: `install.sh`
(the one-line installer), `scripts/audit/installer-registry-parity.sh` (audit
Family 14), and the supporting edits to `scripts/audit/run-all.sh`,
`check-zero.sh`, and `cross-doc-consistency.sh`. **No MUST violations — the spec
remains validly `done`.** Three advisory SHOULD findings, all in the simplicity
and reuse dimensions. The loaded security/api/config rule files target
application-backend concerns (auth, API schemas, secret handling) that this
build/CLI tooling does not engage; per the security pass's authoritative-rule-set
constraint, no security findings were manufactured for the `curl | sh` pattern
(govern has no published release artifacts to checksum — it is live-on-main).

## MUST violations (blocking)

None.

## SHOULD violations (advisory)

### SHOULD: simplicity — installer autodetect serves an edge the explicit commands already cover

- **File**: `install.sh:23-31`
- **Rule**: AGENTS.md §Design / simplicity pass — avoid indirection that is dead under the documented usage.
- **Finding**: The agent autodetect block (single existing `.claude`/`.augment`/`.agents` dir → that agent, else `claude`) only changes behavior for someone who pipes the bare Quick-start command inside an existing non-Claude project. The README gives an explicit `sh -s -- <agent>` one-liner per agent, so the common paths never reach the autodetect. It is correct and tested, but it is the most removable complexity in the script.
- **Auto-fixable**: no
- **Suggested fix**: Optional — drop the autodetect and default to `claude`, relying on the explicit per-agent commands. Kept deliberately for now; recorded as advisory.

### SHOULD: simplicity — `agy` alias accepted but undocumented

- **File**: `install.sh:53`
- **Rule**: simplicity / consistency — inputs the code accepts should match the inputs it advertises.
- **Finding**: The Antigravity arm matches `antigravity | agy`, but the usage comment, the unknown-agent error message (`expected: claude, auggie, antigravity`), and the README all list only the canonical names. The `agy` alias is silently accepted, which is a small surface/doc mismatch.
- **Auto-fixable**: no
- **Suggested fix**: Either document `agy` (it is the Antigravity CLI command name) or drop the alias for one-to-one parity with the registry keys.

### SHOULD: reuse — frontmatter-strip awk duplicated across the installer and govern.md

- **File**: `install.sh:62`
- **Rule**: reuse pass — shared logic should have one home.
- **Finding**: The `awk 'p{print} /^---[[:space:]]*$/{c++; if(c==2)p=1}'` body-extraction transform appears in `install.sh` and in `govern.md`'s self-update comparison. It is a single line and cannot be factored across a shell script and a markdown command file without a shared helper neither layer wants, so this is noted rather than actioned — but the two copies must stay in lockstep if the skill-wrapping convention ever changes.
- **Auto-fixable**: no
- **Suggested fix**: None practical; flagged so the coupling is visible.

## Low-confidence findings

None.

## Waived findings

None.

## Captured issues (pending /gov:groom)

None.

## Skipped passes

None.
</content>
