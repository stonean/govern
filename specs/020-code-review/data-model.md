---
status: draft
---

# 020 — `/gov:review` Data Model

Data structures introduced by [020 — `/gov:review`](spec.md). Authoritative shapes; the spec body and embedded `framework/commands/review.md` artifact reference these.

## Spec frontmatter `review:` block

Added to every spec's YAML frontmatter. Lazy-populated — the block is shipped in templates with safe defaults; existing adopter specs gain it on first `/gov:review` run.

```yaml
review:
  last-run: 2026-05-10T14:32:00Z       # ISO 8601; null until first review
  reviewed-against: <sha>              # HEAD SHA at review time; null until first review
  must-violations: 0                   # post-waiver count
  should-violations: 3
  low-confidence: 2
  blocking: false                      # true iff must-violations > 0
  waivers: []                          # see "Waiver record" below; omitted entirely when empty
```

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `last-run` | ISO 8601 timestamp or null | yes | Set by `/gov:review`. Null in templates and on un-reviewed specs. |
| `reviewed-against` | string (Git SHA) or null | yes | HEAD SHA at review time. Null in templates. |
| `must-violations` | integer ≥ 0 | yes | Count after waivers applied. |
| `should-violations` | integer ≥ 0 | yes | Advisory severity count. |
| `low-confidence` | integer ≥ 0 | yes | Quality-pass findings below 80 confidence. Excluded from `must-violations`. |
| `blocking` | boolean | yes | MUST equal `must-violations > 0`. Read by `/gov:implement`, `/gov:analyze`, CI template. |
| `waivers` | list of waiver records | no | Omitted entirely when empty. Schema below is open per §text-first-artifacts. |

### Validation severity

- **Hard fail** — block missing on a `done` spec; `blocking` not equal to `must-violations > 0`.
- **Blocking** (per §text-first-artifacts validation severity): `done` spec with `blocking: true` or missing `last-run`.
- **Informational** — unknown fields under `review:` (open-schema rule).

## Waiver record

One entry per waived MUST violation. Lives under `review.waivers` in spec frontmatter. The list itself is open-schema — adopters MAY add fields like `co-waived-by`, `approved-by-team`, `ticket`.

```yaml
- rule: SEC-BE-014
  file: src/api/internal.ts
  reason: "Endpoint is internal-only behind mTLS; rule applies to public APIs"
  waived-at: 2026-05-10T14:40:00Z
  waived-by: dev@example.com
```

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `rule` | string (rule ID) | yes | E.g. `SEC-BE-014`. Must reference a known rule at write time. |
| `file` | string (relative path) | yes | The path the waiver is anchored to. Waiver expires when the file moves or is deleted. |
| `reason` | string | yes | Free-text justification. Empty string is invalid. |
| `waived-at` | ISO 8601 timestamp | yes | Set by `/gov:review --waive`. |
| `waived-by` | string (email) | yes | Sourced from `git config user.email`. |
| (additional fields) | any | no | Open-schema; ignored by `/gov:review` and `/gov:analyze`. |

### Expiry rule

A waiver expires (is dropped from frontmatter on the next `/gov:review` run) when **either** of the following holds:

- The `file` path no longer exists in the repository (renamed or deleted).
- The named `rule` no longer fires at `file` (rule removed, or the violating code was fixed).

When the underlying finding still exists elsewhere in scope after expiry, it re-counts toward `must-violations` and the spec's `blocking` flag flips back to `true`. The detailed edge-case behavior is in [`scenarios/waiver-expiry.md`](scenarios/waiver-expiry.md).

## `review.md` artifact

Written to `specs/NNN-feature/review.md` (or `specs/NNN-feature/scenarios/SLUG/review.md` when targeting a scenario). Regenerated wholesale on each run.

### Frontmatter

```yaml
---
spec: 020-code-review
reviewed-at: 2026-05-10T14:32:00Z
reviewed-against: <sha-of-HEAD>
diff-base: <sha-where-status-became-in-progress>
must-violations: 0
should-violations: 3
low-confidence: 2
skipped-passes: []
---
```

| Field | Type | Required | Notes |
| --- | --- | --- | --- |
| `spec` | string | yes | Feature slug — matches the parent directory. |
| `reviewed-at` | ISO 8601 timestamp | yes | Set on each run. |
| `reviewed-against` | string (Git SHA) | yes | HEAD SHA at review time. |
| `diff-base` | string (Git SHA) | yes | The SHA where the spec advanced to `in-progress`, or the value passed to `--since=<ref>`. |
| `must-violations` | integer ≥ 0 | yes | Post-waiver count, matches spec frontmatter. |
| `should-violations` | integer ≥ 0 | yes | Matches spec frontmatter. |
| `low-confidence` | integer ≥ 0 | yes | Matches spec frontmatter. |
| `skipped-passes` | list of strings | yes | Empty when no flag restricts dimensions. Permitted values: `security`, `reuse`, `quality`, `efficiency`, `simplicity`. |

### Body sections (in order)

| Section | When emitted |
| --- | --- |
| `## Summary` | Always |
| `## MUST violations (blocking)` | Always (empty when none) |
| `## SHOULD violations (advisory)` | Always (empty when none) |
| `## Low-confidence findings` | Always (empty when none) |
| `## Waived findings` | Always (empty when none) |
| `## Skipped passes` | Always (empty when none) |

### Finding record

Each finding under MUST/SHOULD/Low-confidence sections:

```markdown
### MUST: <rule-id> — <one-line summary>

- **File**: `path/to/file.ts:42-55`
- **Rule**: <verbatim rule text from framework/rules/...>
- **Finding**: <one to three sentences>
- **Auto-fixable**: yes | no
- **Suggested fix**: <code block or prose>
```

Findings under **Waived findings** include an additional `**Waived**: <reason from spec frontmatter>` field.

### Idempotency invariant

For a given `(code-in-scope, loaded-rules, spec-acceptance-criteria, scenarios, waivers)` input set, the body of `review.md` is byte-identical across runs. Only `reviewed-at` and `reviewed-against` in the frontmatter are permitted to differ. This is the basis of acceptance criterion 6.

## `.govern.toml [review]` section

New TOML section in the project's `.govern.toml`. Added by `/gov:review` (with operator confirmation) on the first successful tech-stack alignment check. `.govern.toml` is shared adopter-side state per AGENTS.md (Workflow); this spec documents the section it adds rather than touching spec 019.

```toml
[review]
tech-stack-verified = true
```

| Key | Type | Default | Notes |
| --- | --- | --- | --- |
| `tech-stack-verified` | boolean | `false` (when key absent) | When `true`, `/gov:review` skips the tech-stack alignment pre-flight on every run until the operator removes the line. There is no auto-reset. |

The section is open-schema; future review-related persisted decisions can land under `[review]` without schema migration.

## Cross-references

- Spec lifecycle and §spec-requirements: [`framework/constitution.md`](../../framework/constitution.md).
- Open-schema rule for frontmatter: [`framework/constitution.md`](../../framework/constitution.md) §text-first-artifacts.
- Security rule ID conventions cited under `waivers[].rule`: [`framework/rules/security-backend.md`](../../framework/rules/security-backend.md), [`framework/rules/security-frontend.md`](../../framework/rules/security-frontend.md).
