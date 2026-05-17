---
title: "008-security-rules — plan"
---

# 008 — Security Rules Plan

## Overview

Operationalize the constitution's "Secure" principle by shipping two rule files (`security-backend.md`, `security-frontend.md`), wiring them into govern's distribution manifest, extending validate to enforce them, and updating the constitution to point at them. The work is entirely prompt-and-data — markdown rule files, an extended validate prompt, and a constitution edit. No application code.

## Technical Decisions

### Rule files live at `framework/rules/`

Per spec, sources are `framework/rules/security-backend.md` and `framework/rules/security-frontend.md`. This anchors the framework's `framework/rules/` location promised in `CLAUDE.md` ("domain rule sets adopted projects can reference (security-backend, security-frontend, …)") — currently a documented-but-empty area until 008 lands.

### Project destination is `specs/`

Per spec, govern writes both files to `specs/security-{backend,frontend}.md` in adopted projects. Sits alongside `system.md`, `errors.md`, `events.md` — the other cross-cutting global specs. Keeps the project root clean and groups all "applies project-wide" docs together.

### Rule entry format is heading-anchored markdown

Each rule is a level-3 heading whose text is the Rule ID, followed by a statement (block-quoted), and then bold-prefixed `**Rationale:**` and `**Verification:**` paragraphs. Example:

```markdown
### BE-AUTHN-001

> Credentials MUST be hashed with a memory-hard algorithm (Argon2id, scrypt, or bcrypt) before persistence.

**Rationale:** Plaintext or fast-hashed credentials enable mass account compromise after a database breach.

**Verification:** Any spec or plan that introduces credential storage MUST specify the hashing algorithm by name. Validate searches for credential/password/auth keywords and flags persistence paths that do not name a memory-hard hash.
```

This format is grep-friendly (`grep '^### BE-' framework/rules/security-backend.md` returns the rule index), human-readable, and parseable by validate's prompt without a custom format. Categories are level-2 headings; the Rule ID heading sits under its category.

Alternative considered: YAML frontmatter per rule. Rejected — adds complexity, fights markdown's natural reading flow, and the heading-anchor approach already gives each rule a stable URL fragment.

### Starter set: real, threat-grounded rules covering OWASP Top 10 + API Top 10 essentials

Backend has 8 categories, frontend has 7. v1 ships ~35 backend + ~21 frontend = ~56 rules total. Each rule is a real, threat-grounded MUST or MUST NOT (occasionally SHOULD) — not a stub. The registry is designed for extension; subsequent specs or PRs add rules incrementally without changing the format.

Coverage targets were selected by walking OWASP Top 10 (2021), OWASP API Security Top 10 (2023), and the CWE Top 25, then mapping each item to the appropriate category. Items that are commonly cited and frequently exploited get explicit rules in v1; items that are emerging or use-case-specific (WebSockets, Service Workers, software supply chain, container hardening, API inventory) are deferred for incremental additions.

Concrete v1 coverage targets (representative, may shift slightly during writing):

| Surface | Category | Targeted rules |
| --- | --- | --- |
| BE | AUTHN | Hashed credential storage; session token handling; MFA consideration; constant-time comparison of secrets |
| BE | AUTHZ | Default-deny; explicit permission checks at every entry point; privilege escalation prevention; mass-assignment / over-posting prevention |
| BE | INPUT | Validation at boundaries; allowlist over denylist; size limits; parameterized queries (SQL/NoSQL/command injection); SSRF prevention on outbound URL fetches; safe deserialization of untrusted data; path-traversal prevention on filesystem ops; XXE prevention on XML parsing; SSTI prevention on template rendering |
| BE | DATA | Encryption at rest commitment; secrets management; PII handling; key rotation |
| BE | API | Rate limiting; CORS policy; security headers; HSTS / HTTPS-only enforcement; `Cache-Control: no-store` for sensitive responses; redirect endpoint validation; error responses do not leak internals |
| BE | LOG | Sensitive data never in logs; audit trail for access changes |
| BE | DEPS | Vulnerability scanning; pinned versions |
| BE | ERR | No stack traces in production; structured error codes |
| FE | XSS | Output encoding; CSP; no inline scripts |
| FE | CSRF | Token-based protection; SameSite cookies |
| FE | STORAGE | No secrets/PII in localStorage; cookie security attributes |
| FE | AUTHN | Token storage; session expiration; redirect validation |
| FE | CSP | CSP header presence; SRI for third-party scripts |
| FE | DEPS | Vulnerability scanning; pinned versions |
| FE | PII | UI masking/redaction; no sensitive data in URLs |

#### Coverage rationale for the additions

The following rules were added during plan refinement after a gap audit against OWASP Top 10 and OWASP API Security Top 10:

- **BE-AUTHN constant-time comparison** — token/HMAC equality with `==` enables timing attacks; explicit rule prevents the bug class.
- **BE-AUTHZ mass assignment** — frameworks that bind request bodies to model fields without an allowlist let users set fields they shouldn't (admin flags, internal IDs).
- **BE-INPUT parameterized queries** — the canonical web vulnerability. Generic "validate at boundaries" doesn't enforce the right mitigation.
- **BE-INPUT SSRF** — OWASP A10:2021 / API6:2023. Server-side fetching of user-supplied URLs is a major attack vector for cloud metadata service exfiltration and internal network probing.
- **BE-INPUT deserialization** — Java/Pickle/JSON deserialization of untrusted input enables RCE on many platforms.
- **BE-INPUT path traversal** — file-handling rule about validating paths against a base directory before opening files.
- **BE-INPUT XXE** — when the project parses XML, default parser configurations frequently allow external entity expansion.
- **BE-INPUT SSTI** — Jinja, Twig, and similar template engines RCE when user input reaches template strings.
- **BE-API HSTS** — `Strict-Transport-Security` header + HTTPS-only redirect — the canonical "force HTTPS" rule.
- **BE-API Cache-Control for sensitive responses** — `no-store` for authenticated/sensitive endpoints prevents shared caches and back-button leakage.
- **BE-API redirect validation** — backend redirect endpoints (`/redirect?url=...`) must validate against an allowlist.

Trade-off: comprehensive coverage would require 80+ rules with careful threat modeling — still too much for v1. The starter set targets the most-likely-to-bite-you items per OWASP/CWE, with the format inviting subsequent additions for emerging or domain-specific concerns (WebSockets, Service Workers, supply-chain attestation, etc.).

### Validate uses each rule's `Verification` field as a mini-prompt

Validate is a markdown-reading agent prompt; it does not run static analysis. For each MUST/MUST NOT rule, the rule's **Verification** field tells validate *how* to check the rule against the project's specs/plans/`system.md`. Verification fields are written as instructions to an agent, not as code patterns:

> Verification: Any spec or plan that introduces credential storage MUST specify the hashing algorithm by name. Validate searches for credential/password/auth keywords and flags persistence paths that do not name a memory-hard hash.

This delegates the per-rule logic to the rule itself — adding a new rule does not require modifying validate. Validate's job is to:

1. Load both rule files (handling edge cases per spec).
2. For each MUST/MUST NOT rule, execute its Verification instruction against the project's specs/plans/system.md.
3. Emit findings with the rule ID.

The trade-off: validate's accuracy depends on the rule author's Verification phrasing. Vague Verifications produce vague findings. Rule format guidance in `Rule Format` and Verification examples in this plan set the standard.

### Edge case behaviors are encoded as a dedicated check section in validate

`framework/commands/analyze.md` gains a new check section, **Security rules**, slotted after **Cross-spec references (advisory)** and before **Markdown lint (advisory)**. The section codifies all 7 edge cases from the spec — block on malformed/unknown/duplicate; warn on missing files / deprecated references; silent on contextually-inert rules.

Each edge case maps directly to a checkbox in validate's check list, so violations show up grouped under the standard hard-fail/blocking/advisory headers in validate's report.

### Brownfield audit lives in govern, not validate

When `/govern` lands rule files in a project with existing `specs/NNN-*/` directories, it runs a one-time audit and writes findings to `specs/inbox.md`. The adopter then walks the inbox via `/{project}:groom`. This reuses 011's brownfield infrastructure rather than introducing baseline files or suppression mechanisms.

`framework/bootstrap/govern.md` gains a new top-level section, **Security audit (brownfield)**, slotted after **Shared Files** (where the manifest deposits the rule files) and before **Per-Agent Scaffolding**. The section's logic:

1. Detect the trigger: at least one of `specs/security-backend.md` or `specs/security-frontend.md` was newly created (not updated) by the manifest pass, AND at least one `specs/NNN-*` directory exists.
2. Load the newly created rule file(s) using the same integrity checks validate uses. If a file fails to load, report and skip the audit for that file.
3. Iterate the rules; for each rule whose Verification trigger fires against an existing project artifact, produce a finding.
4. Append findings to `specs/inbox.md`, deduplicating against existing lines that begin with the same `{Rule ID}: {artifact path}` prefix.
5. Report the audit summary line (`{N} security audit items added to specs/inbox.md.`) in the post-scaffolding output, omitted when N is zero.

Audit logic mirrors validate's per-rule check logic. Both call into the same Verification-evaluation pattern; the difference is the *output sink* (inbox vs. validate's findings report). For implementation, the validate prompt and the govern audit section can share a written description of the per-rule check (referenced rather than duplicated) — the rule-evaluation procedure lives in one place, both consumers reference it.

Trade-off: govern gains complexity from this audit step, but the alternative (adopters running validate post-adoption and manually piping output to log) is high-friction and fragile. Auto-audit on first install matches the brownfield ergonomic that 011 already established.

### Constitution gets a one-line append

`framework/constitution.md` "Secure" principle currently reads:

> **Secure:** protect sensitive data through industry standards and best practices

Becomes:

> **Secure:** protect sensitive data through industry standards and best practices. See `specs/security-backend.md` and `specs/security-frontend.md` for enforceable rules.

Minimal — does not duplicate rule content into the constitution; the rule files are the operational detail.

### Data model formalizes rule entry schema

Like 005's registry, the rule entry is structured data even though it lives in markdown. `data-model.md` documents the required fields, ID format, category enum, and Verification phrasing conventions. This is the contract validate relies on; future rule writers consult it to keep the rule files internally consistent.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/rules/security-backend.md` | Create | Starter set of backend security rules |
| `framework/rules/security-frontend.md` | Create | Starter set of frontend security rules |
| `framework/bootstrap/govern.md` | Modify | Add 2 rows to **Governance-owned shared files (strategy: update)** mapping `framework/rules/security-{backend,frontend}.md` → `specs/security-{backend,frontend}.md`. Add a new **Security audit (brownfield)** section after **Shared Files** and before **Per-Agent Scaffolding**, plus an audit-summary line in **Post-Scaffolding Output**. |
| `framework/commands/analyze.md` | Modify | Add **Security rules** check section codifying the 7 edge cases and the contextual matching rule |
| `framework/constitution.md` | Modify | Append the rule-files reference to the "Secure" principle |
| `specs/008-security-rules/data-model.md` | Create | Schema for rule entries (fields, ID format, category enum, Verification phrasing convention) |

## Trade-offs

### Starter set vs. comprehensive coverage

V1 ships ~45 rules across 15 categories. Less common attack surfaces (GraphQL-specific, gRPC-specific, mobile-app-specific) are not covered. Acceptable because the rule format is designed for trivial extension — adding a rule is one heading, three paragraphs, one new ID number.

### Verification-by-prompt vs. Verification-by-pattern

Validate's "addresses the rule" check is qualitative — it depends on the rule author writing a clear Verification field and the validate agent interpreting specs reasonably. A future enhancement could add structured Verification metadata (e.g., `keywords: [password, credential]`, `must_specify: hashing_algorithm`) but v1 keeps Verifications as natural-language prompts. Trade-off: easier to write, harder to verify mechanically.

### Block on malformed/duplicate/unknown vs. warn-and-skip

User-decided: blocking. Errs on the side of "data must be accurate" — a malformed rule file or stale reference is not a valid working state. Rationale matches the spec's edge-case decisions and is noted there.

### One-line constitution edit vs. expanded principle

Could expand the "Secure" principle into multiple paragraphs of guidance. Rejected — the constitution is the law, the rule files are the operational detail. Mixing levels would dilute both.

### Rule files written in markdown vs. structured format (YAML/JSON)

Per **Technical Decisions**, markdown wins. Heading-anchored rules are grep-friendly, render well in any viewer, and don't fight the rest of governance's markdown-first ethos. The trade-off is that programmatic tooling (if it ever appears) must parse markdown rather than load JSON — but governance has no programmatic tooling and is unlikely to add any.

## Open Questions Resolved

All four open questions resolved during clarification. See `spec.md` Resolved Questions section for severity levels (MUST/SHOULD only), rule ID granularity (per-rule, never-renumber), per-category opt-out (none — contextual application + whole-file pinning suffices), and runtime/infra rules (verified through documentation commitment, not runtime probing).
