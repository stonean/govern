# Configuration Rules

Enforceable rules for operator-tunable values, named constants, and environment variables. These rules apply to all projects adopting `govern`, regardless of whether the code is backend or frontend — configuration is the same problem on both surfaces.

Rules use RFC 2119 language: **MUST** / **MUST NOT** are enforced by the validate command (errors); **SHOULD** / **SHOULD NOT** are flagged as warnings.

Rule IDs follow the format `CFG-{CATEGORY}-{NNN}` and are permanent — once assigned, an ID is never renumbered, even if the rule is moved within the file or deprecated. Categories: `CONST` (constants), `ENV` (environment variables). See `specs/017-derive-dont-ask/data-model.md` for the full schema.

## CFG-CONST — Constants

### CFG-CONST-001

> Shared constants — values used across multiple modules — MUST live in a centralized location (e.g., `shared/constants/`) rather than being duplicated across modules.

**Rationale:** Cross-cutting defaults that drift between modules produce silent inconsistencies — a timeout treated as 30s in one place and 60s in another. A single location makes the canonical value findable and auditable.

**Verification:** Any spec or plan that introduces a value used by more than one module (timeouts, sizes, thresholds, rate limits, format strings, well-known headers, protocol versions) MUST name the centralized constants location it will live in. Validate flags plans that introduce cross-module values without naming the shared-constants location, and flags duplicated literal definitions in plan affected-files snippets.

**Source:** Twelve-Factor App (III. Config); "Don't Repeat Yourself"

### CFG-CONST-002

> Module-local constants — values used only within a single module — MUST live in that module's own constants file, not in the shared constants location.

**Rationale:** Co-locating a single module's constants with that module keeps the module self-contained and avoids coupling unrelated modules through a shared import. The shared constants location stays focused on values that genuinely cross modules.

**Verification:** Any spec or plan that introduces a named constant scoped to one module MUST place it in that module's own constants file, not in the shared location. Validate flags plans that propose adding single-module values to a `shared/constants/` path.

### CFG-CONST-003

> Operator-tunable values (timeouts, retry counts, batch sizes, thresholds, rate limits, expiry durations) MUST be backed by a named constant or an environment variable. They MUST NOT appear as bare literals in business logic.

**Rationale:** Bare literals scattered across the codebase are invisible to operators, hard to audit, and easy to leave inconsistent during tuning. A single named source of truth makes the value findable, changeable, and auditable.

**Verification:** Any spec or plan that introduces operator-tunable behavior MUST commit to a named constant or env var for each value. Validate flags plan affected-files snippets that show numeric or string literals of operator-tunable shape (durations, counts, thresholds, rate limits) without a constant or env var lookup. Ordinary literals used for local logic — loop indices, intermediate calculations, string formatting within a function — are out of scope.

## CFG-ENV — Environment variables

### CFG-ENV-001

> Every environment variable MUST have a default fallback defined as a named constant. The variable MUST be read once at startup and the value cached; per-call reads from `os.environ` (or equivalent) are forbidden.

**Rationale:** Repeated env reads are a silent dependency on process state, slow hot paths, and make the default invisible to readers. Reading once at startup and falling back to a constant produces predictable behavior, makes the default discoverable, and keeps the runtime fast.

**Verification:** Any spec or plan that introduces an env var MUST commit to a named default constant and to startup-time resolution. Validate flags plans that propose env vars without naming the default constant or that show per-call env reads in affected-files snippets.

**Source:** Twelve-Factor App (III. Config)

### CFG-ENV-002

> `.env.example` MUST contain every environment variable the application reads, each with a descriptive comment and a safe placeholder value.

**Rationale:** Operators discover required configuration by reading `.env.example`. Variables introduced in code but absent from the example produce silent runtime failures in fresh deployments and obscure the application's true configuration surface.

**Verification:** Any spec or plan that introduces an env var MUST include adding the variable to `.env.example` as part of its tasks. Validate flags plans that introduce env vars without a corresponding `.env.example` change in affected-files.

### CFG-ENV-003

> Every required environment variable MUST be validated at startup. The application MUST fail fast — exit non-zero with a clear error message naming the variable — when a required variable cannot be resolved (neither environment nor default available).

**Rationale:** Unvalidated config produces partial-failure modes that surface only at first use of the variable, often deep in a request path. Fail-fast at startup turns a confusing intermittent error into an obvious deployment-time failure.

**Verification:** Any spec or plan that introduces required env vars MUST commit to startup-time validation that names the failing variable in its error message. Validate flags plans that introduce env vars without a startup-validation step.

**Source:** Twelve-Factor App (III. Config); "Fail Fast" pattern

### CFG-ENV-004

> Environment variables holding time values MUST include the unit in the variable name (`_MS`, `_SECONDS`, `_MINUTES`, `_HOURS`). The corresponding default constant MUST also make the unit explicit (e.g., `DEFAULT_SHUTDOWN_TIMEOUT_SECONDS = 30`).

**Rationale:** Unit-less time variables produce off-by-1000x bugs — treating milliseconds as seconds, or vice versa. Naming the unit at the source of truth makes the unit unmissable to readers, operators, and future maintainers.

**Verification:** Any spec or plan that introduces a time-valued env var MUST use a unit suffix in both the variable name and the default constant name. Validate flags plans that propose `*_TIMEOUT`, `*_INTERVAL`, `*_DELAY`, `*_TTL`, etc. without a unit suffix.

**Source:** IEC 60027 (units of measurement)
