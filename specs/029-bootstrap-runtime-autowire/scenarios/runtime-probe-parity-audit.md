---
section: "Follow-on scenarios"
---

# Runtime-probe-parity-audit

## Context

029 added the `command -v gvrn`-equivalent binary probe to two places per agent: the bootstrap permission **seed** (the `settings_template` blob in the §Agent Registry table of `framework/bootstrap/govern.md`, 029 Task 5) and the agent's steady-state permission set (`framework/bootstrap/configure/{key}.md`, 029 Task 6). The seed and the configure file overlap but neither contains the other — the seed grants bootstrap-only commands (`tar`, `mktemp`, `git rev-parse`, `git ls-files`, the `Read(...govern-*...)` temp globs) the configure file correctly omits, and the configure file grants pipeline commands the seed omits. The one permission 029 deliberately wired into **both** is the binary probe, so the probe — not the seed as a whole — is the entry whose cross-artifact parity must hold. No `/audit` family guards it: a maintainer adding or removing the probe in one place but not the other (dropping it from a configure file while leaving it in the registry seed, or vice versa) ships silently, and the gap surfaces only when a routine run re-prompts for the detection probe the seed was supposed to pre-grant. Surfaced 2026-06-10 during 029 implementation; deferred from 029 Task 9 so a new audit family is not smuggled into the auto-wiring change.

## Behavior

A new audit family `scripts/audit/runtime-probe-parity.sh` asserts, per agent, that the gvrn binary probe is in parity between the registry `settings_template` seed (scoped to the §Agent Registry section of `framework/bootstrap/govern.md`) and that agent's configure file. The check is a fixed-string presence assertion — no regex interpretation of the permission grammar — comparing the probe literal's presence in the seed against its presence in the configure file:

- present in both → parity holds, no finding;
- present in neither → the probe was deliberately removed from both, no finding;
- present in one only → a finding naming the agent, the probe, and the side missing it.

This guards the cross-artifact pairing 029 introduced (the probe landed in both Task 5's seed and Task 6's configure file); it does **not** assert the full seed is mirrored, because the seed and configure sets legitimately diverge.

The cascade when it lands:

- new `scripts/audit/runtime-probe-parity.sh`;
- a `run_check` line wired into `scripts/audit/run-all.sh` (bump its family-count comment);
- an enumerated entry in `framework/commands/audit.md` describing the family.

## Edge Cases

- **Per-agent grammar differs.** The probe is spelled `Bash(command -v *)` for claude, `"^command -v "` for auggie, and `command(which)` for antigravity (the resolved antigravity form — *not* `command(command -v)`). The assertion compares each agent's probe literal against the same agent's seed and configure file in that agent's native form — it never cross-compares grammars.
- **Bidirectional on the probe.** Parity is checked both ways: the probe missing from the configure file (but seeded) and the probe missing from the seed (but in the configure file) are both findings. Symmetric absence (in neither) is not — deliberately removing the probe from both artifacts is a legitimate change, not drift.
- **Probe-scoped, not seed-scoped.** The seed and configure sets legitimately diverge (bootstrap-only entries like `tar`/`mktemp` live only in the seed; pipeline entries live only in the configure file), so the family guards only the probe — the one permission 029 wired into both — rather than asserting the whole seed is mirrored.
- **New agent rows.** The script hard-codes the three current agents and their probe literals (matching the concreteness of the sibling `installer-registry-parity.sh`); a fourth agent that wires the probe is one added `check_agent` line — the script header calls this out.

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Why a scenario on 029 rather than its own feature spec?** 029 Task 9 worded the follow-up as "its own spec," but that was a sequencing decision — don't expand the audit surface while 029 was mid-implementation. With 029 done, this rides along as a scenario on the spec whose design it guards, matching the precedent set by audit Families 11–13 (each added as a scenario/subtask of the guarding spec under 022, never as a standalone feature spec). A new `specs/NNN-*/` directory for one script plus a `run_check` line plus an `audit.md` entry is heavier than the pattern warrants.
- **Why guard only the probe, not every seed entry?** The originating inbox item assumed the seed was a bootstrap subset of the configure file's full set, so every seed entry should be mirrored. Implementation (2026-06-11) found that false: the seed grants bootstrap-only commands (`tar`, `mktemp`, `git rev-parse`, `git ls-files`, the `Read(...govern-*...)` temp globs) the steady-state configure files correctly omit, and the configure files grant pipeline commands the seed omits — neither set is a subset of the other. The only permission 029 deliberately placed in both artifacts is the binary probe, so it is the sole entry whose cross-artifact parity is a real invariant; a whole-seed check would emit findings against the correct repo and fail this scenario's own done-when.
