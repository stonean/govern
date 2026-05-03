# 016 — Cross-Cutting Rules Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Add §rules section to the constitution

- [x] Edit `framework/constitution.md` to insert a new `<!-- §rules -->` section between §scenario-promotion and §brownfield-inbox.
- [x] Section content: definition, conceptual format summary (ID, Statement, Rationale, Verification, RFC 2119) with back-link to `specs/008-security-rules/data-model.md`, four-indicator promotion checklist (cross-cutting / citable / governance-recognized category / generalizable wording), when-not-to-write (situational → scenario; feature-wide → AC), lifecycle (ID stability, deprecation), three-tier table (rule / AC / scenario by scope).
- [x] Add a new row to the canonical-sources table in §drift-prevention: "Rules artifact definition" → `framework/constitution.md` §rules.
- [x] **Done when:** the constitution renders cleanly under `npx markdownlint-cli2`, the §rules anchor is reachable, and the canonical-sources table includes the new row.

## 2. Update §bug-handling decision tree to include rule promotion

- [x] Edit `framework/constitution.md` §bug-handling. Rewrite the intro to acknowledge the three-tier framing (rule / spec / scenario chosen by scope) before listing the decision tree.
- [x] Reorder the decision tree to four steps: (1) no rule covers this cross-cutting concern → promote to rule; (2) no spec exists; (3) spec is ambiguous; (4) spec is clear, implementation wrong.
- [x] Preserve the closing line: "In all four cases, the spec or rule becomes more precise."
- [x] **Done when:** the §bug-handling section reads as a four-step tree with the rule check first, and `npx markdownlint-cli2` passes.

## 3. Add optional "Applicable Rules" section to the spec template

- [x] Edit `framework/templates/spec/spec.md` to insert `## Applicable Rules` between `## Acceptance Criteria` and `## Open Questions`.
- [x] Body is an HTML comment explaining when to cite rule IDs (e.g., `BE-AUTHN-001`), with one or two example IDs and a note that the section can be deleted if no rules apply.
- [x] **Done when:** the template renders under `npx markdownlint-cli2` and a freshly-copied template still produces a passing spec without manual cleanup beyond the existing comment-prompts.

## 4. Generalize validate.md from "Security rules" to "Rules"

- [x] Edit `framework/commands/validate.md`. Rename the `### Security rules (blocking and advisory)` heading to `### Rules (blocking and advisory)`.
- [x] Rewrite the loading prose to be parameterized: "Load each rule file in the rule-file list (currently `specs/security-backend.md` and `specs/security-frontend.md`) if present in the project. Each file is independently optional."
- [x] Replace remaining security-specific phrasing in the section (e.g., "security rule file," "security checks") with generic equivalents ("rule file," "rule checks") where the meaning is unchanged. Keep the security examples in skip messages where they serve as concrete illustration.
- [x] **Done when:** the section reads as generic rule loading without security-specific framing, the loading prose is parameterized, and `npx markdownlint-cli2` passes.

## 5. Update groom.md decision-tree walk to include rule promotion

- [x] Edit `framework/commands/groom.md`. In the **Groom each item** section, insert a new Step 1 before the existing "Does a spec exist for this behavior?" check.
- [x] New Step 1 text: apply the four-indicator promotion checklist (per `framework/constitution.md` §rules); if the item qualifies as a cross-cutting concern with no covering rule, recommend promoting to a rule. If a rule file covers the domain, the user amends it; if no rule file covers the domain, note that creating a new rule file is its own spec (out of groom's scope) and capture that signal back into the inbox item.
- [x] Renumber existing Steps 1–3 to Steps 2–4. Update the section reference list at the top of the command (`Reference: §bug-handling, §scenarios, §brownfield-inbox`) to include `§rules`.
- [x] **Done when:** the groom decision-tree walk is a four-step sequence with rule-promotion first and `npx markdownlint-cli2` passes.

## 6. Add signpost to 008-security-rules/spec.md [simple]

- [x] Edit `specs/008-security-rules/spec.md`. Insert a quoted "Signpost" note between the YAML frontmatter and the `# 008 — Security Rules` H1, of the form documented in the plan (008 is the security instance of the general rules tier defined in 016; rule-file format defined here remains canonical; see [§rules](../../framework/constitution.md) for general framing).
- [x] Body is not modified.
- [x] **Done when:** the signpost appears at the top of 008's spec.md and `npx markdownlint-cli2` passes on the file.

## 7. Regenerate Claude command mirrors [simple]

- [x] Run `./scripts/gen-claude-commands.sh` from the repo root.
- [x] Verify `.claude/commands/gov/validate.md` and `.claude/commands/gov/groom.md` reflect the source edits (renamed sections, new decision-tree step, parameterized loading prose).
- [x] **Done when:** the generator runs without errors and the two generated files are in sync with their `framework/commands/` sources.

## 8. Lint and validate end-to-end [simple]

- [ ] Run `npx markdownlint-cli2` against all modified files (constitution, validate.md, groom.md, spec template, 008 spec, 016 plan/tasks).
- [ ] Run `/gov:validate --all` against the govern repo. Confirm no new findings introduced by the spec/template/validate changes themselves (specifically: no new "unknown rule reference" findings on existing specs; no template-rule-alignment findings against the new "Applicable Rules" section).
- [ ] **Done when:** markdownlint reports zero errors across all modified files and `/gov:validate --all` passes (or its findings are pre-existing and unrelated to 016).
