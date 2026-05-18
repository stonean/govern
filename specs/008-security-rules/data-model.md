---
title: "008-security-rules — data-model"
---

# 008 — Security Rules Data Model

## Rule file structure

A rule file is a markdown document with the following structure:

```markdown
# {Surface} Security Rules

{One-paragraph introduction stating the file's scope.}

## {Category Name}

### {Rule ID}

> {Rule statement using RFC 2119 keywords.}

**Rationale:** {Threat the rule mitigates.}

**Verification:** {Instruction to the validate agent on how to check the rule.}

### {Rule ID}

…
```

- **Surface** is `Backend` or `Frontend`.
- **Category Name** matches the spec's category list verbatim (`Authentication`, `Authorization`, `Input validation`, `Data protection`, `API security`, `Logging and audit`, `Dependency management`, `Error handling`, `Cross-site scripting (XSS)`, `Cross-site request forgery (CSRF)`, `Secure storage`, `Authentication UX`, `Content security`, `Sensitive data handling`).
- **Rule ID** appears as a level-3 heading and is the only level-3 heading content (no surrounding text). This makes rules grep-able by ID.

## Rule ID format

```text
{surface}-{category}-{NNN}
```

| Element | Type | Constraints |
| --- | --- | --- |
| `surface` | string | `BE` (backend) or `FE` (frontend). Uppercase. |
| `category` | string | Short uppercase alphanumeric abbreviation (`[A-Z][A-Z0-9]*`) drawn from the per-surface set below or extended by a later rule-introducing spec. |
| `NNN` | integer | Zero-padded sequence number, starting at `001`. Never renumbered. Never reused after removal. |

### Category abbreviations

| Surface | Category | Abbreviation |
| --- | --- | --- |
| BE | Authentication | `AUTHN` |
| BE | Authorization | `AUTHZ` |
| BE | Input validation | `INPUT` |
| BE | Data protection | `DATA` |
| BE | API security | `API` |
| BE | Logging and audit | `LOG` |
| BE | Dependency management | `DEPS` |
| BE | Error handling | `ERR` |
| FE | Cross-site scripting (XSS) | `XSS` |
| FE | Cross-site request forgery (CSRF) | `CSRF` |
| FE | Secure storage | `STORAGE` |
| FE | Authentication UX | `AUTHN` |
| FE | Content security | `CSP` |
| FE | Dependency management | `DEPS` |
| FE | Sensitive data handling | `PII` |

`AUTHN` (authentication) and `AUTHZ` (authorization) are deliberately distinct abbreviations to prevent collision; both surfaces use `AUTHN` for their authentication-related rules. `AUTHN` and `DEPS` are shared across surfaces (`BE-AUTHN` and `FE-AUTHN` are different namespaces). The full ID always includes the surface prefix to disambiguate.

## Rule entry fields

| Field | Required | Format | Notes |
| --- | --- | --- | --- |
| Rule ID | yes | Level-3 heading (`### {ID}`) | Matches the format above. The heading contains nothing but the ID. |
| Statement | yes | Block quote (`> …`) | One sentence using RFC 2119 keywords (MUST, MUST NOT, SHOULD, SHOULD NOT). |
| Rationale | yes | Paragraph beginning `**Rationale:**` | Brief explanation of the threat or risk the rule mitigates. |
| Verification | yes | Paragraph beginning `**Verification:**` | Instruction to the validate agent — see **Verification phrasing** below. |
| Source | no | Paragraph beginning `**Source:**` | Citation to authoritative origin (e.g., OWASP cheat sheet name, RFC number, NIST publication, CIS Benchmark). Optional but recommended — aids `Learnable` (readers can trace the rule's grounding) and `Verified` (reviewers can audit the citation). |
| Deprecated | no | Paragraph beginning `**DEPRECATED in {version}:**` | Present only on deprecated rules. Includes the removal target version. The rule remains in the file with this label until removed. |

## Verification phrasing

The Verification field is read by the validate agent during validation runs. It must:

1. Identify the project artifacts in scope (typically: feature specs, plans, `specs/system.md`).
2. Describe the trigger that makes the rule applicable to a given artifact (e.g., "any spec that introduces credential storage", "any plan that handles file uploads"). A rule whose trigger does not fire for any artifact is silently inert (no finding emitted).
3. State what the artifact MUST or SHOULD include when the trigger fires (e.g., "specify the hashing algorithm by name", "describe the rate-limit threshold").
4. Distinguish documentation commitments from code patterns when the rule's enforcement happens outside the repository (runtime config, infrastructure, deployment).

### Examples

**Code-pattern Verification:**

```text
Verification: Any spec or plan that introduces credential storage MUST
specify the hashing algorithm by name. Validate searches feature specs
and plans for credential/password/auth keywords; for each match, flags
the artifact if it does not name a memory-hard hash (Argon2id, scrypt,
or bcrypt).
```

**Documentation-commitment Verification (runtime/infra):**

```text
Verification: specs/system.md or a deployment-related spec MUST describe
how TLS is terminated (load balancer, application, or sidecar) and the
expected protocol/cipher policy. Validate flags the absence of any TLS
handling commitment in the project's specs.
```

## Severity classification

The Statement's RFC 2119 keyword determines the validate severity:

| Keyword | Severity | Reporting |
| --- | --- | --- |
| MUST, MUST NOT | Error | Blocking |
| SHOULD, SHOULD NOT | Warning | Non-blocking |

Rules MUST use exactly one of the four keywords in the Statement. Mixed keywords (e.g., "MUST … SHOULD") are not permitted; split such rules into two entries.

## ID stability invariants

These invariants are enforced by validate (as edge cases in `spec.md`) and are also a discipline for rule authors:

- Once an ID is assigned, the rule retains that ID for life. Editing the Statement or moving the rule within the file does not change its ID.
- Deprecated rules retain their ID. They are removed only after the deprecation window has passed and references in adopting projects have been updated.
- Sequence numbers are never reused after a rule is fully removed. New rules in a category get the next unused number.
- Two rules in the same file MUST NOT share an ID. Validate refuses to load a file with duplicate IDs.

## Rule file integrity

A rule file is considered well-formed if every rule heading, statement, rationale, and verification field is present and the ID format is satisfied. Missing fields, malformed IDs, or unparseable content cause validate to refuse to load the file (per the spec's edge-case decisions). The file is then treated as absent for matching purposes, but the parse failure itself is a hard error.
