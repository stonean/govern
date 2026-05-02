# 008 — Security Rules Code Locations

## AC: `framework/rules/security-backend.md` exists in the governance framework with categorized, numbered rules

- `framework/rules/security-backend.md`

## AC: `framework/rules/security-frontend.md` exists in the governance framework with categorized, numbered rules

- `framework/rules/security-frontend.md`

## AC: Every rule has an ID, statement, rationale, and verification method

- `framework/rules/security-backend.md`
- `framework/rules/security-frontend.md`

## AC: Rule IDs follow the format `{surface}-{category}-{NNN}` with `{surface}` ∈ `{BE, FE}` and `{NNN}` zero-padded starting at `001`

- `framework/rules/security-backend.md`
- `framework/rules/security-frontend.md`

## AC: Rules use RFC 2119 language to distinguish enforced (MUST/MUST NOT) from advisory (SHOULD/SHOULD NOT)

- `framework/rules/security-backend.md`
- `framework/rules/security-frontend.md`

## AC: Both files appear in the govern file manifest with `update` strategy

- `framework/bootstrap/govern.md`

## AC: The govern command fetches `framework/rules/security-backend.md` and writes it to `specs/security-backend.md` in the project

- `framework/bootstrap/govern.md`

## AC: The govern command fetches `framework/rules/security-frontend.md` and writes it to `specs/security-frontend.md` in the project

- `framework/bootstrap/govern.md`

## AC: Re-running govern updates both files to the latest governance version

- `framework/bootstrap/govern.md`

## AC: Projects can pin either file in `.governance.toml` to skip updates

- `framework/bootstrap/govern.md`

## AC: The validate command reads `specs/security-backend.md` and `specs/security-frontend.md` when present in the project

- `framework/commands/validate.md`

## AC: MUST/MUST NOT violations are reported as errors (blocking)

- `framework/commands/validate.md`

## AC: SHOULD/SHOULD NOT violations are reported as warnings (non-blocking)

- `framework/commands/validate.md`

## AC: Rule IDs appear in validate output for each finding

- `framework/commands/validate.md`

## AC: Rules apply contextually — a rule that no spec or plan content exercises produces no finding

- `framework/commands/validate.md`

## AC: Validate emits a warning and continues when no security rule files are present

- `framework/commands/validate.md`

## AC: Validate runs only over the present file when one of the two is pinned out, with no finding for the missing file

- `framework/commands/validate.md`

## AC: Validate blocks with an error on a malformed rule file (missing required field, ID format violation, parse failure)

- `framework/commands/validate.md`

## AC: Validate blocks with an error on a spec/plan reference to an unknown rule ID

- `framework/commands/validate.md`

## AC: Validate emits a warning (not an error) on a spec/plan reference to a `DEPRECATED` rule ID

- `framework/commands/validate.md`

## AC: Validate blocks with an error when a rule file contains duplicate IDs

- `framework/commands/validate.md`

## AC: On a govern run where a security rule file is newly created AND `specs/NNN-*` directories exist, govern audits the existing specs against the rule and writes one inbox item per finding to `specs/inbox.md`

- `framework/bootstrap/govern.md`

## AC: On a greenfield run (no existing `specs/NNN-*` directories), the audit is silently skipped

- `framework/bootstrap/govern.md`

## AC: On a routine re-run (rule files already present), the audit is silently skipped

- `framework/bootstrap/govern.md`

## AC: Inbox items follow the format `- [ ] {Rule ID}: {affected artifact path} does not address — {one-line summary}`

- `framework/bootstrap/govern.md`

## AC: Audit findings are deduplicated against existing inbox content (no duplicate items emitted on re-trigger)

- `framework/bootstrap/govern.md`

## AC: Govern's post-scaffolding output reports the count of new audit items added (omitted when zero)

- `framework/bootstrap/govern.md`

## AC: The "Secure" principle references `specs/security-backend.md` and `specs/security-frontend.md`

- `framework/constitution.md`
