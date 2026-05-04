---
title: "007-govern-workflow — scenario: govern-self-update-precheck"
spec-ref: "007-govern-workflow — Distribution Model"
tags: []
---

# Govern Self-Update Pre-Check

> **Signpost:** the abort path described in the Behavior section below was incomplete — it explicitly stated "No files were modified by this run", which meant the freshly fetched upstream `govern.md` never landed on disk. Adopters who hit the abort were stuck in a loop: the next session loaded the same stale installed copy, detected stale again, and aborted again, with no path forward except a manual `curl`. The pre-check has since been redesigned: it (a) moved earlier in the flow, ahead of the archive fetch and pre-run migrations; (b) does a small `curl` for `framework/bootstrap/govern.md` only (not the full repo archive); (c) **writes the freshly fetched copy to `{config_dir}/commands/govern.md` for every stale agent before aborting**, so the next session loads the up-to-date instructions and the run completes on the first retry. The full archive is fetched only after the self-update check passes (no stale agents). See `framework/bootstrap/govern.md` → `## govern.md Self-Update Check` for the current behavior. The body below is preserved as the original (pre-fix) design per the constitution's frozen-archaeology rule.

## Context

`/govern`'s instructions are loaded into the running session at invocation time. When the upstream `framework/bootstrap/govern.md` has changed since the installed copy was written, the running command is following stale logic — its manifest, audit, migration, and scaffolding rules are whatever shipped with the installed file, not what's on `main`.

The current behavior runs the entire pipeline with that stale logic, then prints a self-update notice in the post-scaffolding output asking the user to start a new session and re-run. By that point the work has already been done with the wrong rules: a manifest entry that the new govern would have skipped was written, a migration the new govern handles differently was applied, an audit the new govern tightened was run with the loose rules. The user re-runs and the new logic re-does (or fails to undo) what the old logic did.

Swapping to the new instructions mid-run is not feasible — slash commands cannot reload their own prompt. Detecting the divergence early and aborting before any destructive work is the only correct path: the user re-runs in a new session, which loads the freshly written `govern.md`, and the second run does the right thing on the first try.

## Behavior

- The existing govern run order is reordered so that **File Fetching → Archive fetch and extract** runs before **Frontmatter Migration** (today the order is the reverse). The new sequence: Pre-flight Checks → Agent Selection → Permission Setup → Project Configuration → Archive fetch and extract → **Self-update pre-check** → Frontmatter Migration → Shared Files → Per-Agent Scaffolding → Security Audit → Post-Scaffolding Output. Archive fetch has no dependency on migration and migration has no dependency on archive contents, so the swap is mechanical.
- After **Archive fetch and extract** completes and before any other manifest pass (no shared files written, no per-agent scaffolding, no security audit, no frontmatter migration), `/govern` runs a self-update pre-check.
- For each selected agent, compare the extracted `{tempdir}/govern-main/framework/bootstrap/govern.md` against the installed `{config_dir}/commands/govern.md`:
  - If the installed file does not exist (first run for this agent), record "no installed copy" and continue — nothing to diverge from.
  - Byte-compare the two files. Identical → record "current". Different → record "stale" if the file is not pinned, or "pinned-divergent" if `{config_dir}/commands/govern.md` is listed in `.governance.toml` `pinned.files`. A pinned file that matches upstream is recorded as "current" (the pin had nothing to suppress this run).
- "pinned-divergent" never triggers the abort — pinning is an opt-out from automatic updates. It produces a single advisory line in the post-scaffolding output: `{agent}: govern.md pinned, upstream has changed`. The line appears only on runs where the pinned file actually differs from upstream; it stays silent on runs where the pinned version happens to match.
- If any selected agent is recorded as "stale", abort the run before any further work. Print:

  > **The govern command itself has updated.** Your installed copy is behind upstream and the running session is using the older instructions. Start a new session and re-run `/govern` to pick up the latest version.
  >
  > Stale agents: {comma-separated names}.
  >
  > No files were modified by this run.

- The abort happens after **Permission Setup** (already applied, additive, harmless) and after **Archive fetch** (the work needed to detect the divergence in the first place). Everything past that point — Frontmatter Migration, the Shared Files manifest, Per-Agent Scaffolding, Security Audit, and Post-Scaffolding Output — is skipped.
- On the next run in a new session, the freshly installed `govern.md` is what loads; the pre-check sees "current" for every agent and the run proceeds normally.
- The end-of-run **Self-update notice** in **Post-Scaffolding Output** becomes unreachable in practice (the abort fires before scaffolding ever updates the file). It is removed; the pre-check abort replaces it.

## Edge Cases

- **First-ever run** — no installed `govern.md` exists for any agent, so every agent is "no installed copy" and the run proceeds. The first run scaffolds the file; subsequent runs gain the comparison anchor.
- **Adding a new agent on a re-run (`--add-agent`)** — the new agent has no installed file (record "no installed copy"); the existing agent's installed file is compared. If the existing agent is stale, abort — the new-agent install rides along with the re-run after the user starts a new session.
- **All selected agents pinned, all matching upstream** — every agent records "current"; no advisory fires; the run proceeds normally. The pin had nothing to suppress this run.
- **All selected agents pinned, at least one divergent** — divergent agents record "pinned-divergent"; the run proceeds (no abort); the post-scaffolding output includes one advisory line per divergent agent. The adopter has explicitly opted out of upstream govern updates and is being told the pin is currently active.
- **Mixed result across selected agents** — any single "stale" triggers the abort; the message lists every stale agent so the user understands the scope before re-running.
- **`govern.md` byte-identical despite an upstream commit** — the comparison is content-based, not version-based. A commit that doesn't change the bootstrap file's bytes (e.g., reformatting unrelated framework files) is correctly recorded as "current" and the run proceeds.
- **Archive fetch failed** — the existing abort path in **File Fetching → Archive fetch and extract** fires first; the self-update pre-check never runs. No regression.

## Open Questions

*All open questions resolved. See Resolved Questions below.*

## Resolved Questions

- **Scope of the comparison** — selected agents only. The pre-check diffs only the agents this run will write to; agents that exist in the project but are not in the run's selection are not checked. Reason: the abort exists to prevent destructive work being done with stale rules, and destructive work is per-agent. An unselected stale agent does not affect this run's correctness — it will trip the check on its very next `/govern` run targeting it, so drift is still detected, just lazily. This keeps the abort message tightly scoped to what the user is actually doing rather than reporting drift in agents they did not ask to touch this run.
- **Pinned `govern.md` semantics** — pinning suppresses the abort but not the information. A pinned file is still byte-compared to upstream every run: matching → recorded as "current" with no output (the pin had nothing to suppress); divergent → recorded as "pinned-divergent", the run proceeds normally (no abort), and the post-scaffolding output includes one advisory line per divergent agent: `{agent}: govern.md pinned, upstream has changed`. Reason: the user pinned the file to opt out of automatic updates, not to opt out of knowing when their frozen version has actually drifted from upstream — that is the moment they would want to either review the upstream changes and unpin, or consciously confirm they are staying on the old version. The advisory is silent on runs where the pinned version happens to match upstream, so adopters who are deliberately and indefinitely on an old version see no recurring nag.
- **Aborting after frontmatter migration** — pre-check moves ahead of migration. The govern run order is reordered so that Archive fetch and the self-update pre-check run before Frontmatter Migration. If the pre-check aborts, the working tree is genuinely untouched and the abort message's "No files were modified by this run" is literally true. Reason: migration is exactly the kind of work the new govern might do differently (new fields, tightened patterns, removed migrations); leaving migration changes from the old govern on disk after a stale-abort either commits the user to old migration semantics (if they `git add` and move on) or forces a `git restore` before re-running — both surprises that defeat the pre-check's purpose. Reordering is mechanical: archive fetch has no dependency on migration and vice versa. The "all 'did anything happen?' answers in one place" code-organization argument loses to the user's working tree staying clean.
