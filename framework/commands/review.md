---
description: Audit code against rules — security, reuse, quality, efficiency, simplicity. Writes review.md; blocks done on MUST violations.
argument-hint: "[--all] [--fix] [feature]"
---

# /{project}:review

Run a comprehensive code review against the targeted feature's implementation,
covering reuse, quality, security, efficiency, and simplicity. Produces a
`review.md` artifact alongside the spec. **Blocks the spec from reaching `done`
when MUST violations are present.**

`/{project}:review` audits **code against rules**. It is complementary to `/{project}:analyze`,
which audits **artifacts against each other**. Both should pass before a spec
advances to `done`.

## Purpose

Quality gate before `done`: audit the feature's implementation against the project's rule files across five dimensions (security, reuse, quality, efficiency, simplicity), record the findings in `specs/NNN/review.md`, and set the spec's `review.blocking` frontmatter so `/{project}:implement`, `/{project}:analyze`, and the CI hook can hold the spec out of `done` while MUST violations stand. Waivers (with recorded justification) are the sanctioned escape.

## Scope Boundaries

- Reads the target spec, its `plan.md` (for Affected Files), the in-scope source files, the selected rule files, `AGENTS.md`, and `.govern/config.toml`; diffs `specs/inbox.md` over the review window. Do NOT review files outside the resolved scope, and do NOT introduce review criteria from outside the project's rule files and `AGENTS.md`.
- Writes exactly two artifacts: `specs/NNN/review.md` and the target spec's frontmatter `review:` block (via `write-review`); with `--waive`, appends a waiver entry; with `--fix`, applies auto-fixable findings to the working tree. No other files are modified — status transitions belong to `/{project}:implement`.
- Reference: §runtime-host-integration, §brownfield-inbox, §text-first-artifacts (constitution loaded by `/{project}:target` — do not re-read).

## Inputs

- **Target** — the current `/{project}:target` feature, or every feature with
  status `in-progress` or `done` when invoked with `--all`.
- **Rules** — every file under the project's rule-file directory
  (`framework/rules/` in govern's own repo, `specs/rules/` in adopter
  projects) selected by the suffix-based discovery in step 2
  (`discover-rule-files`); their content is the authoritative review
  criteria. RFC 2119 language is authoritative:
  **MUST/MUST NOT** are blocking violations, **SHOULD/SHOULD NOT** are
  advisory.
- **Scope** — files referenced by the target's `plan.md` under `Affected Files`,
  plus any files modified since the spec advanced to `in-progress` (whichever
  set is larger). `specs/inbox.md` is also read (diffed against `diff-base`) to
  surface issues captured during the work window — see step 1 (`compute-review-scope`).
- **Config** — three `.govern/config.toml` keys influence this command:
  - `[review] tech-stack-verified` (boolean, default `false`): when
    `true`, the tech-stack alignment check (see step 1) is
    skipped on every run until the operator clears the key. Set
    automatically (with operator confirmation) on the first successful
    alignment check.
  - `[[review.disabled-rule-files]]` (array-of-tables, default empty):
    each entry has a required `file` field (basename of a file in the
    rule-file directory — `framework/rules/` here, `specs/rules/` in
    adopter projects — e.g., `"accessibility-frontend.md"`) and a
    required `reason` field (free-text justification; trimmed length
    ≥ 16 Unicode codepoints). Files listed here are excluded from
    rule-file selection regardless of stack detection. Consulted in
    step 2 (`discover-rule-files`). Reason is mandatory — it is the
    audit trail for the override.
  - `[rules] surfaces` (array of strings, default unset): the project's
    rule surfaces, members in `{"backend", "frontend"}` (full-stack lists
    both; `*-cross.md` files are unconditional and not members). When set,
    it is the source of truth for surface selection in step 2
    (`discover-rule-files`) and replaces stack detection; when unset, step 2
    falls back to the detected stack (and with no detected surfaces supplied,
    the primitive loads **all** recognized surfaces). The **empty list**
    (`[]`) is valid and means cross-only (not the same as unset); an
    unrecognized member (including `"cross"`) or a non-list value fails fast
    in step 2. Collected and persisted
    by `/govern` (`govern.md` §Collect Project Inputs).

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

`/{project}:review` runs after `/{project}:implement` has produced code and before the spec
can advance to `done`. The recommended sequence is:

```text
/{project}:implement   →   /{project}:review   →   /{project}:analyze   →   spec status: done
```

`/{project}:implement` MUST NOT mark a spec `done` while the target's `review.md`
records `must-violations: > 0`. See [Blocking semantics](#blocking-semantics).

## Instructions

> **For agent runtimes**: the Invoke steps below call the MCP tools of the optional gvrn runtime; the host-integration contract — bare↔prefixed tool names, lazy ToolSearch schema fetch, the no-shell-utilities rule, and the two-paths guarantee — lives once in the constitution, §runtime-host-integration. With no gvrn MCP server registered, walk the same prose using the host file-reading tools (Read, Edit, Write).

Run once per targeted feature (every in-progress or done spec under `--all`, otherwise the current `/{project}:target`), in order. Resolve a `[feature]` argument through `resolve-feature` (exact name / number / unique partial slug), and enumerate the `--all` set from `dashboard`'s per-spec status inventory (`specs[].status ∈ {in-progress, done}`) rather than a directory scan. The detailed walk — rule-selection notices, waiver semantics, the report skeleton, and the pass definitions — lives under the Markdown-only reference section below.

1. Invoke `compute-review-scope` to resolve the diff base (the commit the spec advanced to in-progress at, or a `--since` override), the review file scope (whichever is **larger** of the plan's Affected Files and the files modified since the diff base — not a union; ties resolve to the modified-since set), and the inbox additions captured in that window. When the scope is empty, jump straight to the write-review step (step 9) — it emits the nothing-to-review-yet, non-blocking report. Otherwise confirm tech-stack alignment first (host judgment, not a primitive): read the active config file; when its `[review] tech-stack-verified` flag is true, skip the check; else compare the AGENTS.md Tech Stack section against the code in scope, halting with the tech-stack-misalignment message on a mismatch, and — on success — confirm before persisting the flag (the same confirm-before-write gate the other pipeline steps use; see the tech-stack alignment step in the markdown-only reference) and write `[review] tech-stack-verified = true` to the **active config file** (`.govern/config.toml` when it exists, else the legacy root `.govern.toml` when that exists, else `.govern/config.toml`; spec 042 — a write outside the `/govern` migration never creates a partial `.govern/config.toml` alongside a lingering legacy file). Only the flag read is deterministic — the alignment judgment stays with the host.
2. Invoke `discover-rule-files` to select this run's rule files — suffix classification, the `[rules] surfaces` selection, and the disabled-rule-files filter — and emit the ordered notice lines it returns verbatim.
3. <!-- llm:performReview --> Run the **security** pass over the in-scope files against the loaded security rules, returning one finding per violation (rule id, severity, file, line range, confidence, explanation).
4. <!-- llm:performReview --> Run the **reuse** pass: flag logic that duplicates existing utilities or belongs in shared code.
5. <!-- llm:performReview --> Run the **quality** pass: detect bugs, missing error handling, unhandled edge cases, and contract violations; low-confidence findings are recorded separately and do not block.
6. <!-- llm:performReview --> Run the **efficiency** pass: flag N+1 queries, repeated work, and unbounded loops over user-controlled input.
7. <!-- llm:performReview --> Run the **simplicity** pass: flag overengineering, premature abstraction, and dead branches; mark a finding auto-fixable when a simpler form is mechanically derivable. A dimension-restricting flag (`--security` / `--simplicity` / `--quality`) skips the unselected passes.
8. Invoke `process-waivers` to classify the spec's `review.waivers` against the findings the passes just accumulated (apply / expire / retain / malformed / duplicate), emitting each notice it returns. **On a dimension-restricted run (`--security` / `--simplicity` / `--quality`), pass the skipped dimensions as `skipped-passes`** so a waiver whose rule did not fire is _retained_, not expired — the partial run cannot see the dimensions it didn't run, so it must not prune their waivers. The applied set is excluded from the blocking count; the expired set is dropped on the next write; the retained set is left in the frontmatter untouched. On an unrestricted run `skipped-passes` is empty and a waiver expires only when its file is gone or its rule genuinely no longer fires.
9. Invoke `write-review` with the accumulated pass findings, the waiver results (`applied` / `expired`), and the scope to render `specs/NNN-feature/review.md` and update the spec `review:` frontmatter block. Supply the required scalars the primitives don't produce — `reviewed-at` (the current UTC timestamp) and `reviewed-against` (HEAD sha), both host-provided (as the session-write's `set-at` is); `diff-base` comes from step 1. It applies the cross-pass dedup (highest-severity-wins on rule + file + overlapping range), buckets findings into MUST / SHOULD / low-confidence / waived, prunes expired waivers (preserving any adopter-authored waiver fields on the survivors), records the skipped passes, and sets blocking when MUST violations remain. With `--fix`, apply the auto-fixable findings, re-run the affected passes, and invoke `write-review` a second time for the post-fix counts.

## Markdown-only reference

The numbered Instructions above are the deterministic path — the runtime's primitives own the rule-file selection, waiver arithmetic, scope resolution, and report scaffolding, and the five passes cross the boundary at the `performReview` extension point. When no runtime is available, walk the detailed procedure below by hand, for each targeted feature, in order.

### 1. Resolve target and scope

1. Resolve the working feature from `--all` or the current `/{project}:target`.
   If neither yields a target, halt with `no target — run /{project}:target first`.
2. Read the spec frontmatter. If `status` is not in `{in-progress, done}`,
   halt with `review only runs against in-progress or done specs`.
3. Build the file scope per [Inputs](#inputs). If the resolved scope is
   empty (no implementation files yet), write a `review.md` recording 0
   findings across all five passes, `blocking: false`, and exit `0` — there
   is nothing to review yet. Skip steps 4–5 and the rest of this run.
4. **Tech-stack alignment check.**
   - Read the active config file (`.govern/config.toml` when it exists,
     else the legacy root `.govern.toml`). If `[review] tech-stack-verified = true`, skip to
     step 5.
   - Otherwise, read `AGENTS.md`'s `Tech Stack` section and inspect the file
     scope (extensions, imports, runtime/manifest markers). Confirm the
     documented stack appears consistent with the implementation. A
     missing or empty `Tech Stack` section, or an inconsistency between
     documentation and code, halts the run with the
     [tech-stack-misalignment](#blocking-message) message and exits `1`.
   - On a successful check, prompt the operator once (routing the prompt
     through `gate-confirm` on the runtime path, as the other
     confirm-before-write pipeline steps do): _"Tech-stack alignment
     confirmed. Persist this so future runs skip the check? (Y/n)"_. On
     `Y`, write `[review] tech-stack-verified = true` to the active config
     file (`.govern/config.toml` when it exists, else the legacy root
     `.govern.toml` when that exists, else `.govern/config.toml`; spec
     042). On `n` or skip, the check runs again on the next
     invocation. To re-run the check after a stack change, the operator
     removes the line manually — `/{project}:review` does not auto-reset.
5. Discover rule files by suffix. List `framework/rules/*.md` in govern's
   own repository, or `specs/rules/*.md` in adopter projects. For each
   file, classify by basename suffix:
   - `*-backend.md` → backend surface
   - `*-frontend.md` → frontend surface
   - `*-cross.md` → cross-cutting (applies to every stack)
   - anything else → unrecognized — load for every stack and emit one
     stdout line per file:

     ```text
     rule file <name> has unrecognized suffix — loading for all stacks; rename to -backend.md, -frontend.md, or -cross.md
     ```

   Determine the **surface selection** for this run. Read `.govern/config.toml`
   `[rules] surfaces` (see [Inputs](#inputs)):

   - **Set to a valid list** (every member in `{backend, frontend}`) —
     keep the rule files whose surface is listed in `surfaces`, plus
     every `*-cross.md`. This explicit operator-set selection _replaces_
     the detected-stack filter; the stack from step 4 is not consulted
     for rule-file selection. The **empty list** (`surfaces = []`) is a
     valid value of this case and means **cross-only**: no
     `*-backend.md`/`*-frontend.md` file is kept, only `*-cross.md`. The
     empty list is distinct from the key being unset (below) — it is the
     operator explicitly declaring "no surface rules, only cross-cutting
     ones," not a request to derive.
   - **Set to a degenerate value** — fail fast (do not silently ignore,
     do not warn-and-continue), consistent with `CFG-ENV-003`'s
     fail-fast-on-invalid-configuration posture:
     - **Unrecognized member.** A member outside `{backend, frontend}` —
       a typo like `"fullstack"`, or `"cross"` (cross-cutting files are
       unconditional, not a selectable surface) — halts the run with
       `/{project}:review: invalid [rules] surfaces member "<value>" — accepted members are "backend" and "frontend" (use [] for cross-only; -cross.md files always apply)`.
       A list mixing valid and invalid members (`["backend", "fullstack"]`)
       fails on the invalid member; a valid member does not rescue it.
     - **Type mismatch.** A non-list value (`surfaces = "backend"`, a
       bare string) halts the run with
       `/{project}:review: [rules] surfaces must be a list of strings, got <type>`.
   - **Unset** — fall back to the detected stack from step 4: keep the
     matching surface, keep every `*-cross.md` (pre-033 behavior).

   In every non-error case, keep every unrecognized-suffix file
   unconditionally.

   Then apply the **disabled-rule-files filter**. Read `.govern/config.toml`
   `[[review.disabled-rule-files]]` (see [Inputs](#inputs)). For each
   entry, in list order:

   - **Drop + notice (selected match).** `file` matches the
     basename of a file currently in the post-selection set. Remove
     it from the set and emit one line:

     ```text
     disabled-rule-file: <filename> — <reason> (.govern.toml)
     ```

     Collapse internal whitespace in `reason` (including newlines from
     TOML multi-line strings) to single spaces before emitting — the
     notice is single-line by contract.

   - **No-op notice (non-selected match).** `file` matches a
     basename in the rule-file directory but the file was NOT in the
     post-selection set (different surface). Emit one line and
     change nothing:

     ```text
     disabled-rule-file (no-op): <filename> not selected by stack detection
     ```

     This is honest about state — the entry is currently a no-op,
     becomes load-bearing if the project's stack changes later.

   - **Unknown warning.** `file` does not match any basename in the
     rule-file directory. Emit one line and change nothing:

     ```text
     unknown disabled-rule-file: <filename> (no such file in the rule-file directory)
     ```

     This covers renamed/moved files; not a fatal error.

   - **Malformed warning.** Entry is missing `file` or `reason`, or
     `reason`'s trimmed length is < 16 Unicode codepoints. Skip the
     entry (no file is dropped) and emit one line naming the offending
     index (same pattern as §Malformed and duplicate waivers below):

     ```text
     malformed disabled-rule-file at review.disabled-rule-files[N]: <reason>
     ```

     The entry is NOT auto-removed; the operator cleans it up.

   - **Duplicate warning.** Same `file` listed twice. Only the first
     entry applies; each subsequent duplicate emits one line and is
     not auto-pruned:

     ```text
     duplicate disabled-rule-file: <filename> — entry [N] ignored
     ```

   All four warning forms emit to stdout and **do not affect the exit
   code**. `/{project}:review`'s exit status is driven exclusively by MUST
   violations (see [Output](#output)). `.govern/config.toml` hygiene is a
   separate concern.

   Finally, emit a single stdout line naming what was selected:

   ```text
   loading rule files: <comma-separated basenames>
   ```

   Disabled files are excluded from this list. The notice fires AFTER
   all disabled-rule-file lines, so a normal run reads top-down as:
   any `disabled-rule-file: …` notices, then `loading rule files: …`.
   This is the discoverability surface — adopters can confirm which
   files were considered without parsing the report.

### 2. Load rules

Load these inputs inline as the authoritative review criteria:

- Every rule file selected by the suffix-based discovery in step 5
- Any rule file outside the rule-file directory (e.g., `docs/rules/internal-api.md`)
  referenced from `AGENTS.md` — see [Notes for adopters](#notes-for-adopters)
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
and contract violations against `specs/errors.md`. Each finding carries a
`confidence` tier — `high` or `low` (the string the `write-review` contract
consumes, compared case-insensitively). A `low`-confidence finding is recorded
in the Low-confidence section regardless of severity and is excluded from the
blocking count; use it when the finding is plausible but unconfirmed.

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

Write the report to `specs/NNN-feature/review.md`. A scenario-targeted run still writes to the same spec-level path; the `scenario:` frontmatter field records which scenario was reviewed and `reviewed-against` records the commit. Re-running review (scenario- or feature-targeted) supersedes the prior `review.md` wholesale.

```markdown
---
spec: 042-example-feature
reviewed-at: 2026-05-10T14:32:00Z
reviewed-against: <sha-of-HEAD>
diff-base: <sha-where-status-became-in-progress>
must-violations: 0
should-violations: 3
low-confidence: 2
captured-issues: 1
skipped-passes: []
---

# Review — 042-example-feature

## Summary

<one paragraph: overall posture, count by severity, blocking status>

## MUST violations (blocking)

<one heading per finding; `*None.*` when the section is empty>

## SHOULD violations (advisory)

## Low-confidence findings

## Waived findings

## Captured issues

<one bullet per item appended to specs/inbox.md since diff-base; `*None.*` when empty>

## Skipped passes

<`*None.*` when none>
```

Every empty section renders the literal `*None.*` line — the `write-review` primitive emits it, and the markdown-only path writes the same so the two paths produce byte-identical reports. The **Captured issues** heading carries no suffix.

The **Captured issues** section surfaces issues the agent recorded to
`specs/inbox.md` automatically during the work being reviewed (per
§brownfield-inbox Automatic issue capture). Populate it by diffing
`specs/inbox.md` against `diff-base` (`git diff <diff-base>..HEAD -- specs/inbox.md`)
and listing every line added in that window. These are **informational** —
they are incidental findings parked for `/{project}:groom`, not review
findings against the loaded rules. They do **not** count toward
`must-violations` / `should-violations`, do **not** affect `review.blocking`,
and do **not** change the exit code. The section is the "presented as part of
the review" half of the capture contract: it makes mid-task captures visible at
the gate so none is forgotten. When the inbox shows no additions in the window,
write `captured-issues: 0` and leave the section empty.

Each finding follows this shape:

```markdown
### MUST: <rule-id> — <one-line summary>

- **File**: `path/to/file.ts:42-55`
- **Rule**: <verbatim rule text from the rule file (framework/rules/... or specs/rules/...)>
- **Finding**: <one to three sentences>
- **Auto-fixable**: yes | no
- **Suggested fix**: <code block or prose>
```

The report is regenerated on every `/{project}:review` run — never appended.
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
  low-confidence: 2
  blocking: false
```

`blocking: true` when `must-violations > 0`. This is the field other commands
read. (`write-review` writes `last-run`, `reviewed-against`, `must-violations`,
`should-violations`, `low-confidence`, and `blocking`, plus the `waivers` list
when present.)

## Blocking semantics

A spec MUST NOT advance from `in-progress` to `done` while its frontmatter
records `review.blocking: true`. This is enforced as follows:

1. **`/{project}:implement`** — before marking `status: done`, its `check-review-gate`
   runs three checks in order, first failure wins. First, the feature
   directory's markdown lint; a violation halts before the review block is
   consulted. Then the `review:` block: a missing/null `review.last-run` (or
   absent block) halts with

   ```text
   blocked: spec has not been reviewed — run /{project}:review before completing
   ```

   and only `review.blocking: true` halts with the MUST-violations message plus
   waive guidance:

   ```text
   blocked: spec has N MUST violation(s) — see specs/NNN-feature/review.md
   resolve the violations and re-run /{project}:review, or waive with /{project}:review --waive
   ```

2. **`/{project}:analyze`** — adds a check to its existing audit: if the spec's
   status is `done` but `review.blocking` is `true` or `review.last-run` is
   missing, this is a validation failure. Composable with `--fix`:
   `/{project}:analyze --fix` reverts `done` → `in-progress` and emits a notice
   (it never silently downgrades; the notice is the point).

3. **CI hook** — the shipped GHA template at
   `framework/templates/ci/adopter-generators.yml` fails when any
   spec at `status: done` has `review.blocking: true` or missing
   `review.last-run`. A `done` spec with **no** `review:` block at all is
   grandfathered (it predates `/{project}:review`) and exempt — matching
   `/{project}:analyze`'s own grandfather rule.

This implements the constitution's quality gate via three mutually reinforcing
mechanisms rather than relying on any single one — consistent with the
**Design Principles** rule: never depend on human diligence.

## Blocking message

Emitted by `/{project}:review` when tech-stack alignment fails (missing/empty
`AGENTS.md` `Tech Stack` section, or documented stack inconsistent with
implementation):

```text
blocked: tech-stack alignment failed — AGENTS.md Tech Stack {missing | inconsistent with code in scope}

  expected: <stack inferred from scope, e.g., "TypeScript + React frontend">
  documented: <AGENTS.md Tech Stack contents, or "(empty)">

reconcile AGENTS.md Tech Stack with the implementation, then re-run /{project}:review.
to skip this check on future runs after manual reconciliation, add
[review] tech-stack-verified = true to .govern/config.toml.
```

## Waivers

A MUST violation can be waived only with explicit, recorded justification:

```text
/{project}:review --waive <rule-id> --reason "<text>"
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

Waived findings drop out of the `must-violations` count (there is no separate
`waived-violations` frontmatter field; `write-review` reports the waived count
only in its transient result). They appear in `review.md` under the **Waived
findings** section. They survive across `/{project}:review` runs as long as the
rule ID and file location still match; if either changes, the waiver expires
and the finding re-blocks. Line numbers are not part of the waiver anchor —
the contract is `(rule, file)`, so code moving within the file does not
expire the waiver.

### Per-run waiver processing

On every `/{project}:review` run, after the review passes have produced their
findings (§Run review passes) and before counting them into `must-violations`
or writing `review.md`, walk `review.waivers` and classify each entry against
those findings. A waiver can only be judged against findings that exist — when
an empty scope skips the passes entirely, leave the waivers untouched; and on
a dimension-restricted run, waivers anchored to skipped dimensions apply
unchanged rather than expiring:

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
`ticket`) to enforce org-specific waiver policy in their own CI; `/{project}:review`
and `/{project}:analyze` will not error on unknown fields, and `write-review`
preserves them verbatim on a surviving waiver when it re-renders the block, so
an org policy field is never dropped by a later review run.

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

```text
/{project}:review — 042-example-feature

  security    ✓ 0 MUST   2 SHOULD
  reuse       ✓ 0 MUST   1 SHOULD
  quality     ✓ 0 MUST   0 SHOULD   (2 low-confidence)
  efficiency  ✓ 0 MUST   0 SHOULD
  simplicity  ✓ 0 MUST   0 SHOULD

  captured    1 issue logged during work — run /{project}:groom to route
  blocking: no
  report:   specs/042-example-feature/review.md
```

The `captured` line is omitted when no issues were appended to the inbox in the
review window. It is informational and never affects the exit code.

When MUST violations are present:

```text
/{project}:review — 042-example-feature

  security    ✗ 2 MUST   1 SHOULD
  reuse       ✓ 0 MUST   0 SHOULD
  quality     ✗ 1 MUST   0 SHOULD
  efficiency  ✓ 0 MUST   0 SHOULD
  simplicity  ✓ 0 MUST   0 SHOULD

  blocking: yes — 3 MUST violations
  report:   specs/042-example-feature/review.md

  spec cannot advance to done. Resolve violations and re-run /{project}:review,
  or run /{project}:review --waive <rule-id> --reason "..." for each waivable finding.
```

Exit code: `0` when not blocking, `1` when blocking. Allows CI to gate on the
exit code without parsing the report.

## Idempotency

Re-running `/{project}:review` against an unchanged target reproduces an identical
`review.md` (modulo `reviewed-at` and `reviewed-against`). This is a
derive-don't-ask invariant: review output is a function of code + rules,
never of session state.

## Notes for adopters

- Projects that customize shipped rule files (e.g.,
  `specs/rules/security-backend.md`) pin them in `.govern/config.toml`
  `[pinned] files` to prevent `/govern` from overwriting their additions.
  `/{project}:review` reads whatever is on disk — pinned or not.
- Files inside the rule-file directory (`specs/rules/` in adopter
  projects; `framework/rules/` in govern's own repo) are auto-discovered
  by directory walk (see step 2, `discover-rule-files`). No `AGENTS.md` reference is
  required. Adding a new file at `specs/rules/<domain>-{backend,frontend,cross}.md`
  with a recognized suffix is the only step needed; the suffix selects
  which stacks load it.
- The `AGENTS.md` rule-file reference survives strictly for adopter-local
  rule files placed **outside** `specs/rules/` — e.g.,
  `docs/rules/internal-api.md`. The framework cannot directory-walk
  arbitrary adopter paths, so an explicit `AGENTS.md` reference is the
  discovery signal for these files.
- A rule file with an unrecognized suffix loads for every stack and
  emits a one-line stdout warning (see step 2, `discover-rule-files`). The default
  is "load + warn," never "silent skip." Rename to one of the closed
  suffixes — `-backend.md`, `-frontend.md`, `-cross.md` — to silence
  the warning.
- A rule file can be explicitly excluded from a given project's review
  via `.govern/config.toml` `[[review.disabled-rule-files]]` (see
  [Inputs](#inputs) for the schema and step 2 (`discover-rule-files`) for the
  filter behavior). The override is project-wide and requires a
  mandatory `reason` — the reason is the audit trail. Use this when
  the stack-derived selection is correct (the rule file applies) but
  the team is not yet ready to enforce that file's rules (e.g., an
  internal admin UI that has not adopted full WCAG AA). Waivers
  remain the right tool for individual `(rule, file)` exceptions; the
  opt-out is for whole-file deferrals.
- The five-dimension model is fixed. Domain-specific concerns (accessibility,
  i18n, licensing) belong in additional rule files, not new passes.
