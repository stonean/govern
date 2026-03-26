# 011 — Brownfield Process Plan

## Overview

Three workstreams: (1) create the `/capture` command and its Claude Code instance, (2) rename `triage` to `inbox` across all governance artifacts, (3) document the brownfield process, scenario promotion, and cross-spec impact patterns in constitution, sdd-context, and README.

## Technical Decisions

### Capture command structure

The `/capture` command follows the same markdown command format as `/specify` and `/scenario`. It reads the session file, creates a numbered spec directory, drafts a skeleton spec from freeform input, and writes the session target — the same mechanics as `/specify` but without the lightweight track qualifying questions and without pressure for comprehensive criteria.

Two files are created: `commands/capture.md` (platform-agnostic template using `{project}` and `{cli-config-dir}` placeholders) and `.claude/commands/gov/capture.md` (Claude Code instance using `gov` and `.claude`). This follows the command file parity rule in CLAUDE.md.

### Inbox rename strategy

The rename is a direct find-and-replace across all files that reference `triage`. The word "triage" is replaced with "inbox" in file names, headings, section markers, command references, and prose. Files are renamed (not copied) to preserve git history.

The rename touches 16 files. To keep the diff reviewable, the rename is done as a single task before other content changes.

### Govern file manifest updates

Both `govern/govern.md` and `govern/govern-auggie.md` reference `triage` in three places each: the file manifest (template source → destination), the slash command manifest, and the post-scaffolding output. All six references update to `inbox`. The new `/capture` command is added to both manifests.

### Constitution additions

Three new sections added to `constitution.md`:

- **Brownfield process** — under the existing brownfield triage section (renamed to brownfield inbox). Documents the capture → incremental growth → promotion lifecycle.
- **Scenario promotion** — under the existing scenarios section. Documents when and how to promote a scenario to its own spec.
- **Cross-spec impact** — new pipeline boundary. Documents that changes land where they belong with a signpost back to the originating spec.

### Cross-spec signpost in 006

A brief note is added to 006-bug-workflow's spec.md indicating that `triage` was renamed to `inbox` by 011. The note does not change 006's status — it is informational only, not a behavioral change.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `commands/capture.md` | Create | Platform-agnostic capture command template |
| `.claude/commands/gov/capture.md` | Create | Claude Code capture command instance |
| `commands/triage.md` | Rename → `commands/inbox.md` | Rename triage command to inbox |
| `.claude/commands/gov/triage.md` | Rename → `.claude/commands/gov/inbox.md` | Rename Claude Code triage instance |
| `templates/triage.md` | Rename → `templates/inbox.md` | Rename triage template |
| `commands/inbox.md` | Modify | Update heading, references, and content from triage to inbox |
| `.claude/commands/gov/inbox.md` | Modify | Update heading, references, and content from triage to inbox |
| `templates/inbox.md` | Modify | Update heading and content from triage to inbox |
| `govern/govern.md` | Modify | Update file manifest, command manifest, post-scaffolding output; add capture command |
| `govern/govern-auggie.md` | Modify | Same updates as govern.md for Auggie paths |
| `constitution.md` | Modify | Rename triage section to inbox; add brownfield process, scenario promotion, cross-spec impact |
| `sdd-context.md` | Modify | Rename triage to inbox; add capture command, scenario promotion, cross-spec impact |
| `README.md` | Modify | Rename triage to inbox; update brownfield section; add capture to slash commands table |
| `commands/about.md` | Modify | Update triage references to inbox |
| `.claude/commands/gov/about.md` | Modify | Update triage references to inbox |
| `AGENTS.md` | Modify | Update triage references to inbox |
| `specs/006-bug-workflow/spec.md` | Modify | Add signpost noting triage → inbox rename by 011 |

Not modified (historical specs — self-contained at time of writing):

- `specs/006-bug-workflow/plan.md`
- `specs/006-bug-workflow/tasks.md`
- `specs/007-adopt-workflow/spec.md`
- `specs/007-adopt-workflow/plan.md`

## Open Questions Resolved

All open questions were resolved during the clarify phase. No new questions surfaced during planning.
