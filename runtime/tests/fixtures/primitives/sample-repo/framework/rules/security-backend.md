# Security Rules — Backend (fixture)

Minimal rule file used by the `check-rule-ids` primitive. Rule IDs use
the `BE-{CATEGORY}-{NNN}` format.

## BE-AUTHN — Authentication

### BE-AUTHN-001

> Passwords MUST be hashed.

**Verification:** Specs that introduce credential storage name the
hashing algorithm.

### BE-AUTHN-002

> Tokens MUST be stored hashed.

**Verification:** Specs that introduce tokens describe hashed storage.
