---
spec: 020-code-review
status: in-progress
dependencies: []
review:
  last-run: 2026-05-17T22:55:00Z
  reviewed-against: 3794d7ed2b30593b8b5ce292f1d27b168b46405b
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 020 — `/gov:review` code review command with blocking gate

Adds `/gov:review`, a verb-named slash command that audits implementation code against the framework's rules across five dimensions (reuse, quality, security, efficiency, simplicity), writes a `review.md` artifact alongside the spec, and gates the `in-progress → done` transition via three reinforcing mechanisms.

## Summary

Add `/gov:review`, a comprehensive code review slash command covering reuse,
quality, security, efficiency, and simplicity. Reviews are written as
`review.md` artifacts alongside the spec they audit. MUST violations block
the spec from advancing to `done` via three reinforcing mechanisms
(`/gov:implement` halt, `/gov:analyze` drift detection, optional CI gate),
consistent with the constitution's quality standards and the **Design
Principles** rule that framework features must not depend on human diligence.

`/gov:review` audits **code against rules**. It is complementary to
`/gov:analyze`, which audits **artifacts against each other**.

## Motivation

The constitution references quality standards but no command enforces them on
implementation code. `/gov:analyze` ensures spec/plan/tasks artifacts are
internally consistent; nothing checks whether the code that landed actually
satisfies the spec, the security rules added in spec 008, or basic quality
expectations. Adopters currently have to remember to run external review
tools — a discipline dependency the framework should remove.

## Acceptance criteria

- [x] `/gov:review` exists as a verb-named slash command in the same shape as `/gov:analyze`, distributed through the standard `framework/commands/` → `.claude/commands/gov/` regeneration pipeline.
- [x] Running `/gov:review` against an `in-progress` target produces `specs/NNN-feature/review.md` with findings categorized into MUST, SHOULD, and low-confidence sections.
- [x] The command loads `framework/rules/security-backend.md` and `framework/rules/security-frontend.md` as authoritative security criteria. The five-dimension review model (security, reuse, quality, efficiency, simplicity) is applied to every targeted feature.
- [x] Spec frontmatter gains a `review:` block populated by the command: `last-run`, `must-violations`, `should-violations`, `blocking`, optional `waivers`.
- [x] **Blocking gate**: a spec with `review.blocking: true` cannot reach `done`.
  - `/gov:implement` halts before marking `done` and emits the blocking message.
  - `/gov:analyze` reports a violation when a spec at `status: done` has `review.blocking: true` or is missing `review.last-run`.
  - The shipped CI template includes a check that fails PRs in the same conditions.
- [x] Re-running `/gov:review` against unchanged code produces identical `review.md` content modulo timestamp and SHA fields (idempotency invariant).
- [x] `--fix` applies only conservative auto-fixes per the scope rules in the command file. Behavior-changing fixes are never auto-applied.
- [x] Waivers require explicit `--waive <rule-id> --reason "..."`, are recorded in spec frontmatter, and expire automatically when the file location or rule ID they were attached to changes.
- [x] Exit code is `0` when not blocking, `1` when blocking — for CI use.
- [x] The README slash commands table lists `/gov:review` under **Pipeline (advance state)** with a one-line purpose.
- [x] **Tech-stack alignment gate**: before running review passes, `/gov:review` confirms the project's `AGENTS.md` `Tech Stack` section exists and appears consistent with the implementation in scope. Misalignment or a missing/empty section is a blocking error, not a warning. Adopters can persist a successful check by setting `.govern.toml [review] tech-stack-verified = true`, after which subsequent runs skip the check until the operator manually clears the key.
- [x] **Empty scope**: a target with an empty resolved scope (no implementation files) produces a `review.md` recording 0 findings across all five passes, `blocking: false`, and exits `0`.
- [x] **Cross-pass dedupe**: when the same finding (matching rule ID, file, and overlapping line range) is produced by more than one pass, only the highest-severity instance is retained in `must-violations` and `should-violations`; lower-severity duplicates are dropped from the counts and report.

## Non-goals

- Replacing `/gov:analyze`. The two commands target different artifacts.
- Making `/gov:review` a pipeline-advance command in its own right. It is a
  gate, not a state transition.
- Shipping language- or framework-specific rule packs beyond the existing
  security rules. Adopters extend `framework/rules/` themselves.

## Affected files

| File | Change | Strategy |
| --- | --- | --- |
| `framework/commands/review.md` | **create** — full command file (see [Embedded artifacts](#embedded-artifacts)) | update |
| `framework/commands/implement.md` | edit — add blocking check before `status: done` transition | update |
| `framework/commands/analyze.md` | edit — add review-drift check, integrate with `--fix` | update |
| `framework/templates/spec/spec.md` | edit — add `review:` block to frontmatter schema | update |
| `framework/templates/spec/spec-and-plan.md` | edit — same `review:` block addition | update |
| `framework/templates/ci/adopter-generators.yml` | edit — add review-blocking check | update |
| `framework/constitution.md` | edit — reference the review gate in the pipeline section | update |
| `README.md` | edit — add `/gov:review` row to Pipeline commands table; document waivers | update |
| `scripts/regenerate-commands.sh` (or equivalent) | run after edits to regenerate `.claude/commands/gov/review.md` | n/a |
| `.claude/commands/gov/review.md` | regenerated output | derived |

## Tasks

1. Create `framework/commands/review.md` from the embedded artifact below.
2. Edit `framework/commands/implement.md`: before any logic that writes
   `status: done`, read the target spec's `review.blocking`. If `true` (or
   `review.last-run` is missing entirely), halt with the message specified
   in the [Blocking message](#blocking-message) section below and exit
   without modifying status.
3. Edit `framework/commands/analyze.md`: extend the audit to flag any spec
   at `status: done` with `review.blocking: true` or missing `review.last-run`
   as a validation failure. Wire `--fix` to revert affected specs from
   `done` → `in-progress` and emit a notice (never silent).
4. Edit `framework/templates/spec/spec.md` and `spec-and-plan.md` frontmatter
   to include the `review:` block schema (see [Frontmatter schema](#frontmatter-schema)).
5. Edit `framework/templates/ci/adopter-generators.yml` to add a step that
   exits non-zero if any `specs/*/spec.md` has `status: done` with
   `review.blocking: true` or missing `review.last-run`.
6. Edit `framework/constitution.md` pipeline section: add `/gov:review`
   between `/gov:implement` and the `done` transition, with a sentence
   explaining the blocking gate.
7. Edit `README.md`: add the `/gov:review` row to the Pipeline (advance
   state) table; add a short Waivers subsection under Slash Commands; update
   any pipeline diagrams.
8. Run the regeneration script to produce `.claude/commands/gov/review.md`.
9. Run `/gov:analyze --all` against the govern repo itself to confirm
   nothing in govern's own specs broke.
10. Add a scenario at `specs/020-code-review/scenarios/waiver-expiry.md`
    capturing the waiver auto-expiry behavior — this is the subtlest
    requirement and warrants a focused scenario.

## Open questions

*None — all resolved.*

## Resolved questions

- **MUST waivers and second sign-off** — single-author waivers remain the
  framework standard; `/gov:review` does not require a second `co-waived-by`
  field. Govern has no runtime that could verify a second value belongs to a
  different person, so encoding the requirement in frontmatter would be
  performative — exactly the "depend on human diligence" anti-pattern
  §pipeline-boundaries exists to avoid. The real second review happens at
  PR-merge time, where the waiver and the unfixed finding land in the diff
  and human code review applies org policy. Adopters whose policy does
  require two-author waivers can layer fields like `co-waived-by` onto the
  `review.waivers` entries — the §text-first-artifacts open-schema rule
  guarantees `/gov:review` and `/gov:analyze` will not error on unknown
  fields — and gate them in their own CI.
- **Stack detection source** — `/gov:review` continues to read `AGENTS.md`
  `Tech Stack` to choose between `security-backend.md` and
  `security-frontend.md`. Spec 004 (`done`) writes to that exact section
  during `/gov:init`, so `AGENTS.md` `Tech Stack` *is* the canonical sink
  for tech-stack metadata — there is no separate surface to point at. No
  change to the draft.
- **Quality-pass confidence threshold** — fixed at 80; not exposed via
  `.govern.toml`. The threshold is an opinion about LLM calibration, not
  about project domain — adopters have no meaningful information to tune it,
  only an incentive to raise it when reviews are inconvenient. Per
  §pipeline-boundaries ("never depend on human diligence"), making the gate
  tunable would let teams effectively waive it by setting the threshold to
  100. `.govern.toml` is reserved for genuine project-level decisions
  (pinned files, paths, agent identity per specs 017 and 019); a
  framework-wide quality opinion doesn't qualify. If model calibration
  shifts, the framework updates the value uniformly for all adopters.
- **`--all` scope** — `--all` reviews every feature whose status is
  `in-progress` or `done`. Excluding `done` would make the blocking gate
  retroactively blind to new MUST rules added after a feature shipped: the
  existing `/gov:analyze` drift check only fires when `review.blocking` is
  already `true` or `review.last-run` is missing, so rules introduced after
  the last review never re-flip the flag on shipped code. The §drift-prevention
  "done specs are frozen archaeology" rule applies to the spec body, not to
  the code the spec describes — that code keeps living and should stay
  compliant with current rules. Single-target `/gov:review` already accepts
  `done` (the gate halts only when status is *not* in `{in-progress, done}`);
  `--all` simply enumerates the same set the gate already permits.

## Frontmatter schema

The `review:` block added to spec frontmatter:

```yaml
review:
  last-run: 2026-05-10T14:32:00Z      # ISO 8601, set by /gov:review
  reviewed-against: <sha>             # HEAD sha at review time
  must-violations: 0                  # count after waivers applied
  should-violations: 3
  low-confidence: 2
  blocking: false                     # true iff must-violations > 0
  waivers:                            # optional, omitted when empty
    - rule: SEC-BE-014
      file: src/api/internal.ts
      reason: "Endpoint is internal-only behind mTLS"
      waived-at: 2026-05-10T14:40:00Z
      waived-by: dev@example.com
```

When any waiver's `file` no longer exists or `rule` is no longer triggered
at that location, the waiver is dropped on the next `/gov:review` run and
the underlying finding re-blocks if it's still present elsewhere.

## Blocking message

Emitted by `/gov:implement` when it would otherwise mark a spec `done`:

```text
blocked: spec NNN has N MUST violation(s) — see specs/NNN-feature/review.md

resolve the violations and re-run /gov:review,
or run /gov:review --waive <rule-id> --reason "..." for each waivable finding.
```

Emitted by `/gov:analyze` when it detects drift:

```text
review-drift: spec NNN at status=done with review.blocking=true
  → revert to in-progress and re-run /gov:review (use --fix to revert)
```

Emitted by `/gov:review` when tech-stack alignment fails (missing/empty
`AGENTS.md` `Tech Stack` section, or documented stack inconsistent with
implementation):

```text
blocked: tech-stack alignment failed — AGENTS.md Tech Stack {missing | inconsistent with code in scope}

  expected: <stack inferred from scope, e.g., "TypeScript + React frontend">
  documented: <AGENTS.md Tech Stack contents, or "(empty)">

reconcile AGENTS.md Tech Stack with the implementation, then re-run /gov:review.
to skip this check on future runs after manual reconciliation, add
[review] tech-stack-verified = true to .govern.toml.
```

---

## Embedded artifacts

### `framework/commands/review.md`

````markdown
---
description: Run a code review covering reuse, quality, security, efficiency, and simplicity; blocks `done` when MUST violations are present.
argument-hint: "[--all] [--fix] [feature]"
---

# /gov:review

Run a comprehensive code review against the targeted feature's implementation,
covering reuse, quality, security, efficiency, and simplicity. Produces a
`review.md` artifact alongside the spec. **Blocks the spec from reaching `done`
when MUST violations are present.**

`/gov:review` audits **code against rules**. It is complementary to `/gov:analyze`,
which audits **artifacts against each other**. Both should pass before a spec
advances to `done`.

## Inputs

- **Target** — the current `/gov:target` feature, or every feature with
  status `in-progress` or `done` when invoked with `--all`.
- **Rules** — `framework/rules/security-backend.md` and
  `framework/rules/security-frontend.md` are loaded by reference. RFC 2119
  language is authoritative: **MUST/MUST NOT** are blocking violations,
  **SHOULD/SHOULD NOT** are advisory.
- **Scope** — files referenced by the target's `plan.md` under `Affected Files`,
  plus any files modified since the spec advanced to `in-progress` (whichever
  set is larger). Lightweight-track features use the `Affected Files` section
  of `spec-and-plan.md`.
- **Config** — `.govern.toml` `[review] tech-stack-verified` (boolean,
  default `false`): when `true`, the tech-stack alignment check (see
  Behavior step 1) is skipped on every run until the operator clears the
  key. Set automatically (with operator confirmation) on the first
  successful alignment check.

## Flags

| Flag | Behavior |
| --- | --- |
| _(none)_ | Review the current target across all dimensions |
| `--all` | Review every feature with status `in-progress` or `done`. Composes with all other flags. |
| `--security` | Run only the security pass |
| `--simplicity` | Run only the reuse / quality / efficiency / simplicity passes |
| `--quality` | Run only the correctness / bug-detection pass |
| `--fix` | Apply auto-fixable findings (see [Auto-fix scope](#auto-fix-scope) below) |
| `--since=<ref>` | Override the diff base (default: commit at which spec advanced to `in-progress`) |
| `--waive <rule-id> --reason "<text>"` | Record a waiver for a MUST violation (see [Waivers](#waivers)) |

## Pipeline position

`/gov:review` runs after `/gov:implement` has produced code and before the spec
can advance to `done`. The recommended sequence is:

```
/gov:implement   →   /gov:review   →   /gov:analyze   →   spec status: done
```

`/gov:implement` MUST NOT mark a spec `done` while the target's `review.md`
records `must-violations: > 0`. See [Blocking semantics](#blocking-semantics).

## Behavior

For each targeted feature, in order:

### 1. Resolve target and scope

1. Resolve the working feature from `--all` or the current `/gov:target`.
   If neither yields a target, halt with `no target — run /gov:target first`.
2. Read the spec frontmatter. If `status` is not in `{in-progress, done}`,
   halt with `review only runs against in-progress or done specs`.
3. Build the file scope per [Inputs](#inputs). If the resolved scope is
   empty (no implementation files yet), write a `review.md` recording 0
   findings across all five passes, `blocking: false`, and exit `0` — there
   is nothing to review yet. Skip steps 4–5 and the rest of this run.
4. **Tech-stack alignment check.**
   - Read `.govern.toml`. If `[review] tech-stack-verified = true`, skip to
     step 5.
   - Otherwise, read `AGENTS.md`'s `Tech Stack` section and inspect the file
     scope (extensions, imports, runtime/manifest markers). Confirm the
     documented stack appears consistent with the implementation. A
     missing or empty `Tech Stack` section, or an inconsistency between
     documentation and code, halts the run with the
     [tech-stack-misalignment](#blocking-message) message and exits `1`.
   - On a successful check, prompt the operator once: _"Tech-stack
     alignment confirmed. Persist this so future runs skip the check?
     (Y/n)"_. On `Y`, write `[review] tech-stack-verified = true` to
     `.govern.toml`. On `n` or skip, the check runs again on the next
     invocation. To re-run the check after a stack change, the operator
     removes the line manually — `/gov:review` does not auto-reset.
5. Select rule files per the (now-verified) tech stack: load
   `security-backend.md` for backend stacks, `security-frontend.md` for
   frontend, and both for full-stack projects.

### 2. Load rules

Load these files inline as the authoritative review criteria:

- `framework/rules/security-backend.md` (if backend stack present)
- `framework/rules/security-frontend.md` (if frontend stack present)
- Any other `framework/rules/*.md` referenced from `AGENTS.md`
- `AGENTS.md` `Code Style`, `Testing`, `Gotchas`, and `Boundaries` sections
- The target spec's acceptance criteria and any `scenarios/*.md` files

These are the **only** sources of normative rules for the review. Do not
introduce review criteria from outside the project.

### 3. Run review passes

Run the five passes below. When a flag restricts dimensions (`--security`,
`--simplicity`, `--quality`), skip the unselected passes and record them as
`skipped` in the report.

When the same finding (matching rule ID, file, and overlapping line range)
is produced by more than one pass, retain only the highest-severity instance
in `must-violations` and `should-violations`; lower-severity duplicates are
dropped from the counts and the report. Pass-of-record for the surviving
finding is the highest-severity pass that flagged it.

#### Security pass

Walk every file in scope against the loaded security rules. For each finding,
record: rule ID, severity (MUST or SHOULD), file path, line range, the rule
text, and a one-sentence explanation of how the code violates it. **Do not
flag patterns that are not in the loaded rules** — the project's rule set is
authoritative.

#### Reuse pass

Identify logic that duplicates existing utilities or that should be extracted
into shared code. Cross-reference with `specs/system.md` for established
patterns and shared infrastructure. Severity is SHOULD unless the duplication
contradicts an explicit MUST in `AGENTS.md` `Boundaries`.

#### Quality pass

Detect bugs, missing error handling, unhandled edge cases, off-by-one errors,
and contract violations against `specs/errors.md`. Each finding includes a
confidence score 0–100. Findings below 80 confidence are recorded as
`low-confidence` and excluded from the blocking count.

#### Efficiency pass

Flag N+1 queries, repeated work, unbounded loops over user-controlled input,
and other performance issues. Severity is SHOULD by default; promote to MUST
when the inefficiency is also a security concern (e.g. unbounded input is a
DoS vector covered by the security rules).

#### Simplicity pass

Identify overengineering: premature abstraction, unnecessary indirection,
configuration that could be a constant, branches that are dead under the
current spec. Severity is SHOULD. If a simpler form is mechanically derivable,
mark the finding `auto-fixable`.

### 4. Write `review.md`

Write the report to `specs/NNN-feature/review.md`. A scenario-targeted run still writes to the same spec-level path; the `scenario:` frontmatter field records which scenario was reviewed and `reviewed-against` records the commit. Re-running review supersedes the prior `review.md` wholesale.

```markdown
---
spec: 042-example-feature
reviewed-at: 2026-05-10T14:32:00Z
reviewed-against: <sha-of-HEAD>
diff-base: <sha-where-status-became-in-progress>
must-violations: 0
should-violations: 3
low-confidence: 2
skipped-passes: []
---

# Review — 042-example-feature

## Summary

<one paragraph: overall posture, count by severity, blocking status>

## MUST violations (blocking)

<empty section when none; otherwise one heading per finding>

## SHOULD violations (advisory)

## Low-confidence findings

## Waived findings

## Skipped passes

<empty when none>
```

Each finding follows this shape:

```markdown
### MUST: <rule-id> — <one-line summary>

- **File**: `path/to/file.ts:42-55`
- **Rule**: <verbatim rule text from framework/rules/...>
- **Finding**: <one to three sentences>
- **Auto-fixable**: yes | no
- **Suggested fix**: <code block or prose>
```

The report is regenerated on every `/gov:review` run — never appended.
Findings the user has explicitly waived (see [Waivers](#waivers)) carry across
runs as long as their anchor (rule + file) is still valid.

### 5. Apply `--fix` (optional)

When `--fix` is set, after writing the report:

1. Apply every finding marked `auto-fixable: yes` whose severity is SHOULD,
   plus MUST findings whose suggested fix is purely mechanical (e.g. removing
   a hardcoded secret, adding a missing CSRF token attribute).
2. **Never** auto-apply fixes that alter externally observable behavior, change
   error messages or status codes, or modify schema. These require a manual pass.
3. Re-run only the affected passes against the modified files. Update
   `review.md` with the post-fix counts.
4. Stage the modified files but do not commit. The user owns the commit.

### 6. Update spec frontmatter

After writing the report, update the target spec's frontmatter:

```yaml
review:
  last-run: 2026-05-10T14:32:00Z
  reviewed-against: <sha>
  must-violations: 0
  should-violations: 3
  blocking: false
```

`blocking: true` when `must-violations > 0`. This is the field other commands
read.

## Blocking semantics

A spec MUST NOT advance from `in-progress` to `done` while its frontmatter
records `review.blocking: true`. This is enforced as follows:

1. **`/gov:implement`** — before marking `status: done`, reads
   `review.blocking`. If `true` (or `review.last-run` is missing), halts with:

   ```
   blocked: spec has N MUST violation(s) — see specs/NNN-feature/review.md
   resolve the violations and re-run /gov:review, or waive with /gov:review --waive
   ```

2. **`/gov:analyze`** — adds a check to its existing audit: if the spec's
   status is `done` but `review.blocking` is `true` or `review.last-run` is
   missing, this is a validation failure. Composable with `--fix`:
   `/gov:analyze --fix` reverts `done` → `in-progress` and emits a notice
   (it never silently downgrades; the notice is the point).

3. **CI hook** — the shipped GHA template at
   `framework/templates/ci/adopter-generators.yml` fails when any
   spec at `status: done` has `review.blocking: true` or missing
   `review.last-run`.

This implements the constitution's quality gate via three mutually reinforcing
mechanisms rather than relying on any single one — consistent with the
**Design Principles** rule: never depend on human diligence.

## Blocking message

Emitted by `/gov:review` when tech-stack alignment fails (missing/empty
`AGENTS.md` `Tech Stack` section, or documented stack inconsistent with
implementation):

```text
blocked: tech-stack alignment failed — AGENTS.md Tech Stack {missing | inconsistent with code in scope}

  expected: <stack inferred from scope, e.g., "TypeScript + React frontend">
  documented: <AGENTS.md Tech Stack contents, or "(empty)">

reconcile AGENTS.md Tech Stack with the implementation, then re-run /gov:review.
to skip this check on future runs after manual reconciliation, add
[review] tech-stack-verified = true to .govern.toml.
```

## Waivers

A MUST violation can be waived only with explicit, recorded justification:

```
/gov:review --waive <rule-id> --reason "<text>"
```

This appends to the target spec's frontmatter:

```yaml
review:
  waivers:
    - rule: SEC-BE-014
      file: src/api/internal.ts
      reason: "Endpoint is internal-only behind mTLS; rule applies to public APIs"
      waived-at: 2026-05-10T14:40:00Z
      waived-by: <git config user.email>
```

Waived findings drop out of `must-violations` count and into a separate
`waived-violations` count. They appear in `review.md` under the **Waived
findings** section. They survive across `/gov:review` runs as long as the
rule ID and file location still match; if either changes, the waiver expires
and the finding re-blocks. Line numbers are not part of the waiver anchor —
the contract is `(rule, file)`, so code moving within the file does not
expire the waiver.

### Per-run waiver processing

At the start of every `/gov:review` run, before counting findings into
`must-violations`, walk `review.waivers` and process each entry:

1. **Apply** when the file exists at the anchored path AND the rule still
   fires there. The finding appears under **Waived findings** in
   `review.md` with the waiver's `reason`; it is excluded from
   `must-violations`.
2. **Expire** when either the file no longer exists at the anchored path
   (renamed, deleted, moved) or the rule no longer fires there (offending
   code fixed, rule removed, rule renamed — IDs are permanent per
   `specs/008-security-rules/data-model.md`, so a renamed rule is a
   different rule). On expiry, drop the entry from `review.waivers` on the
   next frontmatter write AND emit one line on stdout:

   ```text
   waiver expired: rule {rule-id} at {file} ({reason})
   ```

   The notice is the point of the action; expiry MUST NOT be silent. When
   the same rule still fires anywhere in scope after a drop, the finding
   re-counts toward `must-violations` and `review.blocking` flips to
   `true` if it was previously `false`.
3. **Do not extend** a waiver to a different file. If the same rule fires
   at a path other than the waiver's anchor, that is a separate finding;
   the operator records a fresh `--waive` if it is also intentional.

### Malformed and duplicate waivers

- A waiver entry missing any of `rule`, `file`, `reason`, `waived-at`, or
  `waived-by` is **skipped** with a one-line warning naming the offending
  entry (e.g. `malformed waiver at review.waivers[2]: missing 'reason'`).
  The entry is NOT auto-removed; the operator must clean it up to silence
  the warning. Malformed entries are operator-authored state, not garbage
  for the framework to collect.
- Two or more waivers for the same `(rule, file)` pair: **only the first
  applies**. Each duplicate emits a one-line warning
  (`duplicate waiver: rule {rule-id} at {file} — entry [N] ignored`) and is
  NOT auto-pruned. Same reasoning: the framework treats duplicates as
  operator state worth investigating, not silent state to clean up.

The `review.waivers` list follows the §text-first-artifacts open-schema
rule. Adopters MAY add fields (e.g., `co-waived-by`, `approved-by-team`,
`ticket`) to enforce org-specific waiver policy in their own CI; `/gov:review`
and `/gov:analyze` will not error on unknown fields.

## Auto-fix scope

`--fix` is conservative by design. It applies fixes when **all** of these hold:

- The finding is marked `auto-fixable: yes`
- The fix does not change function signatures, return types, or schema
- The fix does not change observable HTTP status codes, error messages, or
  log formats
- The fix does not delete tests
- The fix is contained to files already in the review scope

When in doubt, leave the finding unfixed and let the user apply the
`Suggested fix` manually.

## Output

Stdout summary (always), followed by the path to `review.md`:

```
/gov:review — 042-example-feature

  security    ✓ 0 MUST   2 SHOULD
  reuse       ✓ 0 MUST   1 SHOULD
  quality     ✓ 0 MUST   0 SHOULD   (2 low-confidence)
  efficiency  ✓ 0 MUST   0 SHOULD
  simplicity  ✓ 0 MUST   0 SHOULD

  blocking: no
  report:   specs/042-example-feature/review.md
```

When MUST violations are present:

```
/gov:review — 042-example-feature

  security    ✗ 2 MUST   1 SHOULD
  reuse       ✓ 0 MUST   0 SHOULD
  quality     ✗ 1 MUST   0 SHOULD
  efficiency  ✓ 0 MUST   0 SHOULD
  simplicity  ✓ 0 MUST   0 SHOULD

  blocking: yes — 3 MUST violations
  report:   specs/042-example-feature/review.md

  spec cannot advance to done. Resolve violations and re-run /gov:review,
  or run /gov:review --waive <rule-id> --reason "..." for each waivable finding.
```

Exit code: `0` when not blocking, `1` when blocking. Allows CI to gate on the
exit code without parsing the report.

## Idempotency

Re-running `/gov:review` against an unchanged target reproduces an identical
`review.md` (modulo `reviewed-at` and `reviewed-against`). This is a
derive-don't-ask invariant: review output is a function of code + rules,
never of session state.

## Notes for adopters

- Projects that customize `framework/rules/security-{backend,frontend}.md`
  pin them in `.govern.toml` `[pinned] files` to prevent `/govern` from
  overwriting their additions. `/gov:review` reads whatever is on disk —
  pinned or not.
- Projects on a stack not covered by the shipped rule files should add
  their own at `framework/rules/<domain>.md` and reference them from
  `AGENTS.md`. `/gov:review` automatically loads anything in
  `framework/rules/` that's referenced from `AGENTS.md`.
- The five-dimension model is fixed. Domain-specific concerns (accessibility,
  i18n, licensing) belong in additional rule files, not new passes.
````

### `framework/commands/implement.md` — required edits

Locate the section that transitions `status: in-progress` → `status: done`.
Before that transition, insert:

````markdown
### Pre-`done` review gate

Before writing `status: done` to the target spec's frontmatter, read the
spec's `review:` block:

1. If `review.last-run` is missing, halt with:

   ```text
   blocked: spec has not been reviewed — run /gov:review before completing
   ```

2. If `review.blocking: true`, halt with:

   ```text
   blocked: spec has {must-violations} MUST violation(s) — see specs/NNN-feature/review.md

   resolve the violations and re-run /gov:review,
   or run /gov:review --waive <rule-id> --reason "..." for each waivable finding.
   ```

3. Otherwise, proceed with the `done` transition.

This gate runs after all tasks are complete but before status changes — the
spec stays at `in-progress` until review is clean.
````

### `framework/commands/analyze.md` — required edits

In the audit section that walks each spec, add a check:

```markdown
### Review drift

For each spec at `status: done`:

- If `review.last-run` is missing → record a violation:
  `done spec missing review — run /gov:review`
- If `review.blocking: true` → record a violation:
  `done spec has unresolved MUST violations — see review.md`

When `--fix` is set, revert affected specs to `in-progress` and emit a
notice for each (one line per spec — never silent). Re-running `/gov:review`
on each is left to the user; auto-running it during `--fix` is out of scope.
```

### Spec template frontmatter additions

Add to both `framework/templates/spec/spec.md` and
`framework/templates/spec/spec-and-plan.md` frontmatter section:

```yaml
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
  waivers: []
```

`null` indicates the review has not yet been run. `/gov:review` populates
the fields on its first run.

### CI template addition

Append to `framework/templates/ci/adopter-generators.yml`:

```yaml
  review-gate:
    name: Block done specs with unresolved MUST violations
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Check review status on done specs
        run: |
          set -euo pipefail
          fail=0
          for spec in specs/*/spec.md specs/*/spec-and-plan.md; do
            [ -f "$spec" ] || continue
            status=$(awk '/^status:/{print $2; exit}' "$spec")
            [ "$status" = "done" ] || continue
            blocking=$(awk '/^  blocking:/{print $2; exit}' "$spec")
            last_run=$(awk '/^  last-run:/{print $2; exit}' "$spec")
            if [ "$blocking" = "true" ] || [ -z "$last_run" ] || [ "$last_run" = "null" ]; then
              echo "::error file=$spec::done spec missing or blocked review"
              fail=1
            fi
          done
          exit $fail
```

### Constitution edit

In `framework/constitution.md`, locate the development pipeline section and
update it to read (preserving surrounding prose):

```markdown
The pipeline is: `/specify` → `/clarify` → `/plan` → `/implement` →
`/review` → `/analyze` → `done`.

`/review` is a quality gate, not a state transition. A spec cannot advance
from `in-progress` to `done` while its `review.md` records MUST violations.
The gate is enforced by `/implement`, by `/analyze`, and by the optional CI
template — three mutually reinforcing checks rather than one, consistent
with the Design Principles rule that framework features must not depend on
human diligence.
```

### README edits

In the **Pipeline (advance state)** table, insert before `/analyze`:

```markdown
| `/review` | Comprehensive code review across reuse, quality, security, efficiency, and simplicity. Writes `specs/NNN-feature/review.md`. Blocks the spec from reaching `done` when MUST violations are present. `--all` reviews every `in-progress` feature. `--fix` applies conservative auto-fixes. Composable: `--all --fix` |
```

Add a brief subsection under Slash Commands documenting the waiver flow and
linking to spec 020 for the full schema.
