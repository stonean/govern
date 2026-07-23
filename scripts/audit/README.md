# `scripts/audit/`

Per-family check scripts for `/audit`. See [spec 026](../../specs/026-framework-self-audit/spec.md) and the [026 plan](../../specs/026-framework-self-audit/plan.md) for the design.

## Contract

Every script in this directory follows the same contract:

- **Output.** Findings written to stdout, one per line, in the format `FAMILY | LOCATION | MESSAGE | SUGGESTED-FIX` (pipe-separated, columns aligned for readability when the runtime renders the aggregated output).
- **Exit code.** `0` when no findings; `1` when any finding is present. Aggregated by `/audit` via logical OR — any family with findings makes the whole audit fail.
- **Read-only.** No file modifications. Scripts may write to `$TMPDIR` for intermediate computation but must not touch the working tree.
- **Idempotent.** Same inputs produce identical output across runs.
- **Self-contained.** Each script can be invoked directly (`bash scripts/audit/{family}.sh`) without orchestration — useful when triaging a specific check.

## Scripts

- `check-zero.sh` — generator/lint precondition pass. Run before family checks; halts `/audit` on failure to avoid misleading findings against known-stale generator output.
- `cross-doc-consistency.sh` — Family 1.
- `manifest-parity.sh` — Family 2.
- `placeholder-roundtrip.sh` — Family 4. (Family 3, registry equivalence, was retired with the workflows feature — spec 043; family numbers are stable identifiers, so the gap stands.)
- `template-alignment.sh` — Family 5.
- `ssot-invariants.sh` — Family 6.
- `sibling-coupling.sh` — Family 7.
- `introducing-drift.sh` — Family 8.
- `primitive-promotion-candidates.sh` — Family 9.
- `migration-coverage.sh` — Family 10.
- `consolidation-pair.sh` — Family 11.
- `fixture-session-shape.sh` — Family 12.
- `runtime-hardcoded-paths.sh` — Family 13.
- `installer-registry-parity.sh` — Family 14.
- `runtime-probe-parity.sh` — Family 15.
- `installer-command-parity.sh` — Family 16. `/govern`'s §Per-Agent Scaffolding slash-command manifest must list exactly the `framework/commands/*.md` files, minus the maintainer-only commands (`audit`) intentionally withheld from adopters.

Families 1–9 are described in detail in the [026 spec](../../specs/026-framework-self-audit/spec.md#check-families) and the [026 plan's Technical Decisions](../../specs/026-framework-self-audit/plan.md#per-family-script-designs); families 10+ were added incrementally and carry their rationale in each script's header comment.
