---
title: "008-security-rules — tasks"
---

# 008 — Security Rules Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Write the backend security rules file

Create `framework/rules/security-backend.md` with the v1 starter set. ~35 rules across 8 categories, biased toward MUST/MUST NOT, each with statement, rationale, and verification per `data-model.md`. Coverage targets were selected by walking OWASP Top 10 (2021), OWASP API Security Top 10 (2023), and CWE Top 25.

- [x] Create `framework/rules/` directory
- [x] Create `framework/rules/security-backend.md` with file-level introduction
- [x] **Authentication** category — at least 4 rules: credential storage (memory-hard hash); session/token handling; MFA consideration; constant-time comparison of secrets
- [x] **Authorization** category — at least 4 rules: default-deny; explicit permission checks at every entry point; privilege escalation prevention; mass-assignment / over-posting prevention
- [x] **Input validation** category — at least 9 rules: boundary validation; allowlist over denylist; size limits; parameterized queries (SQL/NoSQL/command injection); SSRF prevention on outbound URL fetches; safe deserialization of untrusted data; path-traversal prevention on filesystem ops; XXE prevention on XML parsing; SSTI prevention on template rendering
- [x] **Data protection** category — at least 4 rules: encryption at rest commitment; secrets management; PII handling; key rotation
- [x] **API security** category — at least 7 rules: rate limiting; CORS policy; security headers (generic); HSTS / HTTPS-only enforcement; `Cache-Control: no-store` for sensitive responses; redirect endpoint validation; error responses do not leak internals
- [x] **Logging and audit** category — at least 2 rules: sensitive-data exclusion from logs; audit trail for access control changes
- [x] **Dependency management** category — at least 2 rules: vulnerability scanning; pinned versions
- [x] **Error handling** category — at least 2 rules: no stack traces in production responses; structured error codes
- [x] Every rule heading is a level-3 heading containing only the rule ID
- [x] Every rule has Statement (block quote), Rationale, and Verification fields
- [x] All IDs follow `BE-{CATEGORY}-{NNN}` format and start at `001` per category
- [x] At least one rule per category uses MUST/MUST NOT (errors); SHOULD usage is appropriate where noted
- [x] File passes `npx markdownlint-cli2`

**Done when:** every backend category has at least the targeted rule count, all rules conform to the data-model schema, and the file passes markdownlint.

## 2. Migrate and extend the frontend security rules file

`framework/rules/security-frontend.md` already exists from the framework reorganization (commit `3fc76b7`) but predates 008's clarification. It contains 25 rules across 7 categories whose content is sound but whose **format, three category abbreviations, and Verification phrasing** are out of sync with the post-clarify data-model. Apply the same migration treatment used for the backend file in task 1 — see `framework/rules/security-backend.md` and commit `35502af` as the precedent.

### Migration steps

- [x] Read the existing `framework/rules/security-frontend.md` first; do **not** overwrite without preserving content
- [x] Rename three category abbreviations (preserve suffix numbers — never renumber):
  - `FE-AUTH-*` → `FE-AUTHN-*` (3 rules; matches the data-model's authentication abbreviation, parallel to BE)
  - `FE-DEP-*` → `FE-DEPS-*` (3 rules; data-model uses DEPS, not DEP)
  - `FE-DATA-*` → `FE-PII-*` (3 rules; the "Sensitive data handling" category is abbreviated `PII` per the data-model category table)
- [x] Reformat every rule to match the data-model schema:
  - Level-3 heading contains **only** the rule ID (no title after an em-dash)
  - Statement is a single block-quoted sentence using RFC 2119 keywords (`MUST`, `MUST NOT`, `SHOULD`, `SHOULD NOT`)
  - `**Rationale:**`, `**Verification:**`, `**Source:**` are paragraphs (not bullet items)
- [x] **Rewrite every Verification field** as an instruction to the validate agent — concrete keywords to search, what artifacts to scan, what commitments must be present, what to flag. Existing Verifications are written for human reviewers ("Code review confirms…") and validate cannot act on them. See backend file's Verification fields for the right phrasing convention.
- [x] Audit every rule for accuracy against the constitution's Technology guiding principles (`framework/constitution.md` §guiding-principles). Backend revealed two real accuracy issues (over-prescriptive `MUST`, missing CSP/Referrer-Policy headers); frontend may have analogous issues — fix them and note each in the commit message.
- [x] Preserve `**Source:**` citations on every existing rule (the field is now in the data-model schema as optional). Add Source citations to any rule that lacks one (OWASP cheat sheet, RFC, MDN, etc.).

### Coverage additions

- [x] **Cross-site scripting (XSS)** — existing 6 rules likely cover output encoding, framework auto-escaping, safe DOM methods, no inline scripts, HTML sanitization, URL validation. Confirm coverage; add rules only if a v1-target gap exists.
- [x] **Cross-site request forgery (CSRF)** — existing 3 rules likely cover token-based protection, SameSite cookies, and no state changes via GET. Confirm coverage.
- [x] **Secure storage** — existing 3 rules cover no secrets in browser storage, cookie security attributes, no sensitive data in URLs. Confirm coverage.
- [x] **Authentication UX (`FE-AUTHN` after rename)** — existing 3 rules cover redirect validation, session expiration UX, logout completeness. Plan called for token storage / session expiration / redirect validation — confirm coverage; the existing rule set is broader than the plan's targets.
- [x] **Content security (`FE-CSP`)** — existing 4 rules cover CSP header required, strict CSP policy, frame protection, form action restriction. Plan called for CSP + SRI; SRI is currently under `FE-DEP-002`. Decide: keep SRI under DEPS (logical grouping with other dependency-integrity concerns) or move to CSP. Document the decision. **Decision: kept SRI under `FE-DEPS-002` — moving categories would force renumbering, violating the never-renumber discipline; SRI is also conceptually closer to dependency integrity than to CSP directives.**
- [x] **Dependency management (`FE-DEPS` after rename)** — existing 3 rules cover vulnerability scanning, subresource integrity, no dynamic third-party loading. **Add a pinned-versions rule** (`FE-DEPS-004` or next available) matching `BE-DEPS-002`'s posture for npm/pnpm/yarn lockfiles. This is the only confirmed coverage gap.
- [x] **Sensitive data handling (`FE-PII` after rename)** — existing 3 rules cover PII masking, autocomplete control, cache control for sensitive pages. Confirm coverage of the plan's targets (UI masking, URL parameters, browser history).

### Final checks

- [x] No stale IDs remain (`grep -E '^### FE-(AUTH-|DEP-|DATA-)' framework/rules/security-frontend.md` returns nothing)
- [x] Every rule heading is a level-3 heading containing only the rule ID — no em-dash-and-title suffix
- [x] Every rule has Statement (block quote), Rationale, Verification, and Source fields
- [x] All IDs follow `FE-{CATEGORY}-{NNN}` format with categories from `{XSS, CSRF, STORAGE, AUTHN, CSP, DEPS, PII}`
- [x] At least one rule per category uses `MUST`/`MUST NOT`
- [x] File passes `npx markdownlint-cli2`

### Why migrate, not rewrite

The existing rules' *content* (rationale, threat model, OWASP citations) represents real prior thought. Throwing it away to write a fresh ~21-rule starter set loses information and mis-applies the never-renumber discipline. Migration preserves content, fixes format, and lets v1 ship with broader coverage (~26+ rules) than the original plan target.

**Done when:** every frontend rule conforms to the data-model schema, no stale category abbreviations remain, every Verification is written as an agent-actionable instruction, the pinned-versions rule has been added, accuracy issues found during the audit are fixed, and the file passes markdownlint.

## 3. Update govern manifest

Add the two rule files to `framework/bootstrap/govern.md`'s **Governance-owned shared files (strategy: update)** table.

- [x] Insert row mapping `framework/rules/security-backend.md` → `specs/security-backend.md`
- [x] Insert row mapping `framework/rules/security-frontend.md` → `specs/security-frontend.md`
- [x] Rows appear adjacent to the other governance-owned shared files
- [x] `framework/bootstrap/govern.md` passes `npx markdownlint-cli2`

**Done when:** govern syncs both rule files on its next run, and the file passes markdownlint.

## 4. Add the brownfield security audit to govern

Add a new top-level **Security audit (brownfield)** section to `framework/bootstrap/govern.md`, slotted after **Shared Files** and before **Per-Agent Scaffolding**. The section reads each rule file that was newly created by the manifest pass, evaluates each rule's Verification trigger against existing `specs/NNN-*` artifacts, and appends findings to `specs/inbox.md`.

- [x] Insert **Security audit (brownfield)** section between **Shared Files** and **Per-Agent Scaffolding**
- [x] Section's trigger: at least one of `specs/security-backend.md` or `specs/security-frontend.md` was newly **created** by the manifest pass AND the project contains at least one `specs/NNN-*` directory
- [x] Section silently skips when the trigger does not fire (greenfield run, or routine re-run with rule files already present)
- [x] Section loads each newly created rule file using the same integrity checks validate uses (well-formed headings, required fields, valid IDs, no duplicates); on load failure, reports the failure and skips the audit for that file
- [x] Section iterates loaded rules; for each rule whose Verification trigger fires against an existing project artifact (`spec.md`, `spec-and-plan.md`, `plan.md`, scenario files under `specs/NNN-*/`), produces a finding
- [x] Section appends each finding to `specs/inbox.md` as `- [ ] {Rule ID}: {affected artifact path} does not address — {one-line summary}`
- [x] Section deduplicates against existing inbox content: skip any finding whose `- [ ] {Rule ID}: {affected artifact path}` prefix already appears in the inbox
- [x] Add an audit-summary line to **Post-Scaffolding Output**: `{N} security audit items added to specs/inbox.md. Run /{project}:groom to triage.` Omit when N is zero
- [x] `framework/bootstrap/govern.md` passes `npx markdownlint-cli2`

**Done when:** govern runs the audit only when the trigger conditions hold, writes deduplicated findings to inbox in the documented format, reports the count, and the file passes markdownlint.

## 5. Extend validate with the security rule check section

Modify `framework/commands/validate.md` to add a new **Security rules** check section that codifies the spec's edge-case behaviors. Place it after **Cross-spec references (advisory)** and before **Markdown lint (advisory)**.

- [x] Add **Security rules (blocking and advisory)** section
- [x] Section opens by reading `specs/security-backend.md` and `specs/security-frontend.md` (each independently optional — only the present files are loaded)
- [x] Section validates rule file integrity per `data-model.md` (well-formed headings, required fields, ID format) and reports a **blocking** error per malformed rule file (`Malformed security rule file {path} at {location}: {reason}`)
- [x] Section reports a **blocking** error on duplicate IDs within a file (`Duplicate rule ID {ID} in {file}; refusing to load`)
- [x] Section reports an **advisory** warning when neither file is present (`No security rule files found, skipping security checks`)
- [x] When at least one rule file is loaded and well-formed, section iterates over each rule:
  - For each MUST/MUST NOT rule whose Verification trigger fires against any project artifact (spec, plan, `specs/system.md`), execute the Verification check and emit a **blocking** error per failing artifact, including the rule ID
  - For each SHOULD/SHOULD NOT rule whose trigger fires, emit an **advisory** warning per failing artifact, including the rule ID
  - Rules whose Verification trigger does not fire for any artifact produce no finding (silently inert)
- [x] Section reports a **blocking** error when a project artifact references a rule ID that does not exist in the loaded files (`Spec at {path} references unknown rule {ID}`)
- [x] Section reports an **advisory** warning when a project artifact references a `DEPRECATED` rule ID (`Spec at {path} references deprecated rule {ID}; targeted for removal in {version}`)
- [x] Findings are surfaced under the existing severity sections (Hard fail / Blocking / Advisory) in validate's report
- [x] `framework/commands/validate.md` passes `npx markdownlint-cli2`

**Done when:** validate's check list includes the security rule section with all 7 edge-case behaviors, and the file passes markdownlint.

## 6. Update the constitution

Append the rule-files reference to the "Secure" guiding principle in `framework/constitution.md`.

- [x] Edit the "Secure" bullet under **Guiding Principles → Technology** to read: `**Secure:** protect sensitive data through industry standards and best practices. See \`specs/security-backend.md\` and \`specs/security-frontend.md\` for enforceable rules.`
- [x] No other constitution edits in this task
- [x] `framework/constitution.md` passes `npx markdownlint-cli2`

**Done when:** the "Secure" principle references the rule files by their project paths, and the file passes markdownlint.

## 7. Validate end-to-end and run readiness checks

Run all structural and lint checks; verify each acceptance criterion is satisfied by the produced artifacts.

- [x] `npx markdownlint-cli2` passes on all created/modified files: `framework/rules/security-backend.md`, `framework/rules/security-frontend.md`, `framework/bootstrap/govern.md`, `framework/commands/validate.md`, `framework/constitution.md`, and the feature directory (`spec.md`, `plan.md`, `tasks.md`, `data-model.md`, `code-locations.md`)
- [x] Every rule ID in both files matches the `{surface}-{category}-{NNN}` format
- [x] No duplicate rule IDs within either file (check via `grep '^### ' framework/rules/security-*.md | sort | uniq -d`)
- [x] Every rule has Statement, Rationale, and Verification fields present
- [x] Each acceptance criterion in `spec.md` is checked individually against the produced artifacts and marked `- [x]` only if satisfied

**Done when:** all checks pass, the rule files are internally consistent, and every acceptance criterion is satisfied.
