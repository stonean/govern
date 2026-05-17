# Configuration Rules

Enforceable rules for operator-tunable values, named constants, and environment variables. These rules apply to all projects adopting `govern`, regardless of whether the code is backend or frontend — configuration is the same problem on both surfaces.

Rules use RFC 2119 language: **MUST** / **MUST NOT** are enforced by the validate command (errors); **SHOULD** / **SHOULD NOT** are flagged as warnings.

Rule IDs follow the format `CFG-{CATEGORY}-{NNN}` and are permanent — once assigned, an ID is never renumbered, even if the rule is moved within the file or deprecated. Categories: `CONST` (constants), `ENV` (environment variables). See `specs/017-derive-dont-ask/data-model.md` for the full schema.

## CFG-CONST — Constants

### CFG-CONST-001

> Shared constants — values used across multiple modules — MUST live in a centralized location idiomatic to the project's language (e.g., `shared/constants/` in JavaScript/TypeScript, `internal/constants/` in Go, a top-level constants module in Python) rather than being duplicated across modules.

**Rationale:** Cross-cutting defaults that drift between modules produce silent inconsistencies — a timeout treated as 30s in one place and 60s in another. A single location makes the canonical value findable and auditable.

**Verification:** Any spec or plan that introduces a value used by more than one module (timeouts, sizes, thresholds, rate limits, format strings, well-known headers, protocol versions) MUST name the centralized constants location it will live in. Validate flags plans that introduce cross-module values without naming the shared-constants location, and flags duplicated literal definitions in plan affected-files snippets.

**Source:** Twelve-Factor App (III. Config); "Don't Repeat Yourself"

### CFG-CONST-002

> Module-local constants — values used only within a single module — MUST live within that module (a dedicated constants file inside the module, or at the top of the file that uses them), not in the shared constants location.

**Rationale:** Co-locating a single module's constants with that module keeps the module self-contained and avoids coupling unrelated modules through a shared import. The shared constants location stays focused on values that genuinely cross modules.

**Verification:** Any spec or plan that introduces a named constant scoped to one module MUST place it within that module, not in the shared location. Validate flags plans that propose adding single-module values to the shared constants location.

### CFG-CONST-003

> Operator-tunable values (timeouts, retry counts, batch sizes, thresholds, rate limits, expiry durations) MUST be backed by a named constant or an environment variable. They MUST NOT appear as bare literals in business logic.

**Rationale:** Bare literals scattered across the codebase are invisible to operators, hard to audit, and easy to leave inconsistent during tuning. A single named source of truth makes the value findable, changeable, and auditable.

**Verification:** Any spec or plan that introduces operator-tunable behavior MUST commit to a named constant or env var for each value. Validate flags plan affected-files snippets that show numeric or string literals of operator-tunable shape (durations, counts, thresholds, rate limits) without a constant or env var lookup. Ordinary literals used for local logic — loop indices, intermediate calculations, string formatting within a function — are out of scope.

### CFG-CONST-004

> When an operator-tunable value has a valid range, the bounds (minimum and/or maximum) MUST be expressed as named constants alongside the default constant.

**Rationale:** Bare-literal bounds in validation code are invisible to operators looking for the safe range, drift between the defining spec and the implementation, and produce inconsistent enforcement when the same value is checked in more than one place. Naming the bounds makes the safe range discoverable and enforceable from one place.

**Verification:** Any spec or plan that introduces a value with a documented valid range MUST commit to named constants for each bound (e.g., `MinHTTPReadTimeoutSeconds = 1`, `MaxHTTPReadTimeoutSeconds = 300`) alongside the default constant. Validate flags plans that propose range checks using bare literals.

## CFG-ENV — Environment variables

### CFG-ENV-001

> Every environment variable MUST be declared as either **optional with a default fallback defined as a named constant**, or **required with no default** (in which case `CFG-ENV-003` governs its startup validation). Secrets MUST be declared required and MUST NOT carry an in-application default value (see `BE-DATA-003`). All environment variables MUST be read once at startup and the value cached; per-call reads from `os.environ` (or equivalent) are forbidden.

**Rationale:** Repeated env reads are a silent dependency on process state, slow hot paths, and make defaults invisible to readers. Reading once at startup, falling back to a constant for optional values, and failing fast for required values produces predictable behavior, makes defaults discoverable, and keeps the runtime fast. Excluding secrets from in-code defaults prevents accidentally shipping placeholder credentials.

**Verification:** Any spec or plan that introduces an env var MUST declare it optional-with-default-constant or required-no-default, and MUST commit to startup-time resolution. Validate flags plans that propose optional env vars without naming the default constant, plans that propose secrets with a hard-coded default, and plans that show per-call env reads in affected-files snippets.

**Source:** Twelve-Factor App (III. Config)

### CFG-ENV-002

> A single canonical inventory of every environment variable the application reads MUST be maintained alongside the source code. The inventory MUST describe each variable's purpose, declare whether it is required or optional, and provide a safe placeholder or default value. Acceptable forms include `.env.example` (twelve-factor projects), a values schema (`values.schema.json` for Helm charts), a typed config module that lists every variable, or an equivalent declarative manifest checked into the repository.

**Rationale:** Operators discover required configuration by reading this inventory. Variables introduced in code but absent from the inventory produce silent runtime failures in fresh deployments and obscure the application's true configuration surface. The form of the inventory follows the deployment model — `.env.example` for processes, Helm values for Kubernetes, etc. — but the single-source-of-truth requirement does not.

**Verification:** Any spec or plan that introduces an env var MUST name the inventory file and include updating it as part of its tasks. Validate flags plans that introduce env vars without a corresponding inventory update in affected-files.

### CFG-ENV-003

> Every required environment variable MUST be validated at startup. The application MUST fail fast — exit non-zero with a clear error message naming the variable — when a required variable cannot be resolved (neither environment nor default available).

**Rationale:** Unvalidated config produces partial-failure modes that surface only at first use of the variable, often deep in a request path. Fail-fast at startup turns a confusing intermittent error into an obvious deployment-time failure.

**Verification:** Any spec or plan that introduces required env vars MUST commit to startup-time validation that names the failing variable in its error message. Validate flags plans that introduce env vars without a startup-validation step.

**Source:** Twelve-Factor App (III. Config); "Fail Fast" pattern

### CFG-ENV-004

> Environment variables holding time values MUST express their unit unambiguously. Two patterns are acceptable:
>
> - **Unit-in-value (preferred):** the env var value is parsed as a duration string (e.g., `30s`, `5m`, `100ms`); the parser MUST reject values without an explicit unit so a bare `30` is a parse error rather than an ambiguous quantity.
> - **Unit-in-name:** the env var name carries a unit suffix (`_MS`, `_SECONDS`, `_MINUTES`, `_HOURS`) and the value is an integer in that unit.
>
> The corresponding default constant MUST also make the unit explicit — either by being a `time.Duration` value (e.g., `5 * time.Second`) or by carrying a matching unit suffix when the constant is an integer (e.g., `DEFAULT_SHUTDOWN_TIMEOUT_SECONDS = 30`).

**Rationale:** Unit-less time variables produce off-by-1000x bugs — treating milliseconds as seconds, or vice versa. The unit-in-value pattern eliminates the ambiguity at the source: the operator MUST type a unit, and the parser rejects them if they don't. The unit-in-name pattern remains valid for cases where an integer is preferable (e.g., scripting environments that struggle with the duration-string format), but it shifts the safety burden to operator discipline rather than the parser.

**Verification:** Any spec or plan that introduces a time-valued env var MUST commit to one of the two patterns. For unit-in-value, the loader MUST use a duration parser that rejects bare numbers; for unit-in-name, both the env var name and the default constant name MUST carry the unit suffix. Validate flags plans that propose `*_TIMEOUT`, `*_INTERVAL`, `*_DELAY`, `*_TTL`, etc. without committing to one of the two patterns.

**Source:** IEC 60027 (units of measurement); "Make Illegal States Unrepresentable"

### CFG-ENV-005

> Environment variables whose Config field has a documented valid range MUST be validated against that range at startup. Out-of-range values MUST cause fail-fast with an error message naming the variable, the offending value, and the violated bound.

**Rationale:** A misconfigured timeout, retry count, pool size, or threshold can produce silent degraded behavior at runtime — requests timing out under load, exhausting connections, retrying forever — long after deployment. Catching out-of-range values at startup turns a confusing production incident into an obvious deployment-time failure.

**Verification:** Any spec or plan that introduces an env var with a documented valid range MUST include a startup-validation step that names the failing variable, the offending value, and the violated bound in the error message. Validate flags plans that introduce ranged env vars without a startup-range-check step.

**Source:** "Fail Fast" pattern; defense in depth

### CFG-ENV-006

> Configuration values flagged as sensitive (secrets, credentials, tokens, keys, PII) MUST be redacted before appearing in logs, error messages, diagnostic dumps, health-check responses, or any other operator-facing output. The redaction MUST occur at the config-loading layer so downstream code cannot accidentally bypass it.

**Rationale:** Configuration objects are routinely serialized for debugging — a `printf("%+v", config)`, a `console.log(config)`, an uncaught exception that includes the config in its message — and any of those paths can publish secrets to logs, dashboards, and on-call channels. Marking sensitivity at load time and redacting in a single chokepoint (a `Redacted[T]` wrapper type, a `__repr__` override, a logger filter) closes the leak at its source.

**Verification:** Any spec or plan that introduces sensitive configuration MUST commit to a sensitivity marker and to a redaction mechanism applied at the config-loading layer. Validate flags plans that handle secrets without naming the redaction wrapper, and flags affected-files snippets that log or serialize config objects without applying the wrapper.

**Source:** OWASP Logging Cheat Sheet; "Make Illegal States Unrepresentable"

### CFG-ENV-007

> Configuration value precedence MUST be documented and consistent across the application. The standard precedence is, from highest to lowest: command-line flag → environment variable → configuration file → in-code default constant. Deviations from this order MUST be justified in `specs/system.md`.

**Rationale:** Inconsistent precedence — some values reading the env first, others reading the file first — produces baffling deployment incidents where a flag "doesn't take effect" because a stale env value silently wins. A documented, uniform order makes the resolved value predictable and debuggable.

**Verification:** Any spec or plan that introduces a configurable value with more than one source (e.g., both a flag and an env var, both an env var and a file entry) MUST describe the resolution order and confirm it follows the standard. Validate flags plans whose config-loading description departs from the standard order without an explicit justification reference in `specs/system.md`.

**Source:** Twelve-Factor App (III. Config); "Principle of Least Surprise"
