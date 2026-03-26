# 011 — Brownfield Process Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Rename triage to inbox

Rename files and update all references from `triage` to `inbox` across the framework. This is a mechanical rename done first so subsequent tasks work with the final naming.

- [x] Rename `commands/triage.md` → `commands/inbox.md`
- [x] Rename `.claude/commands/gov/triage.md` → `.claude/commands/gov/inbox.md`
- [x] Rename `templates/triage.md` → `templates/inbox.md`
- [x] Update content in `commands/inbox.md` (heading, references, prose)
- [x] Update content in `.claude/commands/gov/inbox.md` (heading, references, prose)
- [x] Update content in `templates/inbox.md` (heading, content)
- [x] Update `govern/govern.md` file manifest, command manifest, and post-scaffolding output
- [x] Update `govern/govern-auggie.md` file manifest, command manifest, and post-scaffolding output
- [x] Update `constitution.md` section heading, marker, and content
- [x] Update `sdd-context.md` references
- [x] Update `README.md` references
- [x] Update `commands/about.md` references
- [x] Update `.claude/commands/gov/about.md` references
- [x] Update `AGENTS.md` references
- [x] Add signpost to `specs/006-bug-workflow/spec.md` noting the rename
- [x] Run `markdownlint-cli2` on all modified files

**Done when:** no file in the repository contains `triage` except in 006's historical spec content and the signpost note, and 011's own spec references.

## 2. Create capture command

Create the `/capture` command in both platform-agnostic and Claude Code forms.

- [ ] Create `commands/capture.md` with freeform input flow, skeleton spec creation, session target update, and post-capture options
- [ ] Create `.claude/commands/gov/capture.md` as Claude Code instance with `/gov:` prefix and `.claude` paths
- [ ] Verify command file parity between the two files
- [ ] Run `markdownlint-cli2` on both files

**Done when:** both capture command files exist, pass lint, and follow the same structure as other commands.

## 3. Update govern file manifests and add migration

Add the capture command to the govern file manifests and add a triage → inbox migration step.

- [ ] Add `commands/capture.md` to `govern/govern.md` slash command manifest with `update` strategy
- [ ] Add `commands/capture.md` to `govern/govern-auggie.md` slash command manifest with `update` strategy
- [ ] Add triage → inbox migration to `govern/govern.md`: rename `specs/triage.md` to `specs/inbox.md` if needed, merge if both exist, delete old triage command
- [ ] Add triage → inbox migration to `govern/govern-auggie.md`: same migration with Auggie paths
- [ ] Migration is reported in post-scaffolding summary
- [ ] Add signpost to `specs/007-govern-workflow/spec.md` noting the govern command changes by this spec
- [ ] Run `markdownlint-cli2` on both govern files and 007 spec

**Done when:** both govern files include the capture command in their manifests and perform the triage → inbox migration for previously adopted projects.

## 4. Document brownfield process in constitution

Add the brownfield process, scenario promotion, and cross-spec impact patterns to `constitution.md`.

- [ ] Add brownfield process section under the existing brownfield inbox section — documents capture → incremental growth → promotion lifecycle
- [ ] Add scenario promotion subsection under the existing scenarios section — documents indicators and the promotion pattern
- [ ] Add cross-spec impact as a pipeline boundary — documents that changes land where they belong with signpost references
- [ ] Run `markdownlint-cli2` on `constitution.md`

**Done when:** constitution documents all three patterns and passes lint.

## 5. Update sdd-context and README

Update documentation to reflect the brownfield process.

- [ ] Add capture command to `sdd-context.md` slash commands table
- [ ] Add brownfield process section to `sdd-context.md` (capture, incremental growth, scenario promotion)
- [ ] Add cross-spec impact to `sdd-context.md`
- [ ] Update `README.md` slash commands table — add `/capture`, rename `/triage` to `/inbox`
- [ ] Update `README.md` brownfield section to reference the process
- [ ] Run `markdownlint-cli2` on both files

**Done when:** both files reflect the brownfield process, capture command, inbox rename, and pass lint.
