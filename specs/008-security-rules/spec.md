# 008 — Security Rules

**Status:** draft
**Dependencies:** 007-adopt-workflow

Comprehensive, enforceable security rules for backend and frontend development. Distributed to adopting projects via the govern command as two files — `security-backend.md` and `security-frontend.md`. These are rules, not guidelines: the validate command checks implementation against them.

## Motivation

The constitution lists "Secure" as a guiding principle but does not operationalize it. Projects adopting governance have no concrete security rules to follow or validate against. Each project reinvents its own security posture, leading to inconsistency and gaps.

Security rules belong at the governance level because they are cross-cutting — the same rules apply regardless of language, framework, or domain. Project-specific security decisions (which auth provider, which encryption library) belong in the project's `system.md` or feature specs. Governance defines *what* must be secured and *how* to think about it; projects decide the implementation.

## Two Rule Files

Security rules are split by attack surface:

### Backend (`security-backend.md`)

Rules for server-side code, APIs, data persistence, and infrastructure integration.

**Categories:**

- **Authentication** — credential storage, session management, token handling, multi-factor considerations
- **Authorization** — permission enforcement, privilege escalation prevention, default-deny posture
- **Input validation** — sanitization at system boundaries, reject-by-default, allowlisting over denylisting
- **Data protection** — encryption at rest and in transit, secrets management, PII handling, key rotation
- **API security** — rate limiting, request size limits, CORS policy, security headers, versioned error responses that do not leak internals
- **Logging and audit** — security-relevant events must be logged, sensitive data must never appear in logs, audit trails for access control changes
- **Dependency management** — known-vulnerability scanning, pinned versions, supply chain considerations
- **Error handling** — no stack traces or internal details in production responses, structured error codes

### Frontend (`security-frontend.md`)

Rules for browser-side code, UI rendering, and client-server interaction.

**Categories:**

- **Cross-site scripting (XSS)** — output encoding, Content Security Policy, trusted types, no inline scripts
- **Cross-site request forgery (CSRF)** — token-based protection, SameSite cookie attributes
- **Secure storage** — no secrets, tokens, or PII in localStorage/sessionStorage; cookie security attributes (HttpOnly, Secure, SameSite)
- **Authentication UX** — secure token handling in the client, session expiration behavior, redirect validation after login
- **Content security** — CSP headers, subresource integrity, frame protection
- **Dependency management** — known-vulnerability scanning, pinned versions, no dynamically loaded third-party scripts without integrity checks
- **Sensitive data handling** — mask or redact PII in the UI, no sensitive data in URL parameters or browser history

## Rule Format

Each rule within a file follows a consistent structure:

- **Rule ID** — short identifier (e.g., `BE-AUTH-001`, `FE-XSS-001`) for reference in specs, plans, and validate output
- **Rule statement** — one sentence declaring what must or must not happen
- **Rationale** — why this rule exists (threat it mitigates)
- **Verification** — how the validate command or a reviewer checks compliance (code pattern, test requirement, configuration check)

Rules use RFC 2119 language: MUST, MUST NOT, SHOULD, SHOULD NOT. MUST/MUST NOT rules are enforced by validate; SHOULD/SHOULD NOT rules are flagged as warnings.

## Govern Integration

Both files are added to the govern file manifest with `update` strategy — governance-owned, always overwritten with the latest version on re-run.

| Source Path | Destination Path |
| --- | --- |
| `security-backend.md` | `security-backend.md` |
| `security-frontend.md` | `security-frontend.md` |

Projects that do not have a frontend can pin `security-frontend.md` in `.governance.toml` to skip it. Backend rules apply to all projects.

## Validate Integration

The validate command gains security rule checking:

- During validation, the validate command reads the applicable security rule files
- For each MUST/MUST NOT rule, validate checks whether the implementation complies
- Violations are reported as errors (blocking)
- SHOULD/SHOULD NOT violations are reported as warnings (non-blocking)
- Rule IDs are included in validate output for traceability

The validate command does not perform static analysis or code scanning. It checks whether the spec, plan, and implementation *address* the applicable rules — for example, whether a spec that handles user input includes input validation acceptance criteria, or whether a plan that stores credentials specifies hashed storage.

## Constitution Reference

The "Secure" principle in the constitution gains a reference to the security rule files:

> Protect sensitive data through industry standards and best practices. See `security-backend.md` and `security-frontend.md` for enforceable rules.

This connects the principle to its operational detail without duplicating content.

## Versioning and Evolution

- Rules are added, modified, or deprecated in the governance repo
- Adopting projects receive updates on the next `/govern` re-run
- Deprecated rules are marked with a `DEPRECATED` label and removal target version rather than deleted immediately, giving projects time to adjust
- New rules are announced in governance commit messages so adopters can review changes

## Acceptance Criteria

### Rule Files

- [ ] `security-backend.md` exists at the governance repo root with categorized, numbered rules
- [ ] `security-frontend.md` exists at the governance repo root with categorized, numbered rules
- [ ] Every rule has an ID, statement, rationale, and verification method
- [ ] Rules use RFC 2119 language to distinguish enforced (MUST/MUST NOT) from advisory (SHOULD/SHOULD NOT)

### Govern Integration

- [ ] Both files appear in the govern file manifest with `update` strategy
- [ ] The govern command fetches and places both files in the project root
- [ ] Re-running govern updates both files to the latest governance version
- [ ] Projects can pin either file in `.governance.toml` to skip updates

### Validate Integration

- [ ] The validate command reads security rule files when present in the project
- [ ] MUST/MUST NOT violations are reported as errors
- [ ] SHOULD/SHOULD NOT violations are reported as warnings
- [ ] Rule IDs appear in validate output for each finding

### Constitution Reference

- [ ] The "Secure" principle references the security rule files

## Open Questions

- Should rules have severity levels beyond the MUST/SHOULD distinction? For example, a critical tier for rules that, if violated, block merge entirely versus rules that require acknowledgment.
- How granular should rule IDs be? One ID per category (e.g., `BE-AUTH`) or one per individual rule (e.g., `BE-AUTH-001`, `BE-AUTH-002`)? Per-rule IDs allow precise references but create more churn when rules are reorganized.
- Should there be a mechanism for projects to declare which rule categories apply? For example, a project with no database might want to skip data-at-rest encryption rules rather than pinning the entire backend file.
- How should the validate command handle rules that require runtime or infrastructure checks (e.g., "TLS must be enabled")? These cannot be verified from code alone. Should they be excluded from automated validation and left to manual review?
