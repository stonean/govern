# 017 — Derive, Don't Ask Data Model

Defines the structures introduced or modified by this spec: the configuration rule file format, the updated spec/scenario frontmatter schema, and the marker-comment convention for generated content blocks.

## Configuration rule file structure

A configuration rule file is a markdown document with the following structure (mirrors the security-rule file precedent declared in `specs/008-security-rules/data-model.md`):

```markdown
# Configuration Rules

{One-paragraph introduction stating the file's scope.}

## {Category Name}

### {Rule ID}

> {Rule statement using RFC 2119 keywords.}

**Rationale:** {Threat the rule mitigates.}

**Verification:** {Instruction to the validate agent on how to check the rule.}

**Source:** {Optional citation to authoritative origin.}

### {Rule ID}

…
```

- **Category Name** is `Constants` or `Environment variables`.
- **Rule ID** appears as a level-3 heading and is the only level-3 heading content (no surrounding text). This makes rules grep-able by ID.

## Configuration rule ID format

```text
CFG-{category}-{NNN}
```

| Element | Type | Constraints |
| --- | --- | --- |
| `CFG` | literal | Always uppercase. The fixed prefix for configuration rules. |
| `category` | string | Short uppercase abbreviation drawn from the set below. |
| `NNN` | integer | Zero-padded sequence number, starting at `001`. Never renumbered. Never reused after removal. |

### Category abbreviations

| Category | Abbreviation |
| --- | --- |
| Constants | `CONST` |
| Environment variables | `ENV` |

The full ID always includes the `CFG-` prefix to disambiguate from `BE-` (backend security) and `FE-` (frontend security) rule IDs.

## Configuration rule entry fields

| Field | Required | Format | Notes |
| --- | --- | --- | --- |
| Rule ID | yes | Level-3 heading (`### {ID}`) | Matches the format above. The heading contains nothing but the ID. |
| Statement | yes | Block quote (`> …`) | One sentence using RFC 2119 keywords (MUST, MUST NOT, SHOULD, SHOULD NOT). |
| Rationale | yes | Paragraph beginning `**Rationale:**` | Brief explanation of the threat or risk the rule mitigates. |
| Verification | yes | Paragraph beginning `**Verification:**` | Instruction to the validate agent — see **Verification phrasing** below. |
| Source | no | Paragraph beginning `**Source:**` | Citation to authoritative origin (e.g., 12-Factor App, NIST SP 800, IEC 60027 for unit suffixes). Optional but recommended. |
| Deprecated | no | Paragraph beginning `**DEPRECATED in {version}:**` | Present only on deprecated rules. Includes the removal target version. The rule remains in the file with this label until removed. |

## Verification phrasing

Same rules as the security-rule file (see `specs/008-security-rules/data-model.md` § Verification phrasing). The Verification field must:

1. Identify the project artifacts in scope (typically: feature plans, source code via spec-stated affected files, `specs/system.md` configuration sections).
2. Describe the trigger that makes the rule applicable to a given artifact.
3. State what the artifact MUST or SHOULD include when the trigger fires.
4. Distinguish documentation commitments from code patterns when the rule's enforcement happens outside the repository.

## Severity classification

Same as security rules:

| Keyword | Severity | Reporting |
| --- | --- | --- |
| MUST, MUST NOT | Error | Blocking |
| SHOULD, SHOULD NOT | Warning | Non-blocking |

Rules MUST use exactly one of the four keywords in the Statement. Mixed keywords are not permitted; split such rules into two entries.

## ID stability invariants

Same as security rules. Once an ID is assigned, the rule retains that ID for life. Editing the Statement or moving the rule within the file does not change its ID. Deprecated rules retain their ID. Sequence numbers are never reused after a rule is fully removed. Two rules in the same file MUST NOT share an ID.

## Initial rule set

The first version of `framework/rules/configuration-cross.md` ships with the following rules. Numbering is assigned at creation and is permanent.

### CFG-CONST namespace (Constants category)

- `CFG-CONST-001` — Shared constants live in a centralized location (e.g., `shared/constants/`)
- `CFG-CONST-002` — Module-local constants live in the module's own constants file
- `CFG-CONST-003` — Configurable values are not bare literals — every operator-tunable value (timeout, retry count, batch size, threshold, rate limit) is a named constant or env-var-backed value, not a repeated literal

### CFG-ENV namespace (Environment variables category)

- `CFG-ENV-001` — Every env var has a default constant defined in code; the env var is read once at startup and falls back to the constant when unset
- `CFG-ENV-002` — `.env.example` contains every introduced var with a descriptive comment and safe placeholder value
- `CFG-ENV-003` — Required env vars are validated at startup with fail-fast naming of any var that cannot be resolved
- `CFG-ENV-004` — Time-value env vars include a unit suffix in the variable name (`_MS`, `_SECONDS`, `_MINUTES`); the corresponding constant makes the unit explicit (e.g., `DEFAULT_SHUTDOWN_TIMEOUT_SECONDS = 30`)

## Frontmatter schema (after this spec)

Updates the schema declared in `framework/constitution.md` §text-first-artifacts.

### Spec files (`spec.md`, `spec-and-plan.md`)

| Field | Required | Type | Allowed values | Notes |
| --- | --- | --- | --- | --- |
| `status` | yes | string | `draft`, `clarified`, `planned`, `in-progress`, `done` | Spec lifecycle state |
| `dependencies` | yes | list of strings | spec slugs (e.g., `002-events`); empty list permitted | **Generated** by `gen-spec-deps.sh` from body inline links; not hand-authored |

Removed: `tags` (deleted from schema and from validate). The `track` field on `spec-and-plan.md` is also removed; track is inferred from the filename.

### Scenario files (`scenarios/{slug}.md`)

| Field | Required | Type | Notes |
| --- | --- | --- | --- |
| `section` | yes | string | The parent spec section the scenario elaborates. Parent feature is implicit in the file path. |

Removed: `spec-ref` (replaced by `section`). `tags` removed.

### Other artifact files

`plan.md`, `tasks.md`, `data-model.md`, and `research.md` have no required frontmatter fields. The `title` field is removed from all template defaults.

### Open-schema rule (unchanged)

Additional fields beyond those listed above are permitted and ignored by uninterested consumers. Stale fields in done specs (`title`, `tags`, `spec-ref`, `track`) remain valid under this rule and produce no validate findings after this spec.

## Marker-comment convention for generated content blocks

Generated content blocks inside otherwise hand-authored files are bounded by HTML marker comments:

```markdown
<!-- generated:{name}:start -->
{generated content}
<!-- generated:{name}:end -->
```

Where `{name}` identifies the block type. The generator scripts find the markers, splice the regenerated content between them, and leave everything outside untouched. If the markers are absent, the generator emits a warning and exits non-zero — it never appends or guesses.

### Initial marker names

| Marker name | Where it lives | Generated by |
| --- | --- | --- |
| `feature-specs` | `README.md` | `scripts/gen-readme-table.sh` |
| `commands-pipeline` | `framework/commands/help.md` | `scripts/gen-help-tables.sh` |
| `commands-elaborate` | `framework/commands/help.md` | `scripts/gen-help-tables.sh` |
| `commands-brownfield` | `framework/commands/help.md` | `scripts/gen-help-tables.sh` |
| `commands-orient` | `framework/commands/help.md` | `scripts/gen-help-tables.sh` |
| `commands-bootstrap` | `framework/commands/help.md` | `scripts/gen-help-tables.sh` |

## Hook sentinel comment

The shipped adopter pre-commit hook contains a single sentinel line near the top:

```bash
# managed-by: govern
```

`/govern`'s hook-installation logic uses the presence of this sentinel to distinguish a govern-installed hook from a hand-rolled one. When the sentinel is present, `/govern` treats the file as `update`-strategy and overwrites it on subsequent runs (subject to `.govern.toml` pinning). When the sentinel is absent, `/govern` skips installation and warns the user.

## File extension and shebang conventions

All generator scripts and hook scripts:

- Use `.sh` extension
- Begin with `#!/usr/bin/env bash`
- Follow `set -euo pipefail` for fail-fast behavior
- Are executable (`chmod +x`)
