# Security Rules — Backend

Enforceable security rules for server-side code, APIs, data persistence, and integration boundaries. These rules apply to all projects adopting governance.

Rules use RFC 2119 language: **MUST** / **MUST NOT** are enforced by the validate command (errors); **SHOULD** / **SHOULD NOT** are flagged as warnings.

Rule IDs follow the format `BE-{CATEGORY}-{NNN}` and are permanent — once assigned, an ID is never renumbered, even if the rule is moved within the file or deprecated. Categories: `AUTHN` (authentication), `AUTHZ` (authorization), `INPUT` (input validation), `DATA` (data protection), `API` (API security), `LOG` (logging and audit), `DEPS` (dependency management), `ERR` (error handling). See `specs/008-security-rules/data-model.md` for the full schema.

## BE-AUTHN — Authentication

### BE-AUTHN-001

> Passwords MUST be hashed using a memory-hard algorithm (Argon2id, scrypt, or bcrypt) before persistence — never encrypted, never stored in plaintext.

**Rationale:** Encryption is reversible — a database breach plus key access yields plaintext credentials. One-way hashing is irreversible by design. Memory-hard algorithms specifically resist GPU-accelerated cracking.

**Verification:** Any spec or plan that introduces credential storage (search keywords: `password`, `credential`, `auth`, `login`) MUST name the hashing algorithm. Validate flags persistence paths that use `MD5`, `SHA-1`, `SHA-256`, `SHA-512`, plain `crypt`, "encrypted", or that omit the algorithm question entirely. Salting and cost factor parameters MUST be specified or referenced.

**Source:** OWASP Password Storage Cheat Sheet

### BE-AUTHN-002

> API tokens and bearer tokens MUST be stored as hashes (SHA-256 minimum). The raw token MUST be returned to the user once at creation and never persisted.

**Rationale:** Storing raw tokens enables immediate credential theft on database compromise. Hashing tokens turns the database into a verification store rather than a credential vault.

**Verification:** Any spec or plan that introduces API tokens, personal access tokens, or service tokens MUST commit to hashed storage and a one-time-display pattern at issuance. Validate flags token-issuance specs that describe storing the token itself in the database.

**Source:** OWASP Authentication Cheat Sheet

### BE-AUTHN-003

> Session IDs MUST be generated using a cryptographically secure pseudorandom generator (CSPRNG) with at least 128 bits of entropy and MUST NOT contain user data, role information, or any meaningful identifier.

**Rationale:** Predictable session IDs enable hijacking via guessing. Embedded data leaks information if the ID is logged, surfaced in errors, or transmitted insecurely.

**Verification:** Any spec or plan that introduces sessions MUST commit to (a) a CSPRNG source for ID generation and (b) opaque, content-free IDs. Validate flags specs that propose deriving session IDs from user attributes, timestamps, or sequential counters, and flags specs silent on the ID generation source.

**Source:** OWASP Session Management Cheat Sheet

### BE-AUTHN-004

> The server MUST issue a new session ID after authentication, after privilege escalation, and after any change in authorization level. The previous session ID MUST be invalidated.

**Rationale:** Without regeneration, an attacker who fixes a known session ID before a victim authenticates inherits the post-login session — the canonical session-fixation attack.

**Verification:** Any spec or plan that introduces authentication, privilege change, or impersonation flows MUST commit to session ID regeneration at each transition. Validate flags auth flows that omit regeneration or that explicitly preserve the pre-auth session ID.

**Source:** OWASP Session Management Cheat Sheet

### BE-AUTHN-005

> Sessions MUST have both an idle timeout and an absolute timeout, both enforced server-side. Client-side timeout enforcement is supplementary only.

**Rationale:** Idle timeout limits exposure from unattended sessions; absolute timeout bounds the lifetime of a hijacked session. Client-side enforcement is bypassable by direct API calls.

**Verification:** Any spec or plan that introduces sessions MUST name both timeout values (idle, absolute) and confirm server-side enforcement. Validate flags session specs that omit either timeout or that rely on client-side expiration alone.

**Source:** OWASP Session Management Cheat Sheet

### BE-AUTHN-006

> Authentication failure responses MUST NOT reveal which credential component was invalid. The response for invalid username, invalid password, and disabled account MUST be indistinguishable in body, status code, and timing.

**Rationale:** Differential responses enable user enumeration — attackers discover valid usernames by varying inputs and observing response differences. Timing differences are exploited the same way as message differences.

**Verification:** Any spec or plan that describes login or password-reset endpoints MUST commit to a generic failure response and to constant-time handling that does not differ across failure modes. Validate flags auth-flow specs that propose different messages or status codes for "user not found" vs. "wrong password."

**Source:** OWASP Authentication Cheat Sheet

### BE-AUTHN-007

> The system MUST throttle authentication attempts (account lockout, exponential back-off, or rate limiting). Throttling SHOULD be account-scoped so distributed attacks against a single account cannot bypass IP-only limits, and SHOULD layer per-IP and per-device limits as defense in depth.

**Rationale:** Per-IP throttling alone is trivially bypassed with distributed botnets. Account-scoped throttling protects the actual target. Layered scopes contain both targeted and credential-stuffing patterns.

**Verification:** Any spec or plan covering login, password reset, or token-issuance endpoints MUST commit to a throttling mechanism, name its scope (account, IP, device), and describe legitimate-recovery paths during lockout. Validate flags auth specs without a documented throttle.

**Source:** OWASP Authentication Cheat Sheet

### BE-AUTHN-008

> Authentication credentials MUST be transmitted only over TLS. Login and credential-change endpoints MUST NOT be served over plain HTTP.

**Rationale:** Plaintext transmission exposes credentials to network observers. Serving login over HTTP — even with an HTTPS redirect — leaves a first-request window where credentials may be intercepted. Refer to `BE-DATA-001` for the required TLS protocol version.

**Verification:** Any spec or plan covering authentication endpoints MUST commit to TLS-only access, including refusal of plain-HTTP requests at the edge. Validate flags auth-endpoint specs that allow HTTP fallback or that omit the transport question.

**Source:** OWASP Authentication Cheat Sheet

### BE-AUTHN-009

> Comparison of secrets, tokens, MACs, signatures, and password hashes MUST use a constant-time comparison primitive.

**Rationale:** Standard string equality short-circuits on the first differing byte, leaking the matching prefix length via response timing. Constant-time comparison closes timing side channels for token validation, signature verification, and HMAC checks.

**Verification:** Any spec or plan that introduces secret/token/HMAC/signature comparison MUST commit to a constant-time primitive (e.g., `crypto.timingSafeEqual`, `hmac.compare_digest`, `subtle.ConstantTimeCompare`, `MessageDigest.isEqual`). Validate flags specs that compare secrets with `==`, `===`, `equals`, `strcmp`, or any short-circuiting comparison.

**Source:** OWASP Authentication Cheat Sheet, NIST SP 800-63B

## BE-AUTHZ — Authorization

### BE-AUTHZ-001

> Access MUST be denied unless explicitly permitted. Authorization logic MUST NOT rely on the absence of a deny rule.

**Rationale:** Default-allow plus selective-deny is brittle — every new endpoint inherits open access unless a developer remembers to add a deny rule. Default-deny plus explicit allow is the only durable posture.

**Verification:** Any spec or plan that introduces a protected endpoint, resource, or operation MUST describe the explicit allow check (middleware, decorator, gateway policy). Validate flags any commitment phrased as "everything except X is public" or "deny these specific paths."

**Source:** OWASP Authorization Cheat Sheet

### BE-AUTHZ-002

> Authorization decisions MUST be made on the server. Client-supplied claims (roles, permissions, tenant IDs) MUST be revalidated against the server's authoritative store on every request.

**Rationale:** Client-side authorization can be inspected, bypassed, or spoofed by direct API calls. Trusting client-supplied identity claims (e.g., a `tenant_id` in a request body) without server-side revalidation enables horizontal privilege escalation across tenants.

**Verification:** Any spec or plan describing role-based, attribute-based, or multi-tenant access control MUST describe how the server validates the claim against its own data — not just trusts the input. Validate flags specs that pass tenant/role IDs from the client into data queries without a documented revalidation step.

**Source:** OWASP Authorization Cheat Sheet

### BE-AUTHZ-003

> Authorization MUST be checked on every request. Permission state MUST NOT be cached from a prior request or assumed from session establishment.

**Rationale:** Permissions can change between requests — role revocation, account suspension, privilege downgrade. Caching permissions creates a window where revoked access continues to be honored.

**Verification:** Any spec or plan that introduces a permission-checked operation MUST commit to per-request authorization (middleware, framework guard, dependency-injected check). Validate flags specs that describe loading permissions at session start and not revalidating on subsequent requests.

**Source:** OWASP Authorization Cheat Sheet

### BE-AUTHZ-004

> A caller MUST NOT be able to grant permissions they do not themselves hold. Role-management and permission-grant operations MUST enforce a ceiling rule — the caller's effective permissions bound what they can assign.

**Rationale:** Without a ceiling rule, any user with role-management access can escalate themselves or others to higher privilege. The ceiling rule contains lateral and vertical privilege escalation in a single check.

**Verification:** Any spec or plan that introduces role grants, permission assignments, or admin invitations MUST commit to a ceiling check at the grant operation. Validate flags admin-management specs that describe assigning roles without a documented permissions-superset check on the caller.

**Source:** OWASP Authorization Cheat Sheet

### BE-AUTHZ-005

> Authorization MUST be checked against the specific resource being accessed (object-level authorization), not just the resource type or endpoint.

**Rationale:** A user authorized to read their own records MUST NOT be able to read another user's records by changing an ID parameter. This is OWASP API1:2023 — Broken Object Level Authorization (BOLA), the most-exploited API vulnerability.

**Verification:** Any spec or plan that introduces resource-by-id endpoints (`/users/:id`, `/orders/:id`, `/projects/:id/files/:fileId`) MUST commit to an ownership or resource-scoped authorization check, not just a coarse "is this user authenticated" check. Validate flags resource endpoints whose authorization commitment is only at the type level (e.g., "any authenticated user can call /orders/:id").

**Source:** OWASP Authorization Cheat Sheet, OWASP API Security Top 10 (API1:2023)

### BE-AUTHZ-006

> Authorization failures SHOULD return `404 Not Found` rather than `403 Forbidden` for resources whose existence should not be revealed to unauthorized callers.

**Rationale:** A `403` confirms the resource exists, which is itself information leakage useful for reconnaissance and enumeration. A `404` reveals nothing about existence.

**Verification:** Any spec or plan that introduces authorization on existence-sensitive resources (private documents, internal user records, draft content) SHOULD commit to `404` responses for unauthorized callers. Validate emits a warning when a spec explicitly proposes `403` for such resources without justification.

**Source:** OWASP Authorization Cheat Sheet

### BE-AUTHZ-007

> Request handlers that bind user input to persistence or domain objects MUST use an explicit allowlist of writable fields. Mass assignment of arbitrary input to model fields is forbidden.

**Rationale:** Frameworks that auto-bind request bodies to model fields (Rails `params`, Spring `@RequestBody`, Express body-spread) let users set fields they shouldn't — admin flags, internal IDs, ownership references, billing tier — when those fields aren't excluded. The allowlist is the only safe default; the denylist (strong parameters with field exclusion) inevitably misses new fields added later.

**Verification:** Any spec or plan that describes accepting structured input (form bodies, JSON requests) and persisting or assigning it to a domain object MUST commit to an allowlist of acceptable fields per endpoint. Validate flags any commitment to "auto-bind", "spread the body", "accept all fields", or "use the request body directly" without a corresponding allowlist mechanism (DTO, schema-validated input type, explicit `pick`/`select` of fields).

**Source:** OWASP API Security Top 10 (API3:2023 — Broken Object Property Level Authorization), OWASP Mass Assignment Cheat Sheet

## BE-INPUT — Input Validation

### BE-INPUT-001

> All input crossing a system boundary MUST be validated server-side against an explicit schema before processing. Client-side validation is for UX only and MUST NOT be trusted.

**Rationale:** Boundary validation gives the rest of the system a strong invariant — once data is in, it conforms. Client-side validation is bypassable by direct API calls.

**Verification:** Any spec or plan that accepts input from clients, third-party APIs, message queues, or file ingestion MUST name a server-side validation mechanism (JSON Schema, type system with runtime guards, dedicated validator library) and commit to running it at the boundary. Validate flags input-handling paths without a named server-side validation step.

**Source:** OWASP Input Validation Cheat Sheet

### BE-INPUT-002

> Input validation MUST use allowlists (define what is acceptable) for constrained inputs. Denylists (block known-bad patterns) MUST NOT be used as the sole defense.

**Rationale:** Denylists are inherently incomplete — every blocked pattern requires a developer to remember it. Allowlists fail closed: anything not explicitly allowed is rejected.

**Verification:** When a spec or plan describes validation for constrained inputs (enums, formats, file types, slugs, identifiers), it MUST express the rule as an allowlist (regex anchoring acceptable characters, explicit enum, MIME type allowlist). Validate flags rules expressed only as "reject if matches X" or "block these characters" without a corresponding allowlist.

**Source:** OWASP Input Validation Cheat Sheet

### BE-INPUT-003

> Database queries MUST use parameterized statements, prepared statements, or an ORM that parameterizes by default. Query strings MUST NOT be constructed via string concatenation, interpolation, or template literals containing user input.

**Rationale:** SQL injection remains one of the most exploited vulnerabilities in web applications. Parameterization makes injection structurally impossible — the database driver, not the application, handles escaping. The same rule applies to NoSQL operator injection (MongoDB `$where`, dynamic field projections) and command execution (`exec`, `system`, shell-out).

**Verification:** Any spec or plan that introduces database access MUST describe the query mechanism (parameterized statements, prepared statements, query builder with parameter binding, ORM defaults). Validate flags any commitment to building queries via concatenation, `f-string`/`.format()`, template literals interpolating request fields, or dynamic operator construction from client input.

**Source:** OWASP SQL Injection Prevention Cheat Sheet

### BE-INPUT-004

> User-supplied values MUST NOT be used directly in filesystem paths. Filesystem operations MUST resolve the canonical path and verify it falls within the expected base directory before opening the file.

**Rationale:** Path traversal (`../../../etc/passwd`) lets attackers read or write arbitrary files when user input flows into file paths without canonicalization-and-base-check.

**Verification:** Any spec or plan that opens, reads, writes, or serves files based on user input (filename parameter, upload destination, template path) MUST commit to path canonicalization (e.g., `path.resolve` + `startsWith(baseDir)`, `os.path.realpath` + `commonpath`, language-equivalent) before opening. Validate flags file-handling specs that take user input without a documented canonicalization step.

**Source:** OWASP Input Validation Cheat Sheet

### BE-INPUT-005

> File uploads MUST validate file type by content inspection (magic bytes / signature), not by Content-Type header or filename extension alone. Uploaded files MUST be stored outside the web root, and stored filenames MUST be generated server-side, not taken from user input.

**Rationale:** Client-supplied Content-Type and filename are trivially spoofed. Storing in the web root enables direct execution of uploaded content. User-controlled filenames enable path traversal and overwrite attacks against existing files.

**Verification:** Any spec or plan that introduces file upload MUST commit to (a) content-based file-type validation, (b) storage outside the document root, and (c) server-generated filenames. Validate flags upload specs missing any of these three.

**Source:** OWASP File Upload Cheat Sheet

### BE-INPUT-006

> Inputs that contribute to resource consumption (request bodies, file uploads, list lengths, page sizes, regex inputs) MUST have explicit upper bounds. Requests exceeding limits MUST be rejected with `413 Payload Too Large` (or the format-appropriate equivalent).

**Rationale:** Unbounded inputs enable denial-of-service through memory exhaustion, payload bombs, and ReDoS. Bounds turn DoS attempts into clean rejections.

**Verification:** Any spec or plan that accepts variable-size input (file uploads, JSON bodies, list parameters, search queries, paginated endpoints) MUST commit to a maximum size or count and to the rejection response. Validate flags input descriptions without explicit limits, particularly for uploads, bulk endpoints, and search queries that accept user-supplied regex.

**Source:** OWASP REST Security Cheat Sheet

### BE-INPUT-007

> When the server fetches a URL supplied or influenced by user input, the destination MUST be validated against an allowlist of acceptable hosts or schemes, AND outbound requests MUST be denied by default to internal address ranges (loopback `127.0.0.0/8`, link-local `169.254.0.0/16`, RFC 1918 `10.0.0.0/8` / `172.16.0.0/12` / `192.168.0.0/16`, IPv6 equivalents, and cloud metadata endpoints).

**Rationale:** Server-Side Request Forgery (SSRF) lets attackers make the server fetch arbitrary URLs, often as a stepping stone to cloud metadata services (`169.254.169.254` for AWS/GCP/Azure), internal admin panels, or arbitrary intranet probing. OWASP A10:2021 / API6:2023.

**Verification:** Any spec or plan that includes server-side URL fetching, webhook callbacks, image proxies, RSS pulls, or any user-supplied URL retrieval MUST commit to (a) host or scheme allowlisting, AND (b) outbound network restrictions or DNS-resolution-time blocks for internal address ranges. Validate flags outbound-fetch features without both commitments. Cloud metadata-address denial MUST be named explicitly when the project deploys to a cloud provider.

**Source:** OWASP SSRF Prevention Cheat Sheet, OWASP API Security Top 10 (API6:2023)

### BE-INPUT-008

> Deserialization of untrusted input MUST use a format and parser that cannot execute code as a side effect of parsing.

**Rationale:** Pickle (Python), `ObjectInputStream` (Java), `unserialize` (PHP), `Marshal.load` (Ruby), and similar binary deserialization formats can execute attacker-controlled code as part of parsing — a direct path to remote code execution. JSON, MessagePack, Protobuf, and similar data-only formats do not have this property.

**Verification:** Any spec or plan that ingests data from clients, queues, files, or third-party APIs MUST name the serialization format. Validate flags any commitment to Pickle, `ObjectInputStream`, PHP `unserialize`, or `Marshal.load` for untrusted input. JSON, MessagePack, and Protobuf are acceptable. YAML is acceptable only when the spec names a safe-loading mode explicitly (e.g., `yaml.safe_load` in Python, `SafeYAML` in Ruby).

**Source:** OWASP Deserialization Cheat Sheet

### BE-INPUT-009

> XML parsers MUST be configured to disable external entity expansion, DTD processing, and parameter entity expansion before parsing untrusted input.

**Rationale:** Default XML parser configurations in many languages allow XXE — external entity expansion that can read local files (`file:///etc/passwd`), perform SSRF via entity URLs, or trigger denial-of-service via billion-laughs attacks. Disabling these features at parser construction is the only reliable mitigation.

**Verification:** Any spec or plan that parses XML from untrusted sources (SOAP services, SAML responses, document uploads, RSS, configuration imports) MUST commit to a hardened parser configuration (`disallow-doctype-decl`, `external-general-entities=false`, `external-parameter-entities=false`, language-equivalent settings) at parser construction. Validate flags XML-parsing specs without an explicit hardening commitment, and flags reliance on default parser settings.

**Source:** OWASP XML External Entity Prevention Cheat Sheet

### BE-INPUT-010

> User input MUST NOT be concatenated or interpolated into template strings rendered by a server-side template engine. User input MAY be passed only as bound variables to a template whose source is controlled by the application.

**Rationale:** Server-Side Template Injection (SSTI) in Jinja, Twig, ERB, Handlebars, Velocity, etc. enables arbitrary code execution or template-sandbox escape when user input becomes part of the template *text* (not template *data*). Treating user input as data — values bound to variables — is safe; treating it as template source is not.

**Verification:** Any spec or plan that uses server-side template rendering MUST describe how user input flows into templates: only as bound variables, never as concatenated template text. Validate flags any commitment to building template strings dynamically from user input, or rendering templates whose source is partially user-controlled.

**Source:** OWASP Server-Side Template Injection guidance, PortSwigger SSTI research

## BE-DATA — Data Protection

### BE-DATA-001

> All network communication MUST use TLS 1.2 or later. Plaintext protocols MUST NOT be used for any data exchange in production.

**Rationale:** Unencrypted traffic is trivially intercepted on shared networks. TLS 1.0 and 1.1 are formally deprecated by IETF (RFC 8996); TLS 1.2 is the floor and TLS 1.3 is preferred.

**Verification:** Any spec or plan covering network communication, edge configuration, or `system.md` MUST commit to TLS 1.2+ across all listening surfaces (HTTPS, gRPC, message brokers, database connections). Validate flags specs that allow plain-text protocols, that name TLS 1.0/1.1, or that omit the TLS version question entirely.

**Source:** OWASP Cryptographic Storage Cheat Sheet, RFC 8996, NIST SP 800-52r2

### BE-DATA-002

> Sensitive data (PII, credentials, financial data, authentication tokens) MUST be encrypted at rest. Encryption keys MUST be stored separately from the encrypted data — co-locating keys and ciphertext defeats the purpose.

**Rationale:** Database compromise without key access yields only ciphertext — the attacker cannot read the protected fields. Storing keys alongside the data (in the same database, the same host, the same backup) means a single compromise reveals both.

**Verification:** Any spec or plan that persists sensitive data MUST name the encryption mechanism (envelope encryption with KMS, transparent database encryption, application-level field encryption) AND describe the key location (KMS, HSM, separate keystore, mounted secret). Validate flags persistence specs that say only "encrypted" without naming the mechanism, and flags any commitment to storing keys in the same database as the data they encrypt.

**Source:** OWASP Cryptographic Storage Cheat Sheet

### BE-DATA-003

> Secrets (API keys, database credentials, encryption keys, signing keys, third-party tokens) MUST NOT be hardcoded in source code, committed to version control, embedded in container images, or stored in environment-variable defaults shipped with the application. Secrets MUST be sourced at runtime from a secrets management system or runtime injection by the orchestrator.

**Rationale:** Committed secrets propagate everywhere — git history, CI logs, image layers, developer laptops, error pages — and can never be fully scrubbed. Externalized secrets via a secret manager or runtime injection are the only reliable model.

**Verification:** Any spec or plan that introduces secrets MUST describe how they are sourced at runtime (vault/secret manager, mounted file, runtime environment variable injected by the orchestrator). Validate flags any commitment to embedding secrets in `.env` files committed to git, `Dockerfile` `ENV` directives, default config files in the repo, or container image layers.

**Source:** OWASP Secrets Management Cheat Sheet

### BE-DATA-004

> Encryption MUST use AES-256 (or AES-128 minimum) with an authenticated mode (GCM, CCM) or ChaCha20-Poly1305. Custom cryptographic primitives MUST NOT be used. ECB mode MUST NOT be used for any payload longer than one block.

**Rationale:** Proven algorithms have undergone extensive analysis. Custom implementations contain vulnerabilities by default. Authenticated modes (AEAD) prevent tampering as well as preserving confidentiality. ECB reveals patterns in ciphertext (the canonical "ECB Penguin").

**Verification:** Any spec or plan that introduces encryption MUST name the algorithm and mode. Validate flags custom or unnamed algorithms, and flags any commitment to ECB, unauthenticated CBC, or pre-AEAD constructions for new code. Legacy compatibility deviations MUST be explicitly justified in the spec.

**Source:** OWASP Cryptographic Storage Cheat Sheet, NIST SP 800-38D

### BE-DATA-005

> Encryption keys and signing keys MUST be rotatable. The rotation procedure (cadence, mechanism, decryption of in-flight data, retirement of old keys) MUST be documented in `specs/system.md` or a dedicated spec. The system MUST support rotation without downtime.

**Rationale:** Keys leak. Without a documented rotation procedure, the only response to suspected compromise is "rebuild the system" — which never happens, so the compromised key stays in use. Documented rotation is a precondition for survivable key compromise.

**Verification:** Any spec or plan that introduces encryption-at-rest, JWT signing, HMAC signatures, or any other long-lived keys MUST describe (a) rotation cadence, (b) versioning/dual-key reads during transition, (c) retirement of old keys. Validate flags key-using specs that omit the rotation question.

**Source:** OWASP Cryptographic Storage Cheat Sheet

### BE-DATA-006

> Personally Identifiable Information (PII) MUST be minimized at collection, scoped to a documented purpose, and deletable on user request. Data retention periods MUST be defined for each PII field.

**Rationale:** PII you don't collect is PII you don't have to protect, leak, or be subpoenaed for. Minimization is the strongest privacy control. Purpose limitation, deletion, and retention bounds are baseline GDPR/CCPA requirements.

**Verification:** Any spec or plan that collects PII (names, emails, addresses, phone numbers, government IDs, biometrics, geolocation) MUST describe (a) the minimum data set required, (b) the documented purpose for each field, (c) the deletion path, and (d) the retention period. Validate flags PII-collecting specs that omit any of these.

**Source:** OWASP Cryptographic Storage Cheat Sheet, GDPR Article 5

### BE-DATA-007

> Application-to-database accounts MUST follow least privilege. Application accounts MUST NOT have DBA, superuser, or schema-modification privileges in production.

**Rationale:** A compromised application with full database privileges enables schema corruption, data exfiltration, and privilege escalation beyond the application's intended scope. Least-privilege accounts contain the blast radius of an SQL injection or application compromise to operations the application legitimately needs.

**Verification:** Any spec or plan covering database access, deployment configuration, or `system.md` MUST commit to (a) separate application accounts per environment, (b) the minimum privilege set the application requires (typically `SELECT`/`INSERT`/`UPDATE`/`DELETE` on its own schema, no `CREATE`/`DROP`/`ALTER`), and (c) a separate account for migrations with elevated privileges that the application itself does not use. Validate flags database access specs that omit the privilege question or that name a `root`/`admin`/`postgres` account for application use.

**Source:** CIS Database Benchmarks, OWASP SQL Injection Prevention Cheat Sheet

### BE-DATA-008

> Secrets MUST NOT be embedded in container images, Dockerfiles, or container-build manifests. Secrets MUST be injected at runtime via environment variables sourced from a secrets manager, mounted secret volumes, or platform-native secret APIs.

**Rationale:** Container images are stored in registries, cached in build systems, and inspectable by anyone with registry pull access. Embedded secrets are trivially extracted via `docker history` or layer inspection. This rule complements `BE-DATA-003` for the container-specific case.

**Verification:** Any spec or plan that describes containerization or container deployment MUST commit to runtime secret injection. Validate flags `Dockerfile` content (or equivalent build manifests) that includes `ENV SECRET=...`, `ARG SECRET=...` baked into layers, or `COPY` of secret files into the image.

**Source:** OWASP Secrets Management Cheat Sheet, NIST SP 800-190 (Container Security)

## BE-API — API Security

### BE-API-001

> All HTTP responses MUST include a documented set of security headers covering: HSTS, content-type options, frame protection, referrer policy, content security policy (for HTML responses), and cache control for sensitive responses.

The minimum required headers and their values:

| Header | Value | Applies to |
| --- | --- | --- |
| `Strict-Transport-Security` | `max-age=63072000; includeSubDomains; preload` | All responses |
| `X-Content-Type-Options` | `nosniff` | All responses |
| `X-Frame-Options` or CSP `frame-ancestors` | `DENY` (or equivalent CSP directive) | HTML responses |
| `Referrer-Policy` | `strict-origin-when-cross-origin` (or stricter) | All responses |
| `Content-Security-Policy` | Project-defined CSP (see frontend rules for guidance) | HTML responses |
| `Cache-Control` | `no-store` | Authenticated and sensitive-data responses |
| `Content-Type` | Explicit type with charset (e.g., `text/html; charset=UTF-8`) | All responses |

**Rationale:** Each header converts the browser into a layered defense. HSTS prevents downgrade attacks; `X-Content-Type-Options` prevents MIME sniffing; frame-ancestors prevents clickjacking; `Referrer-Policy` controls Referer leakage; CSP defends against XSS and mixed content; `Cache-Control: no-store` keeps sensitive responses out of shared caches and the back-button cache.

**Verification:** Any spec or plan that introduces an HTTP response (especially HTML) MUST commit to setting these headers, ideally at the framework or reverse-proxy layer for uniform coverage. Validate flags response specs that omit any of the headers above, and specifically flags HTML-serving specs without a CSP commitment.

**Source:** OWASP HTTP Headers Cheat Sheet, MDN Web Docs

### BE-API-002

> The `Server`, `X-Powered-By`, and framework-specific version headers MUST be removed or set to non-informative values.

**Rationale:** Technology fingerprinting enables targeted attacks against known vulnerabilities in specific framework versions. Suppression denies attackers a reconnaissance shortcut.

**Verification:** Any spec or plan covering edge configuration or web-server configuration MUST commit to suppressing or genericizing these headers. Validate flags response-handling specs that explicitly include these headers or that omit the suppression question.

**Source:** OWASP HTTP Headers Cheat Sheet

### BE-API-003

> CORS policy MUST be explicit. Wildcard `Access-Control-Allow-Origin: *` MUST NOT be used for any endpoint that returns user data, and MUST NOT be combined with `Access-Control-Allow-Credentials: true`.

**Rationale:** A `*` CORS allow on a credentialed or user-data endpoint exposes the user's data to any origin that can convince a browser to make the request. The CORS spec forbids `*` plus credentials; applications still misconfigure servers around it.

**Verification:** Any spec or plan that describes a browser-facing API MUST name the allowed origins explicitly. Validate flags any commitment to `*` origin combined with cookies or auth headers, flags `*` origin on user-scoped endpoints, and flags CORS commitments that omit the origin allowlist.

**Source:** OWASP REST Security Cheat Sheet

### BE-API-004

> All public-facing endpoints MUST implement rate limiting. The throttle scope (per-IP, per-user, per-token) and the limits MUST be documented. Rate-limited responses MUST use `429 Too Many Requests` with a `Retry-After` header.

**Rationale:** Unbounded request rates enable credential stuffing, brute-force authentication, scraping, and resource-exhaustion DoS. Rate limiting is the universal first-line mitigation.

**Verification:** Any spec or plan that introduces a public endpoint MUST commit to a rate-limit policy — what is throttled, the threshold, and the response on exceedance. Validate flags public endpoints without this commitment, especially authentication, password reset, and search endpoints.

**Source:** OWASP REST Security Cheat Sheet

### BE-API-005

> The application MUST accept only documented HTTP methods per endpoint. Undocumented methods MUST return `405 Method Not Allowed`.

**Rationale:** Unrestricted methods enable verb-tampering attacks that bypass authentication or authorization configured for specific verbs (e.g., `GET` is filtered but `POST` is not).

**Verification:** Any spec or plan that introduces HTTP endpoints MUST commit to method allowlisting. Validate flags endpoint specs that allow arbitrary methods or that omit the method-restriction question.

**Source:** OWASP REST Security Cheat Sheet

### BE-API-006

> The application MUST validate the `Content-Type` header on incoming requests. Mismatched or unexpected content types MUST be rejected with `415 Unsupported Media Type`.

**Rationale:** Accepting unexpected content types enables injection attacks via format confusion (e.g., a JSON endpoint that also accepts XML may be vulnerable to XXE even when the JSON path is hardened).

**Verification:** Any spec or plan that introduces request-accepting endpoints MUST commit to content-type validation. Validate flags endpoint specs that accept multiple unrelated formats without explicit per-format hardening.

**Source:** OWASP REST Security Cheat Sheet

### BE-API-007

> Management, administration, and monitoring interfaces for infrastructure services (databases, message brokers, caches, search engines, debug consoles) MUST NOT be reachable from public networks. Management ports MUST be bound to internal networks, VPN, or localhost only.

**Rationale:** Management interfaces provide privileged access to data, configuration, and runtime control. Public exposure makes them targets for credential stuffing, exploit attacks, and direct compromise of the underlying service.

**Verification:** Any spec or plan covering deployment, networking, or `system.md` MUST commit to restricting management interface exposure. Validate flags deployment specs that bind management ports (e.g., RabbitMQ management UI on `15672`, Elasticsearch on `9200`, Redis on `6379`, database admin tools) to public interfaces or that omit the network-exposure question.

**Source:** CIS Benchmarks, OWASP REST Security Cheat Sheet

### BE-API-008

> Database ports MUST NOT be reachable from public networks. Application-to-database connections MUST use authenticated, encrypted channels.

**Rationale:** Direct database access bypasses every application-level security control. Public exposure of database ports is a direct path to data theft. This rule pairs with `BE-DATA-007` (account privileges) — together they enforce both network and account-level least privilege.

**Verification:** Any spec or plan covering deployment or `system.md` MUST commit to (a) database ports unreachable from public networks (private subnet, security group, firewall rule), and (b) TLS-encrypted application-to-database connections. Validate flags deployment specs that allow public database access or that describe database connections without naming TLS.

**Source:** CIS Benchmarks, OWASP SQL Injection Prevention Cheat Sheet

### BE-API-009

> Backend redirect endpoints (e.g., `/redirect?url=...`, post-login redirects, OAuth callbacks, share-link expansions, any user-controlled destination URL) MUST validate the destination against an allowlist of acceptable hosts or paths. Open redirects are forbidden.

**Rationale:** Open redirects facilitate phishing (a legitimate-looking link on the application's domain that redirects to attacker.example), OAuth-flow manipulation, and credential-stealing landing pages. Allowlist validation contains the redirect to known-safe destinations.

**Verification:** Any spec or plan that includes a redirect endpoint, OAuth callback, post-login next-URL, share link, or user-controlled destination URL MUST commit to validating the destination against an allowlist (host allowlist, path allowlist, signed URL parameter). Validate flags redirect features without this commitment.

**Source:** OWASP Unvalidated Redirects and Forwards Cheat Sheet

## BE-ERR — Error Handling

### BE-ERR-001

> Production error responses MUST NOT include stack traces, file paths, database error messages, framework versions, or any internal implementation detail. Verbose errors MUST appear only in non-production environments or behind authentication.

**Rationale:** Internal details in production responses are a reconnaissance gift — attackers learn the framework, language version, file layout, and sometimes credentials in connection strings.

**Verification:** Any spec or plan that describes error handling MUST commit to environment-conditional error formatting — verbose in dev, sanitized in production. Validate flags error-handling specs that do not distinguish environments or that emit internal-detail responses in production paths.

**Source:** OWASP Error Handling Cheat Sheet

### BE-ERR-002

> Error responses MUST use a consistent, structured format with a stable error code, a human-readable message, and a request correlation ID. The format SHOULD follow RFC 7807 (Problem Details for HTTP APIs).

**Rationale:** Stable error codes let clients react programmatically (retry, surface a localized message, branch on specific failures). Correlation IDs let support debug a user's report against the server's logs without exposing internals to the user.

**Verification:** Any spec or plan that describes error responses MUST commit to the structured format with code + message + correlation ID. Validate flags error-response specs that emit only a string message or a raw exception name.

**Source:** OWASP Error Handling Cheat Sheet, RFC 7807

### BE-ERR-003

> The application MUST implement a global error handler that catches unhandled exceptions and returns a structured `5xx` response. Unhandled exceptions MUST NOT propagate to the client.

**Rationale:** Without a global handler, uncaught exceptions produce default framework responses that include stack traces, source paths, and environment details — a direct path to reconnaissance.

**Verification:** Any spec or plan covering the request-handling pipeline or `system.md` MUST commit to a global exception handler that produces a structured response. Validate flags pipeline specs that omit this commitment.

**Source:** OWASP Error Handling Cheat Sheet

## BE-LOG — Logging and Audit

### BE-LOG-001

> The following events MUST be logged: authentication successes and failures, authorization failures, input validation failures, privilege changes (role/permission modifications), session lifecycle events, and sensitive-data access for auditable resources.

**Rationale:** Security logs are the primary source for detecting attacks, investigating incidents, and meeting compliance requirements. Coverage of these events is the baseline for any meaningful incident response.

**Verification:** Any spec or plan that introduces authentication, authorization, validation, or privilege management MUST commit to logging the named events. Validate flags specs that omit the logging question for any of these surfaces.

**Source:** OWASP Logging Cheat Sheet

### BE-LOG-002

> Logs MUST NOT contain passwords, full authentication tokens, session IDs, encryption keys, raw PII, full payment card numbers, or other sensitive field values. If a sensitive field must be referenced, log only the field name (or a redacted/masked form), not its value.

**Rationale:** Logs propagate through aggregation systems, archival storage, on-call dashboards, and developer machines. Anything in a log is widely visible and difficult to redact retroactively. Sensitive data in logs is an exfiltration path waiting to happen.

**Verification:** Any spec or plan that introduces logging MUST commit to redaction or exclusion of sensitive fields. Validate flags logging specs that include request/response bodies in raw form, or that mention "log everything" without redaction commitments.

**Source:** OWASP Logging Cheat Sheet

### BE-LOG-003

> Log storage MUST be separate from application data storage. Log access MUST be restricted and monitored. The system SHOULD implement tamper detection on log integrity (write-once storage, cryptographic chaining, or external archival).

**Rationale:** If an attacker gains application database access, logs stored in the same database can be modified to cover tracks. Separation plus access control plus tamper detection escalate the attacker effort required to hide.

**Verification:** Any spec or plan covering logging infrastructure or `system.md` MUST commit to log storage separation and access control. Tamper detection is a `SHOULD` flagged as advisory when missing.

**Source:** OWASP Logging Cheat Sheet

### BE-LOG-004

> All changes to roles, permissions, user-role assignments, and service-account permissions MUST be recorded in an audit trail with actor, action, target entity, timestamp, and request correlation ID.

**Rationale:** Authorization changes are the highest-impact security events — a single bad role grant can hand over the system. Audit trails enable incident response and compliance reporting (SOC 2, ISO 27001, HIPAA).

**Verification:** Any spec or plan that introduces role management, permission grants, or admin invitations MUST commit to audit-log entries with the five required fields. Validate flags admin-management specs without this commitment.

**Source:** OWASP Logging Cheat Sheet

## BE-DEPS — Dependency Management

### BE-DEPS-001

> Project dependencies MUST be scanned for known vulnerabilities on every CI run. Dependencies with known critical or high-severity vulnerabilities MUST be updated, replaced, or have a documented exception. Scanning SHOULD be automated in CI/CD.

**Rationale:** Vulnerable dependencies are the most common path to compromise in modern web stacks. Continuous scanning catches CVEs as they're disclosed; a documented action policy ensures findings are addressed rather than ignored.

**Verification:** Any spec or plan covering CI, deployment, or `system.md` MUST name the vulnerability scanner (Snyk, Dependabot, Trivy, `pip-audit`, `npm audit`, OWASP Dependency-Check, etc.) AND the policy on findings (fail the build, file an issue, alert on-call). Validate flags CI/dependency specs that omit either.

**Source:** OWASP Dependency-Check, OWASP Top 10 (A06:2021 — Vulnerable and Outdated Components)

### BE-DEPS-002

> Production dependencies MUST be pinned to specific versions (or version + integrity hash) in the lockfile or manifest used for production builds. Floating versions (`latest`, `^`, `~`, range constraints) MUST NOT determine what ships to production.

**Rationale:** Floating versions admit supply-chain attacks via a compromised release of a dependency, and make production debugging guesswork (the same `package.json` resolves to different versions on different days). Pinned versions plus an explicit upgrade process keep deployments deterministic and auditable.

**Verification:** Any spec or plan covering build artifacts or deployment MUST commit to (a) pinned versions in lockfiles or pinned-version manifests for production builds, and (b) a documented upgrade process. Validate flags deployment specs that allow floating dependencies in production or that omit the lockfile question.

**Source:** OWASP A06:2021, npm/pnpm/yarn lockfile documentation, supply-chain attack research
