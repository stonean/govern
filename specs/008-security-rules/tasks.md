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

## 2. Write the frontend security rules file

Create `framework/rules/security-frontend.md` with the v1 starter set. ~21 rules across 7 categories, same format as backend.

- [ ] Create `framework/rules/security-frontend.md` with file-level introduction
- [ ] **Cross-site scripting (XSS)** category — at least 3 rules covering output encoding, CSP, and inline scripts
- [ ] **Cross-site request forgery (CSRF)** category — at least 2 rules covering tokens and SameSite cookies
- [ ] **Secure storage** category — at least 3 rules covering localStorage/sessionStorage and cookie attributes
- [ ] **Authentication UX** category — at least 3 rules covering token handling, session expiration, and redirects
- [ ] **Content security** category — at least 2 rules covering CSP and SRI
- [ ] **Dependency management** category — at least 2 rules covering vulnerability scanning and pinned versions
- [ ] **Sensitive data handling** category — at least 3 rules covering UI masking, URL parameters, and browser history
- [ ] Every rule heading is a level-3 heading containing only the rule ID
- [ ] Every rule has Statement (block quote), Rationale, and Verification fields
- [ ] All IDs follow `FE-{CATEGORY}-{NNN}` format and start at `001` per category
- [ ] At least one rule per category uses MUST/MUST NOT (errors)
- [ ] File passes `npx markdownlint-cli2`

**Done when:** every frontend category has at least the targeted rule count, all rules conform to the data-model schema, and the file passes markdownlint.

## 3. Update govern manifest

Add the two rule files to `framework/bootstrap/govern.md`'s **Governance-owned shared files (strategy: update)** table.

- [ ] Insert row mapping `framework/rules/security-backend.md` → `specs/security-backend.md`
- [ ] Insert row mapping `framework/rules/security-frontend.md` → `specs/security-frontend.md`
- [ ] Rows appear adjacent to the other governance-owned shared files
- [ ] `framework/bootstrap/govern.md` passes `npx markdownlint-cli2`

**Done when:** govern syncs both rule files on its next run, and the file passes markdownlint.

## 4. Add the brownfield security audit to govern

Add a new top-level **Security audit (brownfield)** section to `framework/bootstrap/govern.md`, slotted after **Shared Files** and before **Per-Agent Scaffolding**. The section reads each rule file that was newly created by the manifest pass, evaluates each rule's Verification trigger against existing `specs/NNN-*` artifacts, and appends findings to `specs/inbox.md`.

- [ ] Insert **Security audit (brownfield)** section between **Shared Files** and **Per-Agent Scaffolding**
- [ ] Section's trigger: at least one of `specs/security-backend.md` or `specs/security-frontend.md` was newly **created** by the manifest pass AND the project contains at least one `specs/NNN-*` directory
- [ ] Section silently skips when the trigger does not fire (greenfield run, or routine re-run with rule files already present)
- [ ] Section loads each newly created rule file using the same integrity checks validate uses (well-formed headings, required fields, valid IDs, no duplicates); on load failure, reports the failure and skips the audit for that file
- [ ] Section iterates loaded rules; for each rule whose Verification trigger fires against an existing project artifact (`spec.md`, `spec-and-plan.md`, `plan.md`, scenario files under `specs/NNN-*/`), produces a finding
- [ ] Section appends each finding to `specs/inbox.md` as `- [ ] {Rule ID}: {affected artifact path} does not address — {one-line summary}`
- [ ] Section deduplicates against existing inbox content: skip any finding whose `- [ ] {Rule ID}: {affected artifact path}` prefix already appears in the inbox
- [ ] Add an audit-summary line to **Post-Scaffolding Output**: `{N} security audit items added to specs/inbox.md. Run /{project}:groom to triage.` Omit when N is zero
- [ ] `framework/bootstrap/govern.md` passes `npx markdownlint-cli2`

**Done when:** govern runs the audit only when the trigger conditions hold, writes deduplicated findings to inbox in the documented format, reports the count, and the file passes markdownlint.

## 5. Extend validate with the security rule check section

Modify `framework/commands/validate.md` to add a new **Security rules** check section that codifies the spec's edge-case behaviors. Place it after **Cross-spec references (advisory)** and before **Markdown lint (advisory)**.

- [ ] Add **Security rules (blocking and advisory)** section
- [ ] Section opens by reading `specs/security-backend.md` and `specs/security-frontend.md` (each independently optional — only the present files are loaded)
- [ ] Section validates rule file integrity per `data-model.md` (well-formed headings, required fields, ID format) and reports a **blocking** error per malformed rule file (`Malformed security rule file {path} at {location}: {reason}`)
- [ ] Section reports a **blocking** error on duplicate IDs within a file (`Duplicate rule ID {ID} in {file}; refusing to load`)
- [ ] Section reports an **advisory** warning when neither file is present (`No security rule files found, skipping security checks`)
- [ ] When at least one rule file is loaded and well-formed, section iterates over each rule:
  - For each MUST/MUST NOT rule whose Verification trigger fires against any project artifact (spec, plan, `specs/system.md`), execute the Verification check and emit a **blocking** error per failing artifact, including the rule ID
  - For each SHOULD/SHOULD NOT rule whose trigger fires, emit an **advisory** warning per failing artifact, including the rule ID
  - Rules whose Verification trigger does not fire for any artifact produce no finding (silently inert)
- [ ] Section reports a **blocking** error when a project artifact references a rule ID that does not exist in the loaded files (`Spec at {path} references unknown rule {ID}`)
- [ ] Section reports an **advisory** warning when a project artifact references a `DEPRECATED` rule ID (`Spec at {path} references deprecated rule {ID}; targeted for removal in {version}`)
- [ ] Findings are surfaced under the existing severity sections (Hard fail / Blocking / Advisory) in validate's report
- [ ] `framework/commands/validate.md` passes `npx markdownlint-cli2`

**Done when:** validate's check list includes the security rule section with all 7 edge-case behaviors, and the file passes markdownlint.

## 6. Update the constitution

Append the rule-files reference to the "Secure" guiding principle in `framework/constitution.md`.

- [ ] Edit the "Secure" bullet under **Guiding Principles → Technology** to read: `**Secure:** protect sensitive data through industry standards and best practices. See \`specs/security-backend.md\` and \`specs/security-frontend.md\` for enforceable rules.`
- [ ] No other constitution edits in this task
- [ ] `framework/constitution.md` passes `npx markdownlint-cli2`

**Done when:** the "Secure" principle references the rule files by their project paths, and the file passes markdownlint.

## 7. Validate end-to-end and run readiness checks

Run all structural and lint checks; verify each acceptance criterion is satisfied by the produced artifacts.

- [ ] `npx markdownlint-cli2` passes on all created/modified files: `framework/rules/security-backend.md`, `framework/rules/security-frontend.md`, `framework/bootstrap/govern.md`, `framework/commands/validate.md`, `framework/constitution.md`, and the feature directory (`spec.md`, `plan.md`, `tasks.md`, `data-model.md`, `code-locations.md`)
- [ ] Every rule ID in both files matches the `{surface}-{category}-{NNN}` format
- [ ] No duplicate rule IDs within either file (check via `grep '^### ' framework/rules/security-*.md | sort | uniq -d`)
- [ ] Every rule has Statement, Rationale, and Verification fields present
- [ ] Each acceptance criterion in `spec.md` is checked individually against the produced artifacts and marked `- [x]` only if satisfied

**Done when:** all checks pass, the rule files are internally consistent, and every acceptance criterion is satisfied.
