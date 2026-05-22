---
spec: 027-bootstrap-migration-registry
reviewed-at: 2026-05-22T02:32:17Z
reviewed-against: 3e0053f36dcbc30969ec55221856101451a57f97
diff-base: 6be105dee33a01e15dacb9b5be310dbc044c397b
must-violations: 0
should-violations: 0
low-confidence: 0
skipped-passes: []
---

# Review — 027-bootstrap-migration-registry

## Summary

Clean across all five passes — 0 MUST, 0 SHOULD, 0 low-confidence findings. The spec's scope is overwhelmingly documentation: a TOML registry, six markdown procedure files, a rewritten `## Pre-run Migrations` bootstrap section, a ~120-line bash audit script, plus minor runtime touches (one module docstring trim, one parity-test assertion, one fixture markdown placeholder, and a CHANGELOG entry). The loaded rule files (security-backend, api-backend, configuration-cross) target a different problem domain (server-side auth, HTTP APIs, runtime configuration), so most rules don't fire on the in-scope code by construction — not because they were waived. The bash audit script and the registry-driven loop prose were the highest-leverage areas to scrutinize; both are sound. `blocking: false`.

Rule-file selection: stack inferred as "Markdown + Bash + Rust" (text-first framework, no frontend code in scope). Loaded `security-backend.md`, `api-backend.md`, `configuration-cross.md`; skipped `accessibility-frontend.md`, `performance-frontend.md`, `security-frontend.md`. No `[[review.disabled-rule-files]]` entries in `.govern.toml`. Tech-stack alignment check skipped per `[review] tech-stack-verified = true`.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

_None._

## Low-confidence findings

_None._

## Waived findings

_None._

## Skipped passes

_None._

## Pass-by-pass notes

### Security pass

Walked the in-scope code against `security-backend.md` (BE-AUTHN, BE-AUTHZ, BE-INPUT, BE-DATA, BE-API, BE-ERR) and `api-backend.md` (BE-SCHEMA, BE-APIVER, BE-ERRENV, BE-STATUS, BE-PAGE, BE-IDEMP, BE-COMPAT).

The two surfaces with non-trivial logic were checked explicitly:

- **`scripts/audit/migration-coverage.sh`** — the `python3 -c '…'` invocation on line 46 interpolates `$REGISTRY` (a script-local constant set on line 24 to the literal `"framework/migrations.toml"`) into a single-quoted Python string. BE-INPUT-003 governs shell-out and interpreter invocation where **user input** can alter request structure; `$REGISTRY` is framework-controlled, never derived from user input or external data, so the rule's verification clause does not fire. The rest of the script's `grep -qF`, `[ -e "$p" ]`, and glob-expansion paths take values exclusively from the script's own constants and from framework-owned `framework/migrations.toml` content, parsed via `tomllib` — no user-input surface. No injection vector, no MUST or SHOULD finding.
- **`framework/bootstrap/govern.md` §Pre-run Migrations** — the new loop reads `framework/migrations.toml` from the fetched archive (framework-owned), reads `.govern.toml` (adopter-owned config the adopter explicitly writes), prompts the operator before any mutation, and writes only to `.govern.toml`'s `[migrations].last_applied` field via atomic tempfile+rename. The duplicate-id + reference-integrity guard in the new §Duplicate-id and reference-integrity guard sub-section catches malformed registries before any procedure runs. No new authentication/authorization/transport surface introduced.

Procedure files (`framework/migrations/*.md`), the registry (`framework/migrations.toml`), and the runtime docstring/fixture/test changes are not code-with-secrets-or-network in any sense the rule files contemplate.

### Reuse pass

The bash audit script defines a single `emit` helper that all three checks (10a, 10b, 10c) share — no duplication. The script's structure mirrors the other Family scripts under `scripts/audit/` (one shell script per family, single `emit` helper, drift counter, final `exit "$drift"`); it could not credibly be factored further without manufacturing an abstraction. The six markdown procedure files share a fixed top-level shape (idempotency check → migration step(s) → summary line) per the plan's §Procedure file shape convention; the shape is convention rather than schema, which the plan §Known limitations explicitly accepts for v1.

The runtime docstring trim consolidates the prior "three cleanup loops" claim into "one (slash-command manifest enforcement) plus a forward-pointer to the registry-driven loop." Pure documentation rewrite — no code duplication touched.

### Quality pass

Bug-detection sweep across the bash script's three checks:

- **10a (no-orphan-procedure-files)** — `grep -qF "procedure_file = \"$f\"" "$REGISTRY"` uses fixed-string match, sensitive to TOML quoting variations. Every back-filled entry uses identical double-quoted formatting (verified in `framework/migrations.toml`), so the check is correct for the as-shipped state. A hypothetical future entry written with TOML literal-string quoting (`'…'`) would trigger a false-positive "orphan" report, but: (a) the script's `tomllib` parse on line 46 would still accept it, and (b) the cosmetic-formatting convention is uniform across the registry. Not a bug under current state; documentation-only brittleness.
- **10b (no-stale-target-paths)** — only `framework/`-prefixed paths are checked (line 95); adopter-relative paths like `{config_dir}/commands/{project}/skills/` are explicitly skipped with a `continue`. This matches the spec's contract (the audit cannot observe adopter projects) and is documented in the script header. Glob-expansion on line 101 (`matches=( $p )`) intentionally relies on shell word-splitting; `shellcheck disable=SC2206` acknowledges the choice. When the glob has no matches and `nullglob` isn't set, the loop iterates the literal pattern once and `[ -e "<pattern>" ]` correctly returns false. Behavior is correct.
- **10c (no-broken-procedure-references)** — straightforward existence check per entry; no edge cases.

The `migration-coverage.sh` script omits `set -e` (uses `set -uo pipefail` only) — intentional: the script collects all findings before exiting with the cumulative `$drift` count. Same pattern as the other Family scripts. Per-check failure does not short-circuit the run; that's the contract.

The runtime parity-test assertion (`runtime/tests/parity.rs:216-220`) is a positive assertion about file persistence — no off-by-one or boundary risk. The fixture file is a small markdown placeholder; no logic.

No findings.

### Efficiency pass

The audit script reads `framework/migrations.toml` exactly once via `python3 + tomllib` (lines 53-63) and iterates the parsed entries twice (lines 79-85 for 10c, 90-115 for 10b). Each glob in 10b runs `compgen`-style word-splitting once per pattern, bounded by the registry's `target_paths` count (currently 9 framework-prefixed paths across all entries, all literal except the four glob patterns under `rule-files-relocate` and one under `spec-and-plan-sunset`). The 10a check iterates the procedure-files directory once. All loop bounds are framework-controlled and small (<20 paths total). No N+1, no unbounded input, no repeated work.

The bootstrap loop's filter is a single linear scan of the registry (6 entries today; even at 10x growth it stays trivial). The procedure-file dispatch loop runs at most N times where N is the number of pending entries.

No findings.

### Simplicity pass

The audit script is 121 lines for three checks — a tight floor. The procedure files are 30-50 lines each, each owning idempotency + per-file behavior + summary. The registry's TOML shape is the minimum information needed for the bootstrap filter. The bootstrap loop's prose is the minimum reachable for the contract (read registry, read last_applied, filter, prompt, dispatch, update). The plan's Q11 rejection of an `apply-migrations` runtime primitive is honest about deferring abstraction until Family 9's primitive-promotion-candidates surfaces real duplication.

The runtime docstring trim removed text — strictly subtractive simplification. The added parity-test assertion is one stanza, locked to a one-file fixture.

No overengineering. No premature abstraction. No dead branches.

### Notes on rule applicability

The govern repo's rule files are ported wholesale from the framework's adopter-facing rule set, which targets server-side application code with authentication, HTTP APIs, database access, file uploads, JWT/OAuth flows, encryption at rest, secret management, and CSP/CORS configuration. Spec 027's surface is none of those: it adds a config registry, a procedural-prose loop, a static audit script, and a doc-only runtime contract trim. The loaded rules are inert-by-construction across this scope.

This is structurally distinct from "all clean because rules are weak" — the rules are weighty and well-specified; they simply target a different problem space. `tech-stack-verified` was set true earlier in the project lifecycle precisely because the codebase's "stack" (text-first framework + bash + Rust runtime) sits adjacent to but not inside the rule-file domain.
