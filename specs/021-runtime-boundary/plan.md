# 021 — Runtime Boundary Plan

Implements [021 — Runtime Boundary](spec.md).

## Overview

Three distinct deliverables:

1. **Constitutional amendment** to `framework/constitution.md` — three edits: the opening paragraph of §text-first-artifacts, a new §runtime-boundary subsection at the end of §text-first-artifacts, and a row added to the §drift-prevention canonical sources table.
2. **CI workflow** — a new GitHub Actions workflow that asserts the five deterministic checks from the spec's opt-in invariant. The workflow runs on every PR matching the path filter and is the tripwire that catches a future PR introducing a silent runtime dependency.
3. **Runtime-tools manifest + fallback lint script** — a small bash/grep script under `scripts/` plus a manifest file (initially empty) under `framework/`. The manifest enumerates runtime tools by canonical name; the lint scans `framework/commands/*.md` for references to any tool in the manifest and verifies each is paired with a graceful-fallback marker.

The amendment text is fully drafted in the spec body; this plan's job is to specify *where* in the constitution it lands, *how* the CI workflow is structured, and *how* the fallback lint operates concretely.

## Technical Decisions

### Constitution source-of-truth is `framework/constitution.md`

`framework/constitution.md` is the canonical constitution. The root `constitution.md` referenced in `AGENTS.md` ("sync target") does not exist in this repo — it describes the relationship in *adopting* projects, where `framework/constitution.md` ships as the root file. All three amendment edits land in `framework/constitution.md`. No second file to sync.

### Three discrete edits, ordered

The amendment is three edits, performed in this order so that each commit (or each diff in a single commit) leaves the document well-formed:

1. **Opening paragraph of §text-first-artifacts** — replace the "no bootstrap tooling beyond the AI agent" clause with the two-clause version that distinguishes the markdown framework (standalone) from the optional runtime (opt-in, with a forward reference to §runtime-boundary).
2. **New §runtime-boundary subsection** — append at the end of §text-first-artifacts, after the existing "Validation Severity" subsection. Body composes the five principles (RFC 2119 MUST/MUST NOT), three eligibility criteria, opt-in invariant, versioning rule, non-scope (RFC 2119 MUST NOT), and the one-line forward pointer to spec 022. Anchor marker `<!-- §runtime-boundary -->` placed above the subsection heading per existing convention.
3. **Row added to §drift-prevention canonical sources table** — single new row whose Fact column reads "Runtime contract / boundary" and whose Canonical-source column points at `framework/constitution.md` §runtime-boundary.

The opening-paragraph edit is performed first so that the forward reference to §runtime-boundary, which it introduces, resolves to a section that exists by the time the subsection is added in step 2. In a single PR this ordering is academic; if the work were split across commits, this is the safe order.

### CI workflow lives at `.github/workflows/markdown-only-pipeline.yml`

Filename matches the workflow's job: prove the markdown-only path still works. The workflow has a single job `markdown-only` with five steps mapping 1:1 to the spec's checks (a)–(e). Each step is a thin shell invocation:

| Step | Implementation |
| --- | --- |
| (a) runtime binary absent | `command -v <name> && exit 1 \|\| true` for each name in `framework/runtime-tools.txt` |
| (b) bash generators clean | `bash scripts/gen-spec-deps.sh --dry-run && bash scripts/gen-readme-table.sh --dry-run && bash scripts/gen-help-tables.sh --dry-run` |
| (c) markdownlint | `npx markdownlint-cli2` |
| (d) fallback lint | `bash scripts/lint-runtime-fallback.sh` |
| (e) frontmatter integrity | `bash scripts/lint-frontmatter.sh` |

Workflow trigger is `pull_request` with `paths` filter `framework/**`, `specs/**`, `.claude/commands/**`. Push to `main` also triggers the workflow to catch direct commits.

No matrix, no caching, no Rust/Go toolchain — the entire job runs in plain Ubuntu with bash and `npx`. This is the markdown-only assertion in practice.

### Runtime-tools manifest is a flat text file, initially empty

`framework/runtime-tools.txt` lists canonical runtime tool names, one per line, comments allowed with `#`. The file ships **empty** in this spec — only a comment header explaining its purpose and link to §runtime-boundary. Spec 022 adds the first real entries. This separation keeps 021 strictly constitutional and 022 strictly capability-introducing.

The manifest is plain text rather than YAML/JSON because (1) the consumers are shell scripts, (2) it's a flat list, (3) editing it is unambiguous in PRs.

### Fallback lint operates by proximity scan

`scripts/lint-runtime-fallback.sh` is a bash script that:

1. Reads `framework/runtime-tools.txt` and builds a list of tool names (one per line, ignoring blank lines and `#` comments).
2. Greps each tool name across `framework/commands/*.md` (including any deferred-tool MCP names — full string match, not regex).
3. For every match, scans forward up to 20 lines for a fallback marker — case-insensitive matches against the literal tokens `Otherwise`, `Fallback`, `If unavailable`, `markdown-only path`. If none found within the window, emit the file path, line number, and tool name as an error.
4. Exits non-zero if any error was emitted.

20-line window is a heuristic; revisit when real tools are added in spec 022 and the heuristic can be validated against actual usage. The lint is purely structural — it does not validate that the fallback is *correct*, only that one is present. Correctness is reviewer responsibility.

This trades false-positive risk (a fallback that exists but doesn't match the keywords) for a fully derived check (no author-supplied markers required). False positives are easy to resolve by paraphrasing into one of the four accepted tokens.

### Frontmatter integrity lint is also bash, not Python or yq

`scripts/lint-frontmatter.sh` extracts the frontmatter block from each `specs/**/spec.md`, `specs/**/spec-and-plan.md`, and `specs/**/scenarios/*.md`, then verifies via grep/awk:

- A `---` delimited block exists at the top of the file.
- The `status:` field is one of `draft`, `clarified`, `planned`, `in-progress`, `done` (when present; scenario files don't have status).
- The `dependencies:` field is present (when applicable) and parses as a bracketed list — verified by checking the line starts with `dependencies: [` and ends with `]`, or starts with `dependencies:` followed by a YAML block list on subsequent lines.

This is intentionally less rigorous than a real YAML parser would be — it catches the *shape* errors `/gov:validate`'s hard-fail tier would also flag, but cheaply. If false negatives become an issue in practice, the natural upgrade path is to swap the bash check for `yq` in CI; the lint's contract stays the same.

### Anchor placement convention

The constitution uses `<!-- §<anchor> -->` markers on the line *immediately before* the section heading they anchor. The new `<!-- §runtime-boundary -->` marker follows this convention, placed directly above the `### Runtime Boundary` heading.

### No new data model

The amendment introduces no domain entities. The runtime-tools manifest is a flat list, not a structured schema. No `data-model.md` is created for this spec.

### Order of work

The tasks order is: (1) constitution amendment → (2) runtime-tools manifest stub → (3) fallback lint script → (4) frontmatter lint script → (5) workflow file → (6) `/gov:validate` against this spec → (7) markdownlint pass. The workflow lands last so it can be exercised against the completed amendment and lint scripts in the same PR.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/constitution.md` | Edit | Three amendments: §text-first-artifacts opening paragraph; new §runtime-boundary subsection; new row in §drift-prevention canonical sources table |
| `framework/runtime-tools.txt` | Create | Empty (comment-only) manifest of canonical runtime tool names; populated by spec 022 |
| `scripts/lint-runtime-fallback.sh` | Create | Proximity-scan bash script verifying every runtime-tool reference in `framework/commands/*.md` has a fallback marker within 20 lines |
| `scripts/lint-frontmatter.sh` | Create | Bash script verifying frontmatter shape (delimiters, status enum, dependencies field presence/format) across spec and scenario files |
| `.github/workflows/markdown-only-pipeline.yml` | Create | CI workflow running the five deterministic checks on every PR matching the path filter |
| `specs/021-runtime-boundary/plan.md` | Create | This file |
| `specs/021-runtime-boundary/tasks.md` | Create | Task breakdown derived from this plan |

## Trade-offs

### Considered and rejected

- **Storing the runtime-tools manifest as YAML or JSON** — rejected. Consumers are shell scripts; a flat text file is unambiguous in PRs and removes a parsing dependency from CI. Upgrade path remains open if structured metadata is ever needed.
- **Implementing the fallback lint as a structured-marker check (e.g., `<!-- runtime-tool: name --> ... <!-- /runtime-tool -->`)** — rejected. Structured markers depend on author discipline, exactly the anti-pattern the Design Principles section in `AGENTS.md` forbids. The proximity scan is derived from existing command prose and requires no author markup.
- **Running the fallback lint as a Rust binary** — rejected for spec 021. The runtime itself is deferred to 022; introducing a binary here contradicts the spec's "no binary in this spec" non-goal. If 022 lands a Rust runtime, the lint can be subsumed into it as a runtime-eligible capability.
- **Splitting the amendment into three separate PRs** — rejected. The three edits are interdependent (the opening-paragraph forward reference, the subsection it points to, and the canonical sources row pointing back at the subsection). They land together or not at all.
- **Triggering the workflow on every PR regardless of path** — rejected. Docs-only or release-only PRs cannot introduce a silent runtime dependency; running the workflow on them wastes runner minutes without strengthening the invariant.
- **Hand-authored "runtime-eligibility" labels on slash commands** — rejected during clarify (Q6). Resolution recorded in spec.

### Known limitations

- The fallback lint's 20-line proximity window is a heuristic. False positives occur when a real fallback uses synonyms outside the accepted token list (`Otherwise`, `Fallback`, `If unavailable`, `markdown-only path`). Resolution is paraphrasing — author cost is low. The window can be tuned when spec 022 introduces real tool references and the heuristic is empirically tested.
- The frontmatter lint is shape-only, not a real YAML parser. Adversarial frontmatter (deeply nested, multi-line strings with `---` inside) can defeat it. `/gov:validate`'s hard-fail tier remains the rigorous check; this lint is a CI-side smoke test.
- The CI workflow asserts the runtime binary is absent, but only knows binary names from `framework/runtime-tools.txt`. A binary using a different name than is in the manifest could slip past. Mitigated by treating the manifest as the canonical list spec 022 must populate; deviations are caught at PR review of 022.
- The workflow does not exercise LLM-driven slash commands. By design — see spec's Q2 resolution. A separate scheduled smoke-test job exercising the LLM path is out of scope for 021.
