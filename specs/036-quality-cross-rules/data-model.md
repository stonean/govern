# 036 — Cross-cutting code-quality rules Data Model

Registers the `QUAL` rule-ID surface and its inaugural `STUB` category. It does not restate the rule-file schema — that is defined once in `specs/008-security-rules/data-model.md` (Statement / Rationale / Verification; `### {ID}` headings; permanent IDs; MUST/SHOULD severity) and `quality-cross.md` follows it, mirroring how `configuration-cross.md` (017) follows the same schema.

## `QUAL` surface

`QUAL` is a new rule-ID surface — the third segment of the registered-surface set alongside `BE`/`FE` (`008-security-rules/data-model.md`) and `CFG` (`017-derive-dont-ask/data-model.md`).

| Property | Value |
| --- | --- |
| Surface prefix | `QUAL` (always uppercase) |
| Scope | Cross-cutting code-quality discipline — applies to every stack |
| File | `framework/rules/quality-cross.md` (the `-cross.md` suffix loads it for every stack via 024's rule-loader; 033's surface filter keeps cross files unconditionally) |
| ID grammar | `QUAL-{CATEGORY}-{NNN}` — matches the runtime `check-rule-ids` harvester grammar (`[A-Z]{2,5}-[A-Z][A-Z0-9]+-\d{3,4}`) and the `scripts/lint-rule-ids.sh` allowlist (extended to include `QUAL` by this spec) |

The surface prefix disambiguates `QUAL-` IDs from `BE-`/`FE-`/`CFG-` IDs (AC #3). `scripts/lint-rule-ids.sh` is the machine-checked registry of accepted surfaces; this data-model is the documented source of truth its comment block cites for `QUAL`.

## `QUAL` rule ID format

```text
QUAL-{category}-{NNN}
```

| Element | Type | Constraints |
| --- | --- | --- |
| `QUAL` | literal | Always uppercase. The fixed prefix for code-quality rules. |
| `category` | string | Short uppercase abbreviation drawn from the table below (`[A-Z][A-Z0-9]*`, first char a letter). |
| `NNN` | integer | Zero-padded sequence number, starting at `001`. Never renumbered. Never reused after removal. |

### Category abbreviations

| Category | Abbreviation |
| --- | --- |
| Silent stubs | `STUB` |

The category set is declared in the `quality-cross.md` file header per the per-file category-declaration policy (`016-cross-cutting-rules`). It grows as concerns promote — adjacent categories (e.g. swallowed errors, dead code) are added when a concrete need appears, via `/gov:amend` or a follow-on spec.

## Initial rule set

The first version of `framework/rules/quality-cross.md` ships with one rule. Numbering is permanent.

### QUAL-STUB namespace (Silent stubs category)

- `QUAL-STUB-001` (MUST) — partial or unimplemented code paths fail loudly (panic / explicit error / failing test fixture) rather than silently passing through; stubs that return zero values, no-op middleware that returns `next` unchanged, handlers that return early without an error, and methods that return `nil, nil` are forbidden when the surrounding contract implies the path performs work. Verified at review time by `/gov:review`'s quality pass; cites `api-backend.md` `BE-SCHEMA-002` for the build-time schema case.

## Severity and ID-stability invariants

Inherited unchanged from `specs/008-security-rules/data-model.md`: MUST/MUST NOT are blocking errors, SHOULD/SHOULD NOT are advisory warnings; each Statement uses exactly one RFC 2119 keyword; an assigned ID is permanent (never renumbered, moved-within-file, or reused after removal); two rules in one file never share an ID.
