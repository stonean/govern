# System

<!-- This is a living document describing your project's architecture.
     Establish it early with core architecture decisions, then update as shared
     infrastructure evolves. Include only the sections that apply to your project
     and remove the rest. -->

## Configuration

<!-- How the application is configured. Example:

All configuration is read from environment variables at startup.
Secrets are injected via the deployment platform and never committed to source.

| Variable | Required | Default | Purpose |
| --- | --- | --- | --- |
| `DATABASE_URL` | yes | — | Primary database connection string |
| `LOG_LEVEL` | no | `info` | Logging verbosity |

-->

## Application Lifecycle

<!-- Startup sequence, initialization order, and shutdown behavior. Example:

1. Load configuration from environment
2. Connect to database and run migrations
3. Initialize cache connection
4. Start background workers
5. Begin accepting requests

-->

## Request Lifecycle

<!-- The path a request or message takes through the system. Example:

1. Request received
2. Middleware chain: logging → authentication → rate limiting → routing
3. Handler processes request
4. Response returned

-->

## Multi-tenancy

<!-- Remove this section if your project is not multi-tenant.

Describe how tenancy is scoped and enforced. Example:

Tenant is identified by the `X-Tenant-ID` header, validated against the session.
All database queries are scoped to the tenant via a shared query filter.
Background jobs carry tenant context through a metadata field.

-->

## Shared Infrastructure

<!-- Packages, modules, or services shared across features. Example:

| Package | Purpose |
| --- | --- |
| `shared/database` | Connection pool and query helpers |
| `shared/middleware` | Common middleware stack |
| `shared/logging` | Structured logger configuration |

-->

## Module Pattern

<!-- How application modules are structured and isolated. Example:

Each feature module exposes a public API through a single entry point.
Modules may not import from other modules directly — shared code lives
in the shared infrastructure packages listed above.
Dependencies are injected at initialization, not imported globally.

-->
