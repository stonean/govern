# Code Quality Rules

Enforceable code-quality rules that are not specific to a surface. These rules apply to all projects adopting `govern` regardless of stack — code-quality discipline is the same problem on backend workers, domain methods, middleware, and frontend stores alike.

Rules use RFC 2119 language: **MUST** / **MUST NOT** are enforced as errors; **SHOULD** / **SHOULD NOT** are flagged as warnings.

Rule IDs follow the format `QUAL-{CATEGORY}-{NNN}` and are permanent — once assigned, an ID is never renumbered, even if the rule is moved within the file or deprecated. Categories: `STUB` (silent stubs), `GROUND` (unverified external contracts). See `specs/036-quality-cross-rules/data-model.md` for the `QUAL` surface registration and `specs/008-security-rules/data-model.md` for the full rule schema.

These rules verify **code patterns** rather than design-time commitments; `/{project}:review`'s quality pass enforces them against source in scope. Cross-cutting (`-cross.md`) rule files always apply — a project that customizes this file can pin it in `.govern.toml` `[pinned]` so `govern` updates skip it.

## QUAL-STUB — Silent stubs

### QUAL-STUB-001

> A partial or unimplemented code path whose surrounding contract implies it performs work MUST fail loudly — via a panic, an explicit error return, or a failing/skipped test fixture — rather than silently passing through. Returning a zero value, returning `next` unchanged from middleware, returning early from a handler without an error, or returning `nil, nil` from such a method is a silent pass-through and does not satisfy this rule.

**Rationale:** Silent stubs ship indistinguishably from working implementations — a no-op rate-limiter, an always-allow permission check, a publisher that drops events on the floor — and the gap surfaces only when the missing behavior is needed, which is precisely when the system is under stress. In the anvil adopter project, a passthrough stub left in `RateLimit` enabled-mode would have silently disabled rate limiting in production had the follow-on task been skipped; failing loudly turns that latent production incident into an immediate, visible failure at the point the stub is exercised (or built/tested).

**Verification:** `/{project}:review`'s quality pass flags a code path in scope when **all three** hold: (1) it is **reachable** under the current spec; (2) its **surrounding contract implies work** — it is named for a behavior, documented to do something, or called by code that depends on its effect; and (3) it returns a success / zero / pass-through value with **no loud signal**. The following are compliant and are **not** flagged: an explicit incompleteness marker (`panic`/`todo!`/`unimplemented!`, a raised not-implemented error, or a failing/skipped test fixture) — that *is* failing loudly; intentional pass-through middleware documented as deliberate; a default or interface implementation meant to be empty; and a not-yet-reachable branch behind a feature flag or guard. The build-time **schema** fail-loud case is already governed by `api-backend.md` `BE-SCHEMA-002` — this rule covers the broader runtime/contract case across all surfaces rather than restating it.

**Source:** Fail-fast principle (Jim Shore, "Fail Fast", IEEE Software, 2004).

## QUAL-GROUND — Verify external contracts

### QUAL-GROUND-001

> Code whose correctness depends on an external contract it does not own — a database schema, another service's API shape, a config key, a file or wire format — SHOULD bind to that contract in a way that fails loudly when the assumption is wrong (a typed or generated binding, a schema/migration reference, a startup or first-use validation, or a test that exercises the real shape) rather than silently encoding an unverified assumption.

**Rationale:** An unverified assumption about an external contract fails silently and asymmetrically — the code works until the assumed column, field, or key differs from reality, at which point it breaks in production with no early signal. Grounding the assumption at the point it enters the code (constitution §grounding) turns a latent production incident into a build-time or test-time failure. This is the code-side counterpart to the artifact-grounding check `/{project}:analyze` runs against spec and plan claims, and it applies `QUAL-STUB-001`'s fail-loud principle to external contracts rather than unimplemented paths.

**Verification:** `/{project}:review`'s quality pass flags a code path in scope when **all three** hold: (1) its correctness depends on an external contract the code does **not** own — a database schema, an external service's API, a config key, or a file/wire format; (2) the assumed shape is **encoded directly** — a literal column/table/field/key name, or an assumed response structure — rather than through a binding that would surface a mismatch; and (3) there is **no fail-loud guard** — no generated or typed binding (ORM model, generated client, typed config), no schema or migration reference, no startup or first-use validation, and no test that exercises the real contract. The following are compliant and **not** flagged: a typed or generated client/ORM binding; a documented assumption paired with a validating guard that fails loudly; a value covered by a test against the real contract; and a contract the project itself owns and defines in-repo (an internal type, a local schema). Advisory (SHOULD) — the finding asks the author to ground or guard the assumption; it does not block `done`.

**Source:** Constitution §grounding (evidence discipline); fail-fast principle (Jim Shore, "Fail Fast", IEEE Software, 2004), as applied by `QUAL-STUB-001`.
