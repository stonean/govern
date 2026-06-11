# 029 — Bootstrap Runtime Auto-Detect and Wire Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Add the Pre-flight Phase to `govern.md`

- [x] Generalize the §govern.md Self-Update Check section into a **Pre-flight Phase** positioned after the §Permission Setup seed and before §Pre-run Migrations / the archive fetch.
- [x] Describe the phase running both checks (gvrn detection + self-update), accumulating restart-requiring writes, and emitting a single combined abort if anything was written.
- [x] Preserve the existing self-update byte-compare / stale-write behavior inside the phase.
- Done when: govern.md describes one pre-flight phase whose worst case is a single restart, and the prior standalone self-update section no longer exists as a sibling.

## 2. Add the three-state detection mechanism

- [x] Document **State A** = tool-inventory introspection (`mcp__gvrn__*` / `mcp:gvrn:*`, lazy names count as present); no shell, no permission.
- [x] Document **State B** = binary probe succeeds, no tools → wire + grant perms + contribute to the pre-flight abort (names every file written).
- [x] Document **State C** = probe fails *or* cannot run *or* is denied → markdown path + one tip line pointing at README Runtime.
- Done when: all three states and the "deny/unavailable ⇒ State C" degradation are specified in govern.md.

## 3. Add the MCP Wiring subsection

- [x] Specify the per-layout target (`.mcp.json` / `{config_dir}/mcp_config.json`) and the `gvrn` entry shape.
- [x] Specify the additive/idempotent in-place merge rules: missing file, missing `gvrn`, existing `gvrn` (no-op), missing `mcpServers` key, malformed JSON (skip + warn + degrade).
- [x] State explicitly that the write updates and never replaces/truncates the file.
- Done when: govern.md's MCP Wiring subsection covers all five merge cases.

## 4. Update §Permission Setup and reverse the "not scaffolded" decision

- [x] Add the binary-probe permission to the always-applied seed description.
- [x] Describe the State-B gvrn tool-permission grant (per-layout wildcard, additive).
- [x] Rewrite the paragraph that says the runtime is "wired separately and not scaffolded" into the auto-wire-on-detect behavior.
- [x] Confirm the §Procedural-fidelity allowed-prompts list is unchanged (no new prompt).
- Done when: the "not scaffolded" language is gone and no new confirmation prompt was added.

## 5. Add the probe to the Agent Registry `settings_template` seeds

- [x] Claude row: add `Bash(command -v *)`.
- [x] Auggie row: add `{ "toolName": "launch-process", "shellInputRegex": "^command -v ", "permission": { "type": "allow" } }`.
- [x] Antigravity row: add `command(command -v)` (or the validated fallback) and keep it identical to the configure file's form.
- Done when: all three registry seeds grant the probe.

## 6. Add the probe to the three configure files

- [x] `configure/claude.md`: add `Bash(command -v *)` to the canonical allow set.
- [x] `configure/auggie.md`: add the `"^command -v "` `toolPermissions` entry.
- [x] `configure/antigravity.md`: add the probe to `permissions.allow`, matching the registry-seed form; resolve the token-prefix grammar (`command(command -v)` vs `command(which)`).
- Done when: each configure file grants the probe in the same form as its registry seed.

## 7. Reframe the README Runtime section

- [x] Change the framing from "manual MCP wiring" to "`/govern` auto-wires gvrn when the binary is detected."
- [x] Keep the binary download/install instructions (the binary is still installed out of band).
- Done when: README states auto-wiring and no longer instructs the user to hand-edit the MCP config.

## 8. Add new edge cases and the State-C tip to govern.md output

- [x] §Edge Cases: malformed wiring file (no clobber), denied/unavailable probe ⇒ State C, compounding restart collapsed to one.
- [x] §Post-Scaffolding Output: the State-C tip line; ensure the State-B abort message lists every file written.
- Done when: the new edge cases and output lines are present.

## 9. Run the parity audit (new family deferred — Option B)

- [x] Investigate the existing audit families for a seed↔configure permission-parity home: none fits (Family 14 is install.sh↔registry; Family 3 is the workflow registry; Family 1 is pipeline-status wording). Log a follow-up to `specs/inbox.md` to add a dedicated `runtime-probe-parity.sh` family as its own spec rather than expand the audit surface mid-feature.
- [x] Run `scripts/audit/run-all.sh` and confirm no regressions from tasks 1–8.
- Done when: the audit passes clean with no regressions, and the deferred seed/configure parity family is captured as an inbox follow-up.

## 10. Verification pass

- [x] `npx markdownlint-cli2` clean on the changed markdown (govern.md, configure files, README, this spec dir).
- [x] Manually walk each state's prose to confirm coherence: State A (silent deterministic), State B (wire + single abort + file list), State C (markdown + tip), and the merged stale-`govern.md` + unwired case (one restart).
- [x] Confirm `gvrn`-absent CI (`markdown-only-pipeline.yml`) reasoning still holds — the new probe is a detection step, not a runtime dependency.
- Done when: lint is clean and all state walk-throughs are coherent.
