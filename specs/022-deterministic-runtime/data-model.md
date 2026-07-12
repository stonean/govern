# 022 — Deterministic Runtime Data Model

Defines the data structures the runtime owns: the parsed procedure AST, the JSON-over-stdio protocol envelope and message types, the primitive request/response schemas, and the extension-point schemas (the three initial-release points plus the follow-on request/response shapes). These types live in `runtime/src/schema/` as Rust types with `serde::{Serialize, Deserialize}` derives; their serialized JSON shape is the stable contract for host integrators.

## Procedure AST

Produced by `runtime/src/parser/`. Internal to the runtime; not serialized to disk. JSON serialization exists only for `runtime parse <file>`'s debug output.

```rust
struct Procedure {
    command: String,           // e.g., "status"
    steps: Vec<Step>,
}

enum Step {
    Primitive {
        number: StepNumber,    // "1", "1.1", "2", etc.
        name: String,          // matches a primitive name from §The primitive library
        prose: String,         // surrounding prose for the markdown-only/MCP path
        location: SourceRange,
    },
    Extension {
        number: StepNumber,
        identifier: String,    // "writeCode", "assessSpecQuality", "writeSpecBody", ...
        prose: String,
        location: SourceRange,
    },
    Prose {
        number: StepNumber,
        text: String,
        location: SourceRange,
    },
}

struct StepNumber(Vec<u32>);   // [1, 2] for "1.2"; [3] for "3"

struct SourceRange {
    start_line: u32,
    start_col: u32,
    end_line: u32,
    end_col: u32,
}
```

The parser rejects a malformed procedure with a hard `ParseError::Invalid` — an unrecognized primitive name, or a single step naming two or more *distinct* primitives (only one can dispatch, so a second would be silently dropped on the exec path). `runtime parse --check` reports these, and `scripts/lint-procedure-parseability.sh` fails CI on them (for every `framework/commands/*.md` and `framework/bootstrap/*.md` file). A file that has no procedure-shaped Instructions section returns `ParseError::LegacyProse`, tolerated only for the files on the parseability allowlist.

## JSON-over-stdio protocol

Newline-delimited JSON. Each line is one complete JSON object terminated by `\n`. The envelope's `type` field is a discriminated union; the closed set of types below is the entire protocol surface.

### Envelope

```json
{ "type": "<discriminator>" }
```

### Outbound (runtime → host) messages

```json
{
  "type": "llm-request",
  "extension-point": "writeCode | writeSpecBody | assessSpecQuality | performReview | askClarifyQuestion | routeInboxItem",
  "request-id": "<opaque string, unique per request>",
  "request": { }
}
```

```json
{
  "type": "gate-confirm",
  "gate": "<gate name, e.g. plan-finalize-status>",
  "request-id": "<opaque>",
  "prompt": "<user-facing prompt string>"
}
```

```json
{
  "type": "progress",
  "message": "<human-readable string>",
  "step": "<step number, e.g. '3.1'>",
  "primitive": "<primitive name if applicable>"
}
```

```json
{
  "type": "complete",
  "result": { },
  "runtime-version": "<semver>"
}
```

```json
{
  "type": "error",
  "code": "<machine-readable code, e.g. 'parse-error'>",
  "message": "<human-readable description>",
  "runtime-version": "<semver>",
  "location": { "file": "...", "line": 0, "col": 0 }
}
```

### Inbound (host → runtime) messages

```json
{
  "type": "llm-response",
  "request-id": "<matches an open llm-request>",
  "response": { }
}
```

```json
{
  "type": "gate-response",
  "request-id": "<matches an open gate-confirm>",
  "confirmed": true
}
```

The runtime ignores any other inbound JSON shape — it logs to stderr and continues waiting for a valid response.

## Primitive request/response schemas

Each primitive has a typed args struct (the CLI subcommand's `clap` derive shape) and a typed result struct. Below is the canonical JSON shape for each; the CLI surface translates command-line flags into the args; the MCP surface uses the same JSON via `rmcp` tool calls.

### `read-spec` — parse spec frontmatter and body sections

Args:

```json
{ "feature": "022-deterministic-runtime", "include-body": true }
```

Result:

```json
{
  "frontmatter": {
    "status": "clarified",
    "dependencies": ["021-runtime-boundary"],
    "review": { }
  },
  "sections": [
    { "heading": "Motivation", "level": 2, "body": "..." },
    { "heading": "Architecture", "level": 2, "body": "..." }
  ],
  "acceptance-criteria": [
    { "checked": false, "text": "A single binary builds..." }
  ],
  "open-questions": [],
  "path": "specs/022-deterministic-runtime/spec.md"
}
```

### `read-tasks` — parse tasks.md into structured task list

Args:

```json
{ "feature": "022-deterministic-runtime" }
```

Result:

```json
{
  "tasks": [
    {
      "number": "1",
      "heading": "Bootstrap Rust crate",
      "subtasks": [
        { "text": "Create Cargo.toml", "checked": false }
      ],
      "done-when": "cargo build succeeds"
    }
  ],
  "path": "specs/022-deterministic-runtime/tasks.md"
}
```

### `mark-task` — flip checkbox state on a task

Args:

```json
{
  "feature": "022-deterministic-runtime",
  "task-number": "1",
  "subtask-index": 0,
  "checked": true
}
```

Result:

```json
{ "previous": false, "current": true, "path": "specs/.../tasks.md" }
```

### `mark-criterion` — flip checkbox state on an acceptance criterion

Args:

```json
{
  "feature": "022-deterministic-runtime",
  "criterion-index": 3,
  "checked": true
}
```

Result:

```json
{ "previous": false, "current": true, "path": "specs/.../spec.md" }
```

### `set-status` — update spec frontmatter status field

Args:

```json
{
  "feature": "022-deterministic-runtime",
  "from": "clarified",
  "to": "planned"
}
```

Result:

```json
{ "previous": "clarified", "current": "planned", "path": "specs/.../spec.md" }
```

On mismatch (`from` doesn't equal current), returns an `error` envelope with `code: "status-mismatch"` and does not write.

### `derive-boundary` — compute runtime write boundary

Args:

```json
{ "feature": "022-deterministic-runtime" }
```

Result:

```json
{
  "boundary": [
    "specs/022-deterministic-runtime/**",
    "runtime/**",
    "framework/commands/status.md"
  ],
  "first-commit": "<sha>",
  "current-head": "<sha>"
}
```

The boundary is derived from `git diff --name-only <first-commit-on-spec-dir>..HEAD` plus the spec dir itself.

### `check-stuck` — count tasks.md commits since `in-progress`

Args:

```json
{ "feature": "022-deterministic-runtime", "threshold": 10 }
```

Result:

```json
{ "commit-count": 3, "stuck": false, "since-sha": "<sha>", "threshold": 10 }
```

### `validate-frontmatter` — full frontmatter schema check

Args:

```json
{ "path": "specs/022-deterministic-runtime/spec.md" }
```

Result:

```json
{
  "findings": [
    { "severity": "blocking", "field": "status", "message": "..." }
  ],
  "clean": false
}
```

### `resolve-anchor` — verify every `§<anchor>` reference resolves

Args:

```json
{ "path": "framework/constitution.md" }
```

Result:

```json
{
  "references": [
    { "anchor": "runtime-boundary", "line": 459, "resolved": true }
  ],
  "unresolved": []
}
```

### `traverse-deps` — verify spec dependencies and status compatibility

Args:

```json
{ "feature": "022-deterministic-runtime" }
```

Result:

```json
{
  "dependencies": [
    {
      "feature": "021-runtime-boundary",
      "exists": true,
      "status": "done",
      "compatible": true
    }
  ],
  "compatible": true
}
```

### `check-rule-ids` — verify cited rule IDs exist and aren't deprecated

Args:

```json
{ "path": "specs/022-deterministic-runtime/spec.md", "rule-files": ["framework/rules/security-backend.md"] }
```

Result:

```json
{
  "citations": [
    { "rule-id": "SEC-AUTH-001", "found": true, "deprecated": false }
  ],
  "missing": [],
  "deprecated": []
}
```

### `run-generator` — invoke a bash generator in `--dry-run`

Args:

```json
{ "script": "scripts/gen-spec-deps.sh" }
```

Result:

```json
{ "drift": false, "stdout": "...", "stderr": "...", "exit-code": 0 }
```

Non-zero exit code is a drift finding (`drift: true`), not an operational error.

### `lint-markdown` — wrap `npx markdownlint-cli2`

Args:

```json
{ "paths": ["framework/constitution.md", "specs/**"], "fix": false }
```

Result:

```json
{ "violations": [], "clean": true, "exit-code": 0 }
```

### `gate-confirm` — surface a gate to the user through the host

Args:

```json
{ "gate": "plan-finalize-status", "prompt": "Advance status from clarified to planned?" }
```

Result:

```json
{ "confirmed": true }
```

Under the MCP surface, this is the only primitive whose semantics depend on host capability — an MCP host that cannot route a prompt to the user returns `confirmed: false` and the procedure halts at the gate.

### `resolve-feature` — resolve an identifier to a feature directory

Args:

```json
{ "identifier": "22", "scenario": "scaffolding-primitives" }
```

Result (resolved):

```json
{
  "outcome": "resolved",
  "feature": "022-deterministic-runtime",
  "path": "specs/022-deterministic-runtime",
  "status": "in-progress",
  "candidates": [],
  "scenario": {
    "slug": "scaffolding-primitives",
    "path": "specs/022-deterministic-runtime/scenarios/scaffolding-primitives.md",
    "exists": true,
    "section": "Follow-on scenarios"
  }
}
```

Result (ambiguous / not-found):

```json
{ "outcome": "ambiguous", "candidates": ["022-deterministic-runtime", "023-command-runtime"] }
```

Matching order: exact directory name, then feature number (`7` and `007` both match the zero-padded `007-` prefix), then case-insensitive partial slug substring. Ambiguity and no-match are domain outcomes in the result — never operational errors; disambiguation stays with the user through the host. `scenario` is present only when the args named a slug and the outcome is `resolved`; `status` is best-effort (absent when `spec.md` is unreadable). The scenario `section` field falls back to the legacy `spec-ref` frontmatter key.

### `create-feature` — scaffold the next feature directory

Args:

```json
{ "title": "Webhook Delivery" }
```

Result:

```json
{
  "created": true,
  "feature": "043-webhook-delivery",
  "path": "specs/043-webhook-delivery",
  "template": "specs/templates/spec.md"
}
```

The number is `max(existing three-digit prefix) + 1`, zero-padded; the slug is the lowercased title with non-alphanumeric runs collapsed to single hyphens and trimmed. The spec template is resolved in `writeSpecBody`'s candidate order — `{specs-root}/templates/spec.md`, then `framework/templates/spec/spec.md` — and copied atomically with the source file's mode mirrored. An already-existing target directory is the `created: false` domain outcome (`template` absent, nothing written); a missing template is an operational error raised before the directory is created.

### `append-inbox` — append one bullet to the inbox

Args:

```json
{ "text": "security: token logged in plaintext — src/auth.rs (captured during 022)", "dedup-prefix": "security: token logged" }
```

Result:

```json
{ "path": "specs/inbox.md", "created": false, "deduped": false }
```

Appends `- {text}` atomically to `{specs-root}/inbox.md`, creating the file when missing (from `framework/templates/project/inbox.md` when that file exists on disk — the framework source repo — else a bare `# Inbox` heading). With `dedup-prefix` supplied, an existing bullet whose text starts with the prefix (checkbox bullets included) suppresses the write and the result reports `deduped: true`. Embedded newlines in `text` are rejected as an operational error (structure injection), matching `append-task`'s single-line rule.

### `check-artifacts` — deterministic artifact-check families for one feature

Args:

```json
{ "feature": "022-deterministic-runtime" }
```

Result:

```json
{
  "feature": "022-deterministic-runtime",
  "status": "planned",
  "findings": [
    {
      "family": "artifact-completeness",
      "severity": "blocking",
      "message": "plan.md is required at status 'planned' but does not exist",
      "path": "specs/022-deterministic-runtime/plan.md"
    }
  ],
  "clean": false,
  "path": "specs/022-deterministic-runtime/spec.md"
}
```

Four families, mirroring `/gov:analyze`'s markdown-only reference exactly (severity tiers included — the primitive mechanizes the documented policy): `artifact-completeness` (blocking — `plan.md`/`tasks.md` required at `planned`/`in-progress`/`done`; `data-model.md` never required), `task-consistency` (blocking, when `tasks.md` exists — strictly-increasing numbering, `Done when` presence), `scenario-consistency` (advisory — every `scenarios/*.md` has a referencing task, skipped for `done` specs and satisfied by §tasks-phase pruning evidence: zero task sections or non-contiguous numbering), and `review-state-drift` (blocking — a `done` spec with `review.last-run` unset or `review.blocking: true`; a `done` spec with no `review:` block is grandfathered). `--all` iteration stays with the caller. The command-frontmatter-completeness family stays in the markdown-only reference (it reads the host's command directory, which the runtime does not own).

## Extension-point schemas (initial release)

The three initial-release single-shot extension points, plus the follow-on points: `askClarifyQuestion` and `routeInboxItem`, whose typed shapes ship ahead of their scenarios per the extension-request-hygiene scenario, and `verifyCriteria`, which ships with the implement-completion-gate scenario as `/gov:implement`'s criterion-verification seam. Each has request and response payload schemas; the runtime validates incoming responses against these and emits `error: schema-mismatch` on failure. An extension identifier outside this closed set is an `error: unknown-extension` at request-build time — never a raw walker-context dump. In every request that carries legacy-compat context fields after its typed prefix (`writeCode`, `writeSpecBody`, `performReview`), walker-internal accumulator keys (prior `llm:*` response echoes and the accumulated `findings` array) are filtered out; primitive results threaded through the context (`scope`, `diff-base`, `selected`, `rules-dir`, `notices`, …) pass through.

### `assessSpecQuality`

Used by `/gov:analyze`'s per-rule Verification reads.

Request payload:

```json
{
  "spec-path": "specs/022-deterministic-runtime/spec.md",
  "spec-content": "...full spec text...",
  "rule": {
    "id": "QUAL-CLARITY-001",
    "verification": "Acceptance criteria are concrete and testable",
    "severity": "must"
  }
}
```

Response payload:

```json
{
  "passed": false,
  "finding": {
    "severity": "must",
    "rule-id": "QUAL-CLARITY-001",
    "location": { "section": "Acceptance Criteria", "line": 213 },
    "message": "Acceptance criterion 8 ('parses cleanly') is not testable as written..."
  }
}
```

When `passed: true`, `finding` is `null`.

### `writeCode`

Used by `/gov:implement`'s per-task work step.

Request payload:

```json
{
  "task": {
    "number": "3",
    "heading": "Implement read-spec primitive",
    "subtasks": ["..."]
  },
  "plan-relevant-files": [
    { "path": "runtime/src/primitives/read_spec.rs", "content": "..." },
    { "path": "runtime/src/schema/spec.rs", "content": "..." }
  ],
  "write-boundary": [
    "runtime/**",
    "specs/022-deterministic-runtime/**"
  ],
  "constitution-excerpts": ["..."]
}
```

Response payload:

```json
{
  "edits": [
    {
      "path": "runtime/src/primitives/read_spec.rs",
      "action": "create",
      "content": "..."
    },
    {
      "path": "runtime/src/primitives/mod.rs",
      "action": "edit",
      "patch": "..."
    }
  ],
  "summary": "Implemented read-spec primitive..."
}
```

Every edit path must fall within the `write-boundary`; the runtime rejects out-of-boundary edits and surfaces an `error: out-of-boundary-edit` before applying any edit.

### `writeSpecBody`

Used by `/gov:specify` and `/gov:plan` at template-fill moments.

Request payload:

```json
{
  "template-path": "framework/templates/spec/spec.md",
  "template-content": "...",
  "section": "Motivation",
  "feature-description": "...",
  "existing-content": null
}
```

Response payload:

```json
{
  "content": "...filled-in section content...",
  "section": "Motivation"
}
```

When invoked from `/gov:plan` to fill in plan sections, `template-path` points at the plan template and `section` enumerates the plan section to fill.

Field sourcing (extension-request-hygiene):

- `template-path` / `template-content` — resolved from the running command (`/gov:plan` → the plan template, `/gov:specify` → the spec template), trying `{specs-root}/templates/<file>` (the installed adopter layout) then `framework/templates/spec/<file>` (the framework source layout). Both are empty strings when no template exists on disk.
- `section` — the section heading named by the step prose ("Fill the `<name>` section …"); empty when the step fills a whole body rather than one section (`/gov:specify`).
- `feature-description` — the `feature-description` walker-context key, seeded by the host from the slash command's `$ARGUMENTS` (session file or `key=value` exec argument); empty when the host seeds none.
- `existing-content` — the named section's current body from the file the running command owns (`/gov:plan` reads `plan.md`, `/gov:specify` reads `spec.md` — selected by command, never by fallback order); omitted when the file or section is absent or empty.

### `askClarifyQuestion` (follow-on)

Reserved by the [clarify-command-acceleration](scenarios/clarify-command-acceleration.md) scenario; the typed request builder ships ahead of it ([extension-request-hygiene](scenarios/extension-request-hygiene.md)) so the point never falls back to a raw context dump. One host-mediated request/response round trip per open question.

Request payload:

```json
{
  "spec-path": "specs/022-deterministic-runtime/spec.md",
  "spec-content": "...full spec text...",
  "question": {
    "text": "Should retries back off exponentially or linearly?",
    "section": "Open Questions"
  }
}
```

`question.section` is optional and omitted when the walker cannot attribute the question to a section. The question comes from an explicit `question` walker-context value when present, else the first entry of `read-spec`'s merged `open-questions` result.

Response payload:

```json
{ "answer": "Exponential, capped at 60s." }
```

The answer is the user's resolution verbatim; applying it to the spec body remains LLM work per the clarify scenario.

### `routeInboxItem` (follow-on)

Reserved by the [groom-command-acceleration](scenarios/groom-command-acceleration.md) scenario; typed builder ships ahead of it. Kept deliberately minimal: the item under decision, the closed route vocabulary (the groom decision tree's leaves, in walk order), and the specs the router may match — enough to make the routing decision without a walker-context dump.

Request payload:

```json
{
  "item-text": "Bug: retry loop never backs off",
  "routes": ["rule", "spec", "scenario", "chore", "discard"],
  "available-specs": [
    { "feature": "021-webhook-delivery", "status": "done" },
    { "feature": "022-deterministic-runtime", "status": "in-progress" }
  ]
}
```

`item-text` comes from the `item-text` walker-context key (seeded per inbox item by the groom walk); `available-specs` is scanned from the spec root (`NNN-slug` directories, sorted, with each spec's frontmatter `status` — status drives the done → in-progress reopen consent on a scenario route; empty status means the spec file was unreadable).

Response payload:

```json
{
  "route": "scenario",
  "feature": "021-webhook-delivery",
  "reason": "Durable edge case the spec covers at a high level."
}
```

`route` is one of the request's `routes` vocabulary (closed set — anything else is a schema mismatch); `feature` is present when the route targets an existing spec; `reason` is optional prose the host may surface in the per-item confirmation prompt.

### `verifyCriteria` (follow-on)

Introduced by the [implement-completion-gate](scenarios/implement-completion-gate.md) scenario: `/gov:implement`'s completion gate sends one request carrying every acceptance criterion, and the LLM judges each criterion against the implementation — the verification stays semantic while the surrounding tallies and checkbox flips stay mechanical. Each `met: true` verdict drives one `mark-criterion` call; a `met: false` verdict leaves its checkbox unchecked and is reported, never batch-marked.

Request payload:

```json
{
  "spec-path": "specs/022-deterministic-runtime/spec.md",
  "spec-content": "...full spec text...",
  "criteria": [
    { "index": 0, "text": "`runtime exec implement` walks the procedure to completion.", "checked": false },
    { "index": 1, "text": "Out-of-boundary edits are rejected.", "checked": false }
  ]
}
```

`criteria` mirrors `read-spec`'s merged `acceptance-criteria` result in body order; `index` is the 0-based position `mark-criterion` addresses (the two share the same comment/fence-aware section walker, so index N here is the checkbox `mark-criterion` flips at N).

Response payload:

```json
{
  "results": [
    { "index": 0, "met": true },
    { "index": 1, "met": false, "note": "boundary rejection has no covering test yet" }
  ]
}
```

`results` carries one verdict per criterion. `note` is optional prose surfaced in the completion report — a failing criterion's note explains the failure; a missing verdict for a criterion is treated as not met (the gate only flips criteria the response affirmatively confirms).

## Versioning of these schemas

Schemas evolve in lockstep with the runtime binary per §runtime-boundary's lockstep-versioning rule. A breaking schema change increments the runtime's major version. Hosts integrating against the JSON protocol pin a runtime version; mismatches are surfaced by `error` envelopes that carry `runtime-version`, per the resolved Versioning Enforcement question in the spec.

## Notes

- All paths in request/response payloads are repo-relative (use `/` separators on all platforms).
- All timestamps are ISO-8601 UTC strings.
- All commit shas are 40-character lowercase hex strings.
- Unknown JSON fields in incoming envelopes are ignored. Unknown fields in outgoing envelopes are not emitted (forward-compatibility is reserved for future spec evolution, not stowaway fields).
