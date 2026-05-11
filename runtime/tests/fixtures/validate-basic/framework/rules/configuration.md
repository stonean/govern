# Configuration Rules

Fixture rule file used by the validate-basic parity test.

### CFG-CONST-001

> **Statement:** Compile-time configuration constants live in a single
> central module so callers cannot drift apart on the value.

**Rationale:** Centralizing constants makes drift impossible by
construction; reviewers can see every value change in one diff.

**Verification:** Every numeric or string literal that varies across
environments (timeouts, retry counts, default ports) is sourced from
the central module.
