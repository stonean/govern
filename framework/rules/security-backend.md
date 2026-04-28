# Security Rules — Backend

Enforceable security rules for server-side code, APIs, data persistence, and infrastructure. These rules apply to all projects adopting governance.

Rules use RFC 2119 language: **MUST** and **MUST NOT** are enforced by the validate command (errors). **SHOULD** and **SHOULD NOT** are flagged as warnings.

## BE-AUTH — Authentication

### BE-AUTH-001 — Credential storage

Passwords MUST be hashed using Argon2id, scrypt, or bcrypt — never encrypted or stored in plaintext. Only a one-way hash is stored; the original value is never recoverable.

- **Rationale:** Encryption is reversible. If the database is compromised, encrypted passwords can be decrypted. Hashing is one-way.
- **Verification:** Code review confirms password storage uses an approved hashing algorithm with no reversible encryption.
- **Source:** OWASP Password Storage Cheat Sheet

### BE-AUTH-002 — Token storage

API tokens and bearer tokens MUST be stored as hashes (SHA-256 minimum). The raw token is shown once at creation and never stored.

- **Rationale:** Storing raw tokens allows immediate credential theft on database compromise.
- **Verification:** Data model review confirms token columns store hashes, not plaintext.
- **Source:** OWASP Authentication Cheat Sheet

### BE-AUTH-003 — Session ID properties

Session IDs MUST be generated using a cryptographically secure pseudorandom number generator (CSPRNG) with at least 128 bits of entropy. Session IDs MUST NOT contain user data, roles, or any meaningful information.

- **Rationale:** Predictable session IDs enable session hijacking. Embedded data leaks information if the ID is exposed.
- **Verification:** Code review confirms CSPRNG usage and that session IDs are opaque identifiers.
- **Source:** OWASP Session Management Cheat Sheet

### BE-AUTH-004 — Session regeneration

The server MUST issue a new session ID after authentication, privilege escalation, or any change in authorization level. The previous session ID MUST be invalidated.

- **Rationale:** Prevents session fixation attacks where an attacker sets a known session ID before the user authenticates.
- **Verification:** Test confirms session ID changes after login and privilege changes.
- **Source:** OWASP Session Management Cheat Sheet

### BE-AUTH-005 — Session expiration

Sessions MUST have both an idle timeout and an absolute timeout, enforced server-side. Client-side timeout enforcement is supplementary only.

- **Rationale:** Idle timeout limits exposure from unattended sessions. Absolute timeout bounds the lifetime of a hijacked session. Server-side enforcement cannot be bypassed.
- **Verification:** Configuration review confirms both timeouts are defined and enforced on the server.
- **Source:** OWASP Session Management Cheat Sheet

### BE-AUTH-006 — Generic authentication errors

Authentication failure responses MUST NOT reveal which credential component was invalid. The response for an invalid username, invalid password, and disabled account MUST be indistinguishable.

- **Rationale:** Specific error messages enable user enumeration — attackers discover valid usernames by varying inputs.
- **Verification:** Test confirms identical response body, status code, and timing for all failure cases.
- **Source:** OWASP Authentication Cheat Sheet

### BE-AUTH-007 — Brute force protection

The system MUST implement account lockout or rate limiting after repeated authentication failures. Lockout MUST be based on the target account, not the source IP.

- **Rationale:** IP-based lockout is trivially bypassed with distributed attacks. Account-based throttling protects the actual target.
- **Verification:** Test confirms lockout triggers after threshold and that legitimate recovery (password reset) remains available during lockout.
- **Source:** OWASP Authentication Cheat Sheet

### BE-AUTH-008 — Transport security

Authentication credentials MUST only be transmitted over TLS. Login endpoints MUST NOT be available over unencrypted connections.

- **Rationale:** Credentials transmitted in plaintext are trivially intercepted via network sniffing.
- **Verification:** Configuration review confirms TLS requirement; test confirms HTTP redirects to HTTPS or is rejected.
- **Source:** OWASP Authentication Cheat Sheet

## BE-AUTHZ — Authorization

### BE-AUTHZ-001 — Default deny

Access MUST be denied unless explicitly permitted. Authorization logic MUST NOT rely on the absence of a deny rule.

- **Rationale:** Default-allow means any missed rule silently grants access. Default-deny means any missed rule safely blocks access.
- **Verification:** Test confirms that an unauthenticated or unprivileged request to any protected endpoint receives a denial response.
- **Source:** OWASP Authorization Cheat Sheet

### BE-AUTHZ-002 — Server-side enforcement

Authorization decisions MUST be made on the server. Client-side checks are for UX only and MUST NOT be trusted.

- **Rationale:** Client-side logic can be inspected, modified, or bypassed entirely.
- **Verification:** Test confirms authorization is enforced when requests are made directly (bypassing the UI).
- **Source:** OWASP Authorization Cheat Sheet

### BE-AUTHZ-003 — Per-request validation

Authorization MUST be checked on every request, not cached from a prior request or assumed from session state.

- **Rationale:** Permissions can change between requests (role revocation, account suspension). Skipping checks creates a window where revoked access is still honored.
- **Verification:** Code review confirms middleware or framework-level authorization runs on each request.
- **Source:** OWASP Authorization Cheat Sheet

### BE-AUTHZ-004 — Privilege escalation prevention

A caller MUST NOT be able to grant permissions they do not hold. Role and permission management operations MUST enforce a ceiling rule — the caller's effective permissions bound what they can assign.

- **Rationale:** Without a ceiling rule, any user with role-management access can escalate to full privileges.
- **Verification:** Test confirms that assigning a permission the caller lacks returns an error.
- **Source:** OWASP Authorization Cheat Sheet

### BE-AUTHZ-005 — Resource-level checks

Authorization MUST be checked against the specific resource being accessed, not just the resource type. Object-level authorization MUST prevent horizontal privilege escalation.

- **Rationale:** A user authorized to read their own records should not be able to read another user's records by changing an ID.
- **Verification:** Test confirms that accessing another user's resource returns a denial response even when the caller has the permission for that resource type.
- **Source:** OWASP Authorization Cheat Sheet, OWASP API Security Top 10

### BE-AUTHZ-006 — Failure response opacity

Authorization failures SHOULD return 404 Not Found rather than 403 Forbidden for resources whose existence should not be revealed to unauthorized callers.

- **Rationale:** A 403 confirms the resource exists, which is itself information leakage. A 404 reveals nothing.
- **Verification:** Test confirms unauthorized access returns 404, not 403, for existence-sensitive resources.
- **Source:** OWASP Authorization Cheat Sheet

## BE-INPUT — Input Validation

### BE-INPUT-001 — Server-side validation

All input MUST be validated on the server before processing. Client-side validation is for UX only and MUST NOT be trusted.

- **Rationale:** Client-side validation can be disabled or bypassed by sending requests directly.
- **Verification:** Test confirms that invalid input sent directly to the API (bypassing UI) is rejected.
- **Source:** OWASP Input Validation Cheat Sheet

### BE-INPUT-002 — Allowlist over denylist

Input validation MUST use allowlists (define what is authorized) rather than denylists (block known-bad patterns).

- **Rationale:** Denylists are incomplete by definition — attackers find bypasses. Allowlists define the valid set explicitly.
- **Verification:** Code review confirms validation uses allowlists for constrained inputs (dropdowns, enums, formats).
- **Source:** OWASP Input Validation Cheat Sheet

### BE-INPUT-003 — Parameterized queries

Database queries MUST use parameterized statements or prepared statements. Query strings MUST NOT be constructed by concatenating user input.

- **Rationale:** SQL injection remains one of the most exploited vulnerabilities. Parameterized queries make injection structurally impossible.
- **Verification:** Code review confirms no string concatenation or interpolation in query construction.
- **Source:** OWASP SQL Injection Prevention Cheat Sheet

### BE-INPUT-004 — Path traversal prevention

User-supplied values MUST NOT be used directly in file paths. If file access is required, the application MUST resolve the canonical path and verify it falls within the expected directory.

- **Rationale:** Path traversal (`../../../etc/passwd`) allows reading or writing arbitrary files on the server.
- **Verification:** Test confirms that path traversal sequences in input do not access files outside the intended directory.
- **Source:** OWASP Input Validation Cheat Sheet

### BE-INPUT-005 — File upload validation

File uploads MUST validate file type using content inspection (magic bytes), not the Content-Type header or file extension alone. Uploaded files MUST be stored outside the web root. Filenames MUST be generated by the server, not taken from user input.

- **Rationale:** Headers and extensions are trivially spoofed. Storing in the web root enables direct execution. User-controlled filenames enable path traversal and overwrite attacks.
- **Verification:** Code review confirms content-based validation, storage location, and filename generation.
- **Source:** OWASP File Upload Cheat Sheet

### BE-INPUT-006 — Request size limits

The application MUST enforce maximum request body size. Requests exceeding the limit MUST be rejected with 413.

- **Rationale:** Unbounded request sizes enable denial-of-service via resource exhaustion.
- **Verification:** Configuration review confirms size limits are defined; test confirms oversized requests are rejected.
- **Source:** OWASP REST Security Cheat Sheet

## BE-DATA — Data Protection

### BE-DATA-001 — Encryption in transit

All network communication MUST use TLS 1.2 or later. Plaintext protocols MUST NOT be used for any data exchange.

- **Rationale:** Unencrypted traffic is trivially intercepted on shared networks.
- **Verification:** Configuration review confirms TLS is required; test confirms plaintext connections are rejected.
- **Source:** OWASP Cryptographic Storage Cheat Sheet

### BE-DATA-002 — Encryption at rest

Sensitive data (PII, credentials, financial data) MUST be encrypted at rest. Encryption keys MUST be stored separately from the encrypted data.

- **Rationale:** Database compromise without key access yields only ciphertext. Co-locating keys and data defeats the purpose.
- **Verification:** Data model review confirms sensitive columns are encrypted; key management review confirms separation.
- **Source:** OWASP Cryptographic Storage Cheat Sheet

### BE-DATA-003 — Secrets management

Secrets (API keys, database credentials, encryption keys) MUST NOT be hardcoded in source code, committed to version control, or stored in environment variables without a secrets management solution. Secrets MUST be injected at deployment time from a centralized secrets store.

- **Rationale:** Source code and environment variables are accessible through multiple vectors (repo access, process inspection, error pages). A secrets store provides access control, rotation, and audit.
- **Verification:** Code review confirms no hardcoded secrets; configuration review confirms secrets management integration.
- **Source:** OWASP Secrets Management Cheat Sheet

### BE-DATA-004 — Cryptographic standards

Encryption MUST use AES-256 (or AES-128 minimum) with authenticated modes (GCM or CCM). Custom cryptographic algorithms MUST NOT be used. ECB mode MUST NOT be used.

- **Rationale:** Proven algorithms have undergone extensive analysis. Custom implementations contain vulnerabilities. ECB reveals patterns in ciphertext.
- **Verification:** Code review confirms approved algorithms and modes.
- **Source:** OWASP Cryptographic Storage Cheat Sheet

### BE-DATA-005 — Key rotation

Encryption keys MUST have a defined rotation schedule. The system MUST support key rotation without downtime. Old keys MUST be retained temporarily for decrypting existing data.

- **Rationale:** Rotation limits the exposure window if a key is compromised. Downtime during rotation is operationally unacceptable. Immediate old key destruction makes existing data unreadable.
- **Verification:** Operations review confirms rotation schedule exists and has been tested.
- **Source:** OWASP Cryptographic Storage Cheat Sheet

### BE-DATA-006 — Data minimization

The system MUST NOT store sensitive data beyond its required purpose and retention period. Data no longer needed MUST be securely deleted.

- **Rationale:** Data that does not exist cannot be breached.
- **Verification:** Data model review confirms retention policies are defined for sensitive data types.
- **Source:** OWASP Cryptographic Storage Cheat Sheet

## BE-API — API Security

### BE-API-001 — Security headers

All HTTP responses MUST include the following headers:

| Header | Value |
| --- | --- |
| `Strict-Transport-Security` | `max-age=63072000; includeSubDomains; preload` |
| `X-Content-Type-Options` | `nosniff` |
| `X-Frame-Options` | `DENY` |
| `Cache-Control` | `no-store` (for authenticated responses) |
| `Content-Type` | Explicit type with charset (e.g., `text/html; charset=UTF-8`) |

- **Rationale:** Each header mitigates a specific attack class: HSTS prevents downgrade attacks, X-Content-Type-Options prevents MIME sniffing, X-Frame-Options prevents clickjacking, Cache-Control prevents sensitive data caching.
- **Verification:** Test confirms all required headers are present in responses.
- **Source:** OWASP HTTP Headers Cheat Sheet

### BE-API-002 — Information suppression

The `Server`, `X-Powered-By`, and framework-specific version headers MUST be removed or set to non-informative values.

- **Rationale:** Technology fingerprinting enables targeted attacks against known vulnerabilities in specific versions.
- **Verification:** Test confirms headers are absent or generic.
- **Source:** OWASP HTTP Headers Cheat Sheet

### BE-API-003 — CORS policy

CORS MUST be disabled if cross-origin requests are not required. When required, allowed origins MUST be explicitly listed — wildcard (`*`) MUST NOT be used for authenticated endpoints.

- **Rationale:** Wildcard CORS disables same-origin protection, allowing any site to make authenticated requests.
- **Verification:** Configuration review confirms explicit origin allowlist; test confirms wildcard is not used.
- **Source:** OWASP REST Security Cheat Sheet

### BE-API-004 — Rate limiting

All public-facing endpoints MUST implement rate limiting. Rate limit responses MUST use 429 Too Many Requests.

- **Rationale:** Unbounded request rates enable brute force attacks, credential stuffing, and denial of service.
- **Verification:** Test confirms rate limit triggers and returns 429.
- **Source:** OWASP REST Security Cheat Sheet

### BE-API-005 — HTTP method restriction

The application MUST accept only documented HTTP methods per endpoint. Undocumented methods MUST return 405 Method Not Allowed.

- **Rationale:** Unrestricted methods enable verb tampering attacks that bypass authentication or authorization.
- **Verification:** Test confirms that unsupported methods return 405.
- **Source:** OWASP REST Security Cheat Sheet

### BE-API-006 — Content type enforcement

The application MUST validate the Content-Type header on incoming requests. Mismatched content types MUST be rejected with 415 Unsupported Media Type.

- **Rationale:** Accepting unexpected content types enables injection attacks through format confusion.
- **Verification:** Test confirms that requests with wrong Content-Type are rejected.
- **Source:** OWASP REST Security Cheat Sheet

## BE-ERROR — Error Handling

### BE-ERROR-001 — No internal details in responses

Production error responses MUST NOT include stack traces, file paths, database errors, framework versions, or internal system details.

- **Rationale:** Internal details enable reconnaissance — attackers identify specific technologies and versions to target known vulnerabilities.
- **Verification:** Test confirms error responses contain only a structured error code and user-safe message.
- **Source:** OWASP Error Handling Cheat Sheet

### BE-ERROR-002 — Structured error responses

Error responses MUST use a consistent, structured format with an error code, human-readable message, and request correlation ID. The format SHOULD follow RFC 7807 Problem Details for APIs.

- **Rationale:** Consistent error formatting enables programmatic error handling and correlates errors to logs without leaking internals.
- **Verification:** Code review confirms a global error handler produces structured responses.
- **Source:** OWASP Error Handling Cheat Sheet, OWASP REST Security Cheat Sheet

### BE-ERROR-003 — Global error handler

The application MUST implement a global error handler that catches unhandled exceptions and returns a safe, structured response. Unhandled exceptions MUST NOT propagate to the client.

- **Rationale:** Missing error handlers produce default framework responses that include stack traces and internal details.
- **Verification:** Test confirms that an unexpected exception returns a structured 500 response with no internal details.
- **Source:** OWASP Error Handling Cheat Sheet

## BE-LOG — Logging and Audit

### BE-LOG-001 — Security event logging

The following events MUST be logged: authentication successes and failures, authorization failures, input validation failures, privilege changes (role/permission modifications), and session lifecycle events (creation, expiration, termination).

- **Rationale:** Security logs are the primary source for detecting attacks, investigating incidents, and meeting compliance requirements.
- **Verification:** Code review confirms logging calls for each event type.
- **Source:** OWASP Logging Cheat Sheet

### BE-LOG-002 — Sensitive data exclusion

Logs MUST NOT contain passwords, session IDs, API tokens, encryption keys, or personally identifiable information in plaintext. If a sensitive field must be referenced, log the field name only — not its value.

- **Rationale:** Logs are stored, aggregated, and often accessible to a broader set of users than the application data itself. Sensitive data in logs creates a secondary breach vector.
- **Verification:** Code review confirms sensitive fields are excluded or masked in log statements.
- **Source:** OWASP Logging Cheat Sheet

### BE-LOG-003 — Tamper protection

Log storage MUST be separate from application data storage. Log access MUST be restricted and monitored. The system SHOULD implement tamper detection on log integrity.

- **Rationale:** If an attacker gains application database access, logs stored in the same database can be modified to cover tracks.
- **Verification:** Infrastructure review confirms log storage separation and access controls.
- **Source:** OWASP Logging Cheat Sheet

### BE-LOG-004 — Audit trail for access control changes

All changes to roles, permissions, user-role assignments, and service account permissions MUST be recorded in an audit trail with actor, action, entity, timestamp, and request correlation.

- **Rationale:** Authorization changes are the highest-impact security events. Audit trails enable incident response and compliance.
- **Verification:** Test confirms audit entries are created for each authorization change type.
- **Source:** OWASP Logging Cheat Sheet

## BE-INFRA — Infrastructure

### BE-INFRA-001 — Management interface isolation

Management, administration, and monitoring interfaces for infrastructure services (databases, message brokers, caches, search engines) MUST NOT be accessible from public networks. Management ports MUST be bound to internal networks or localhost only.

- **Rationale:** Management interfaces provide privileged access to data and configuration. Public exposure makes them targets for credential stuffing and exploit attacks.
- **Verification:** Network configuration review confirms management ports are not reachable from public networks.
- **Source:** CIS Benchmarks, OWASP REST Security Cheat Sheet

### BE-INFRA-002 — Database access restriction

Database ports MUST NOT be accessible from public networks. Application-to-database connections MUST use authenticated, encrypted channels. Database accounts used by the application MUST follow least privilege — no DBA or superuser access for application accounts.

- **Rationale:** Direct database access bypasses all application-level security controls. Superuser application accounts allow attackers to escalate beyond the application's intended scope.
- **Verification:** Network configuration review confirms database port isolation; database configuration review confirms application account privileges.
- **Source:** CIS Benchmarks, OWASP SQL Injection Prevention Cheat Sheet

### BE-INFRA-003 — Dependency vulnerability scanning

Project dependencies MUST be scanned for known vulnerabilities. Dependencies with known critical or high-severity vulnerabilities MUST be updated or replaced. Scanning SHOULD be automated in CI/CD.

- **Rationale:** Dependencies are the most common source of exploitable vulnerabilities in modern applications.
- **Verification:** CI/CD configuration review confirms scanning is configured; scan results show no unaddressed critical/high findings.
- **Source:** OWASP Dependency-Check

### BE-INFRA-004 — Container secret injection

Secrets MUST NOT be embedded in container images, Dockerfiles, or container definitions. Secrets MUST be injected at runtime via environment variables from a secrets store or mounted secret volumes.

- **Rationale:** Container images are stored in registries, cached in build systems, and inspectable by anyone with registry access. Embedded secrets are trivially extracted.
- **Verification:** Dockerfile review confirms no secrets; deployment configuration confirms runtime injection.
- **Source:** OWASP Secrets Management Cheat Sheet
