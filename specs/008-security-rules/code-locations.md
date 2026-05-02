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
