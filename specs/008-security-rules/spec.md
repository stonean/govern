---
title: "008-security-rules — spec"
status: done
dependencies: [007-govern-workflow, 016-cross-cutting-rules]
tags: [security, format]
---

# 008 — Security Rules

> **Signpost:** 008 defines the *security instance* of the general rules tier later formalized in [016 — Cross-Cutting Rules](../016-cross-cutting-rules/spec.md). The rule-file format, ID conventions, and validate enforcement defined here remain the canonical reference for any future rule file (observability, performance, accessibility, etc.). See §rules in `framework/constitution.md` for the general framing of rules as a cross-cutting artifact tier alongside specs and scenarios.

Comprehensive, enforceable security rules for backend and frontend development. Distributed to adopting projects via the govern command as two files — `security-backend.md` and `security-frontend.md`. These are rules, not guidelines: the validate command checks implementation against them.

## Motivation

The constitution lists "Secure" as a guiding principle but does not operationalize it. Projects adopting governance have no concrete security rules to follow or validate against. Each project reinvents its own security posture, leading to inconsistency and gaps.

Security rules belong at the governance level because they are cross-cutting — the same rules apply regardless of language, framework, or domain. Project-specific security decisions (which auth provider, which encryption library) belong in the project's `system.md` or feature specs. Governance defines *what* must be secured and *how* to think about it; projects decide the implementation.

## Two Rule Files

Security rules are split by attack surface:

### Backend (`security-backend.md`)

Rules for server-side code, APIs, data persistence, and infrastructure integration.

**Categories:**

- **Authentication** — credential storage, session management, token handling, multi-factor considerations, constant-time comparison of secrets
- **Authorization** — permission enforcement, privilege escalation prevention, default-deny posture, mass-assignment / over-posting prevention
- **Input validation** — sanitization at system boundaries, reject-by-default, allowlisting over denylisting, parameterized queries (SQL/NoSQL/command), safe deserialization, path-traversal prevention, XXE/SSTI prevention, SSRF prevention on outbound URL fetches
- **Data protection** — encryption at rest and in transit, secrets management, PII handling, key rotation
- **API security** — rate limiting, request size limits, CORS policy, security headers, HSTS / HTTPS enforcement, cache-control for sensitive responses, redirect validation, versioned error responses that do not leak internals
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

- **Rule ID** — short identifier formatted as `{surface}-{category}-{NNN}` (e.g., `BE-AUTHN-001`, `FE-XSS-001`) for reference in specs, plans, and validate output. See **ID stability** below.
- **Rule statement** — one sentence declaring what must or must not happen
- **Rationale** — why this rule exists (threat it mitigates)
- **Verification** — how the validate command or a reviewer checks compliance (code pattern, test requirement, configuration check, or documentation commitment — see **Verification phrasing** below)

Rules use RFC 2119 language: MUST, MUST NOT, SHOULD, SHOULD NOT. MUST/MUST NOT rules are enforced by validate; SHOULD/SHOULD NOT rules are flagged as warnings.

### ID stability

Rule IDs are permanent. Once an ID is assigned, it must never be renumbered, even if the rule is moved within the file, edited, or deprecated. New rules get the next available sequence number for their category. Deprecated rules retain their ID with a `DEPRECATED` label and a removal target version. Sequence numbers are not reused after a rule is fully removed. Reading a file top-to-bottom may show non-sequential IDs after deprecations and reorders — IDs are anchors, not a reading order.

### Verification phrasing

Rules whose enforcement happens *outside* the code repository (runtime configuration, infrastructure, deployment) phrase their **Verification** field as a documentation commitment rather than a code pattern. Example:

> `BE-API-002` — TLS for production traffic
> Verification: `specs/system.md` or a deployment-related spec MUST describe how TLS is terminated (load balancer, application, sidecar) and the expected protocol/cipher policy. Validate flags absence of any TLS handling commitment in the project's specs.

Validate does not probe running infrastructure or parse deployment configs — it confirms the project has documented its approach. Code-pattern Verification (e.g., "every handler accepting user input MUST call a validator before persisting") remains the norm for rules whose enforcement lives in the repository.

## Govern Integration

Both files are added to the govern file manifest with `update` strategy — governance-owned, always overwritten with the latest version on re-run.

| Source Path | Destination Path |
| --- | --- |
| `framework/rules/security-backend.md` | `specs/security-backend.md` |
| `framework/rules/security-frontend.md` | `specs/security-frontend.md` |

Source files live in the governance framework under `framework/rules/`, alongside the constitution and other ship-everything artifacts. Destination is the project's `specs/` directory, alongside `system.md`, `errors.md`, and `events.md` — the other cross-cutting global specs. Projects that do not have a frontend can pin `specs/security-frontend.md` in `.governance.toml` to skip it. Backend rules apply to all projects.

**Local edits will be overwritten.** Because both files use the `update` strategy, any local edits to `specs/security-backend.md` or `specs/security-frontend.md` are discarded on the next `/govern` run. To diverge from the governance-owned ruleset, pin the file in `.governance.toml` — pinned files are never updated. Editing rule files directly without pinning is a path to losing work.

## Validate Integration

The validate command gains security rule checking:

- During validation, the validate command reads the applicable security rule files
- For each MUST/MUST NOT rule, validate checks whether the implementation complies
- Violations are reported as errors (blocking)
- SHOULD/SHOULD NOT violations are reported as warnings (non-blocking)
- Rule IDs are included in validate output for traceability

The validate command does not perform static analysis or code scanning. It checks whether the spec, plan, and implementation *address* the applicable rules — for example, whether a spec that handles user input includes input validation acceptance criteria, or whether a plan that stores credentials specifies hashed storage.

Rules apply **contextually** based on what the spec or plan actually addresses. A rule that no spec or plan content exercises is silently inert — no finding emitted, no opt-out required. A project with no database, for example, naturally produces no findings for data-at-rest rules because no spec mentions data persistence.

Rules with a **runtime or infrastructure dimension** (e.g., "TLS must be enabled in production") are not a separate validation category. They are verified the same way every other rule is — by checking that a spec, plan, or `system.md` documents how the project addresses the rule. Validate does not probe a running server or parse Terraform/Helm/Ansible; it confirms that the project has *thought about* the rule and recorded its approach. Rules whose enforcement happens outside the code repository should phrase their **Verification** field as a documentation commitment (e.g., "system.md MUST describe how TLS is terminated and the expected protocol/cipher policy"). The trade-off is deliberate: a spec that *says* TLS is enabled but where production is actually misconfigured will pass validate. Validate findings are necessary but not sufficient — runtime enforcement is the job of deployment tooling, infra-as-code review, and observability.

## Brownfield Adoption

When `/govern` installs the security rule files in a project that already has feature specs, the adopter inherits a backlog: existing specs were written before these rules existed and almost certainly do not address all of them. To avoid dumping that backlog directly on validate (where it would block the next pipeline gate), 008 hooks into the existing brownfield workflow defined by 011 — bugs and findings flow through `specs/inbox.md` and are routed via `/{project}:groom`.

### Trigger

Govern runs a one-time security audit when **both** conditions hold after the file manifest has been processed:

- Either `specs/security-backend.md` or `specs/security-frontend.md` was newly **created** by the manifest pass (i.e., not already present, not just updated).
- The project contains at least one feature spec directory under `specs/` matching the `NNN-*` pattern.

When neither condition holds — greenfield adoption with no existing specs, or a routine re-run where the rule files already exist — the audit is silently skipped. There is no per-run audit; the trigger is "rule file newly installed in a project with existing specs."

### Audit logic

For each newly created rule file:

1. Load the rule file, applying the same integrity checks validate uses (well-formed headings, required fields, valid IDs, no duplicates). If the file fails to load, govern reports the load failure and skips the audit for that file — same posture as validate.
2. For each MUST/MUST NOT and SHOULD/SHOULD NOT rule whose Verification trigger fires against any artifact under `specs/NNN-*/` (`spec.md`, `spec-and-plan.md`, `plan.md`, scenario files), produce a finding.
3. Append each finding to `specs/inbox.md` as a new item.

### Inbox item format

Each finding becomes a one-line inbox item:

```text
- [ ] {Rule ID}: {affected artifact path} does not address — {one-line summary}
```

Examples:

```text
- [ ] BE-AUTHN-001: specs/004-user-login/spec.md does not name a memory-hard password hashing algorithm
- [ ] FE-XSS-002: specs/007-comment-rendering/spec.md does not specify an output encoding strategy
```

Prefixing every line with the rule ID makes related findings group naturally during `/{project}:groom` and gives the adopter a stable handle for cross-referencing.

### Idempotency

Audit findings are deduplicated against existing inbox content. Before appending, govern scans `specs/inbox.md` for any line beginning with `- [ ] {Rule ID}: {affected artifact path}` (the line up to the first em-dash). If a matching line exists, the new finding is skipped. This makes the audit safe to re-run if a user deletes and re-installs a rule file or otherwise re-triggers the "newly created" path.

Inbox items already grommed by the user (lines that have been removed or rewritten by `/{project}:groom`) are not re-emitted — once the adopter has triaged a finding, governance does not resurrect it.

### Reporting

After the audit completes, govern's post-scaffolding output gains a new line in the summary:

```text
{N} security audit items added to specs/inbox.md. Run /{project}:groom to triage.
```

When `N == 0` (no new findings), the line is omitted.

### Why not block validate instead?

A simpler alternative would be: validate runs on existing specs after govern adoption and emits errors as usual. Rejected — for brownfield projects, that produces an immediate validate failure that blocks every pipeline gate until the adopter fixes dozens of legacy specs. The inbox model gives the adopter a real-world path: triage at their own pace, treating each finding as a backlog item rather than a release blocker.

The inbox approach also reuses 011's existing groom workflow rather than inventing baseline files or suppression mechanisms — there is one place backlog items live (`specs/inbox.md`) and one tool to process them (`/{project}:groom`), regardless of whether the source is a bug report, a brownfield spec gap, or a security audit finding.

## Constitution Reference

The "Secure" principle in the constitution gains a reference to the security rule files:

> Protect sensitive data through industry standards and best practices. See `specs/security-backend.md` and `specs/security-frontend.md` for enforceable rules.

This connects the principle to its operational detail without duplicating content.

## Versioning and Evolution

- Rules are added, modified, or deprecated in the governance repo
- Adopting projects receive updates on the next `/govern` re-run
- Deprecated rules are marked with a `DEPRECATED` label and removal target version rather than deleted immediately, giving projects time to adjust
- New rules are announced in governance commit messages so adopters can review changes

## Edge Cases

How validate behaves when the inputs are unusual:

- **Neither rule file present.** If a project has no `specs/security-backend.md` and no `specs/security-frontend.md` (e.g., both pinned out, or files manually deleted), validate emits a warning: `No security rule files found, skipping security checks.` Validate continues; the security check is non-blocking in this case.
- **Only one file present.** If a project has only one of the two files (e.g., backend-only project that pinned the frontend file out, or vice versa), validate runs over the present file and emits no finding for the missing one. This is the common case for non-fullstack projects.
- **Malformed rule file.** A rule file is malformed if any rule is missing a required field (ID, statement, rationale, verification), if any rule's ID does not match the `{surface}-{category}-{NNN}` format, or if the file fails to parse. Validate **blocks** with an error: `Malformed security rule file {path} at {location}: {reason}`. The accompanying file is treated as unloadable; no rules from that file are applied. Rationale: validate's findings must rest on accurate rule data — partial or guessed-at parsing produces unreliable findings.
- **Stale rule reference.** A spec or plan references a rule ID that does not exist in the current rule files (the rule was removed upstream after `/govern` updated the file). Validate **blocks** with an error: `Spec at {path} references unknown rule {ID}.` The adopter must update or remove the reference before validate will pass. Rationale: stale references silently rot if tolerated.
- **Reference to DEPRECATED rule.** A spec or plan references a rule ID that exists but is marked `DEPRECATED`. Validate emits a warning (not an error): `Spec at {path} references deprecated rule {ID}; targeted for removal in {version}.` The reference still satisfies the rule for the duration of the deprecation window. Rationale: deprecation needs a real grace window between the label landing and references becoming invalid; a hard block at deprecation collapses the window to zero.
- **Local edits overwritten by `/govern`.** The govern command overwrites `specs/security-{backend,frontend}.md` on every run because they use the `update` strategy. This is normal govern behavior, not a security-rules-specific concern, but is called out in **Govern Integration** above so adopters know to use `.governance.toml` pinning rather than local edits when they need to diverge.
- **Duplicate rule IDs in a file.** If two rules in the same file share an ID (a botched edit broke the never-renumber discipline), validate **blocks** with an error: `Duplicate rule ID {ID} in {file}; refusing to load.` The whole file is skipped to prevent ambiguous references — validate cannot tell which of the two rules a spec's reference points to. Rationale: same as malformed file — accurate rule data is non-negotiable.

## Acceptance Criteria

### Rule Files

- [x] `framework/rules/security-backend.md` exists in the governance framework with categorized, numbered rules
- [x] `framework/rules/security-frontend.md` exists in the governance framework with categorized, numbered rules
- [x] Every rule has an ID, statement, rationale, and verification method
- [x] Rule IDs follow the format `{surface}-{category}-{NNN}` with `{surface}` ∈ `{BE, FE}` and `{NNN}` zero-padded starting at `001`
- [x] Rules use RFC 2119 language to distinguish enforced (MUST/MUST NOT) from advisory (SHOULD/SHOULD NOT)

### Govern Integration

- [x] Both files appear in the govern file manifest with `update` strategy
- [x] The govern command fetches `framework/rules/security-backend.md` and writes it to `specs/security-backend.md` in the project
- [x] The govern command fetches `framework/rules/security-frontend.md` and writes it to `specs/security-frontend.md` in the project
- [x] Re-running govern updates both files to the latest governance version
- [x] Projects can pin either file in `.governance.toml` to skip updates

### Validate Integration

- [x] The validate command reads `specs/security-backend.md` and `specs/security-frontend.md` when present in the project
- [x] MUST/MUST NOT violations are reported as errors (blocking)
- [x] SHOULD/SHOULD NOT violations are reported as warnings (non-blocking)
- [x] Rule IDs appear in validate output for each finding
- [x] Rules apply contextually — a rule that no spec or plan content exercises produces no finding

### Edge-case behavior

- [x] Validate emits a warning and continues when no security rule files are present
- [x] Validate runs only over the present file when one of the two is pinned out, with no finding for the missing file
- [x] Validate blocks with an error on a malformed rule file (missing required field, ID format violation, parse failure)
- [x] Validate blocks with an error on a spec/plan reference to an unknown rule ID
- [x] Validate emits a warning (not an error) on a spec/plan reference to a `DEPRECATED` rule ID
- [x] Validate blocks with an error when a rule file contains duplicate IDs

### Brownfield Adoption

- [x] On a govern run where a security rule file is newly created AND `specs/NNN-*` directories exist, govern audits the existing specs against the rule and writes one inbox item per finding to `specs/inbox.md`
- [x] On a greenfield run (no existing `specs/NNN-*` directories), the audit is silently skipped
- [x] On a routine re-run (rule files already present), the audit is silently skipped
- [x] Inbox items follow the format `- [ ] {Rule ID}: {affected artifact path} does not address — {one-line summary}`
- [x] Audit findings are deduplicated against existing inbox content (no duplicate items emitted on re-trigger)
- [x] Govern's post-scaffolding output reports the count of new audit items added (omitted when zero)

### Constitution Reference

- [x] The "Secure" principle references `specs/security-backend.md` and `specs/security-frontend.md`

## Open Questions

None — all resolved during clarification.

## Resolved Questions

1. **Severity levels beyond MUST/SHOULD** — No new tiers. Keep the RFC 2119 MUST/SHOULD distinction only. MUST/MUST NOT violations are errors (blocking); SHOULD/SHOULD NOT violations are warnings (non-blocking). A "critical / blocks merge" tier adds nothing over MUST since CI gating on validate already blocks. An "acknowledge" tier is a documentation requirement, better expressed as a SHOULD whose rationale says "if you choose not to follow this, record the deviation in the spec." Trade-off accepted: all SHOULDs are equal in v1; if prioritization becomes painful, address it in validate output, not by adding tiers.

2. **Rule ID granularity** — Per-rule IDs with a never-renumber policy. Format: `{surface}-{category}-{NNN}` where `{surface}` is `BE` or `FE`, `{category}` is a short uppercase abbreviation (backend: `AUTHN`, `AUTHZ`, `INPUT`, `DATA`, `API`, `LOG`, `DEPS`, `ERR`; frontend: `XSS`, `CSRF`, `STORAGE`, `AUTHN`, `CSP`, `DEPS`, `PII`), and `{NNN}` is zero-padded starting at `001`. `AUTHN` covers authentication, `AUTHZ` covers authorization — distinct abbreviations to prevent collision. Numbering is per-category, not global. Once an ID is assigned it is permanent for the lifetime of the rule — reorganization moves rules but never renumbers them. Deprecated rules keep their ID with a `DEPRECATED` label and removal target. Sequence numbers are never reused, even after a rule is fully removed. Trade-off accepted: reading the file top-to-bottom may show non-sequential IDs after deprecations; IDs are anchors, not a reading order.

3. **Per-category opt-out** — No category-level opt-out for v1. Whole-file pinning (via `.governance.toml`) handles the case where an entire surface does not apply (e.g., backend-only projects pin `security-frontend.md`). Within a surface, rules apply *contextually* — a rule that no spec or plan content exercises is silently inert, so a project with no database naturally produces no data-at-rest findings without needing to opt out. Adding category-level opt-outs would require a declaration syntax, validate logic to honor it, and a migration story when the project's stack evolves — substantial surface area for a use case that does not yet exist. Trade-off accepted: if validate produces a false-positive finding because contextual matching catches something the project doesn't actually do, there is no opt-out — fix the matching, not the policy.

4. **Runtime/infrastructure rules** — No special handling. Runtime rules (e.g., "TLS must be enabled") are verified the same way every other rule is — validate checks that the project's spec, plan, or `system.md` documents how the rule is addressed. Validate does not probe running infrastructure or parse deployment configs; it confirms the project has *thought about* the rule and recorded its approach. Such rules should phrase their **Verification** field as a documentation commitment (e.g., "system.md MUST describe how TLS is terminated"). Trade-off accepted: a spec that says TLS is enabled but where production is misconfigured will pass validate — runtime enforcement is the job of deployment tooling, infra-as-code review, and observability, not of a text-based pipeline gate.

## References

Declared dependencies for this spec, surfaced here so the dependency-derivation generator (`scripts/gen-spec-deps.sh`) sees them in the body.

- [007-govern-workflow](../007-govern-workflow/spec.md)
