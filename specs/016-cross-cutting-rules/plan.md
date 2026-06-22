---
title: "016-cross-cutting-rules — plan"
---

# 016 — Cross-Cutting Rules Plan

Implements [016 — Cross-Cutting Rules](spec.md).

## Overview

016 promotes rules to a first-class artifact tier without changing rule semantics, validate enforcement logic, or the rule-file format. The work is concentrated in five files: the constitution gains a §rules section and a fourth decision-tree route; the spec template gains an optional "Applicable Rules" prompt; `analyze.md` and `groom.md` are reframed from security-specific to general; and 008's spec gets a top-of-file signpost. After source edits, the generated `.claude/commands/gov/*.md` mirrors are regenerated.

The governing constraint is **canonical-source discipline**: the constitution carries the *concept* of rules (definition, lifecycle, promotion checklist), and 008's `data-model.md` remains the *canonical schema*. The constitution back-links rather than restating, except for a short summary that lets a constitution reader understand the artifact without context-switching.

## Technical Decisions

### Section placement: §rules sits between §scenarios and §brownfield-inbox

The constitution today walks the artifact tiers in this order: §spec-phase / §spec-requirements / §spec-lifecycle, then §bug-handling and §scenarios, then §brownfield-inbox / §brownfield-process. §rules belongs alongside §scenarios as a sibling artifact-tier definition, immediately after §scenario-promotion and before §brownfield-inbox. This co-locates all three artifact tiers (specs, scenarios, rules) and reads naturally in the flow.

The HTML anchor is `<!-- §rules -->`, matching the existing convention. Adding it to the canonical-sources table in §drift-prevention is part of this work — "Rules artifact definition" → `framework/constitution.md` §rules.

### Bug decision tree extension: insert as new step 1, push existing 1–3 to 2–4

Today's §bug-handling decision tree starts with "1. No spec exists for the behavior." For cross-cutting concerns (perf budget, observability commitment, etc.), a missing spec isn't the right diagnosis — the right diagnosis is "no rule covers this." If §rules sits after §scenarios in the constitution, the decision tree should evaluate the rule question first (broadest-scope check) before falling back to spec-level questions.

Decision-tree order after this change:

1. **No rule covers this cross-cutting concern** — promote to a rule (new or amended), then fix the code.
2. **No spec exists for the behavior** — write the spec first, then fix the code.
3. **Spec exists but is ambiguous or incomplete** — fix the spec, then fix the implementation.
4. **Spec is clear but implementation is wrong** — add a scenario capturing the correct behavior, then fix the code.

The intro paragraph is rewritten to mention the three-tier framing (rule / spec / scenario) before listing the tree.

### §rules section content (concept summary + back-link to 008)

The §rules section carries:

1. **Definition** — a rule is an enforceable, citable requirement that applies across multiple features. Rule files ship under `specs/rules/{rule-set}.md`. Specs cite rules by ID.
2. **Conceptual format summary** — every rule has an ID (e.g., `BE-AUTHN-001`), a Statement (RFC 2119: MUST / MUST NOT / SHOULD / SHOULD NOT), a Rationale, and a Verification step. The full schema is canonically declared in [`specs/008-security-rules/data-model.md`](../../specs/008-security-rules/data-model.md). *(Path is from the constitution; in adopting projects the link resolves to `specs/008-security-rules/data-model.md` if 008 was scaffolded — see Edge Case below.)*
3. **When to write a rule** — the four-indicator promotion checklist (cross-cutting / citable / governance-recognized category / generalizable wording), framed as "all four should hold."
4. **When NOT to write a rule** — situational → scenario; feature-wide → acceptance criterion. Cross-references §scenarios and §spec-requirements.
5. **Lifecycle** — IDs are permanent; rules are deprecated with a removal target version, not deleted. Cross-reference 008's data-model for the full ID-stability invariants.
6. **Relationship to specs and scenarios** — three-tier table (rule / AC / scenario) selected by scope, mirroring the table in 016's spec body.

The link from the constitution to 008's data-model is a relative markdown link. In govern itself it resolves; in adopting projects, the link resolves only if 008 was scaffolded (which it always is — both security rule files use `update` strategy). Edge case: an adopter who pins both rule files and never adopts 008's spec directory would have a broken link. Acceptable — pinning has documented consequences.

### Validate generalization: prose-only, list still hardcoded

Today's `framework/commands/analyze.md` "Security rules" section names the two security files explicitly:

> Load `specs/rules/security-backend.md` and `specs/rules/security-frontend.md` if either is present in the project.

The generalization renames the section heading to "Rules" and rewrites the loading prose to be parameterized: "Load each rule file in the rule-file list (currently `specs/rules/security-backend.md` and `specs/rules/security-frontend.md`) if present in the project." The list itself remains hardcoded in analyze.md.

This is *prose-only* generalization. The list does not move to the manifest, nor is shared infrastructure introduced for cross-command list reuse. Reasoning:

- Adding a new rule file is already a per-spec event (see 008 as precedent — the rule files were added to the manifest *and* to validate as part of 008's work). The discipline of "update both lists in the same change" is a small invariant.
- The cleaner factoring (single source-of-truth list shared between `framework/bootstrap/govern.md`'s manifest and `framework/commands/analyze.md`'s loader) is exactly the kind of cross-doc invariant the deferred `/audit` command (already in the inbox) is meant to mechanize. Doing it here would expand 016's scope.
- Validate's per-rule integrity check, reference check, severity behavior, and brownfield-audit hook are unchanged.

The references to the rule-file schema location (currently "canonically declared in `specs/008-security-rules/data-model.md`") remain — that's still where the schema lives.

### Spec template: "Applicable Rules" goes between Acceptance Criteria and Open Questions

The current `framework/templates/spec/spec.md` has the body order: `{Section}` placeholder, `## Acceptance Criteria`, `## Open Questions`. The new `## Applicable Rules` section sits between `## Acceptance Criteria` and `## Open Questions` — close enough to the AC section to read together (rules constrain the same surface as ACs), but not interrupting the open-questions / clarify flow.

The section is comment-prompt only. The body is an HTML comment listing example IDs and explaining when to use it. Authors who skip the section delete the comment along with the section header; the markdownlint config allows but does not require the heading. Verified that `## Applicable Rules` headings would not collide with any existing required-heading check in `analyze.md` — the section is purely additive.

### Groom decision-tree update: add rule-promotion step before existing steps

`framework/commands/groom.md` walks a three-step decision tree that mirrors §bug-handling. After 016, this becomes a four-step walk with rule promotion as the first check. The new step text:

> **Step 1: Is this a cross-cutting concern with no covering rule?** — Apply the four-indicator promotion checklist (§rules in the constitution). If the concern qualifies, recommend promoting to a rule. Direct the user to amend the relevant rule file (e.g., `specs/rules/security-backend.md`) or, if no rule file covers the domain, note that a new rule file is its own spec (out of groom's scope) — capture the item back into the inbox with that signal.

The existing Steps 1–3 become 2–4 with no other changes. Groom still creates scenarios and appends tasks; rule promotion is *user-action*, not auto-applied (matching scenario promotion's user-decision discipline).

### 008 signpost: top-of-file note, body untouched

008's spec is `done`. Per the constitution's frozen-archaeology rule, the body is not rewritten. The signpost is a quoted note inserted between the frontmatter and the H1, of the form:

> **Signpost:** 008 defines the *security instance* of the general rules tier later formalized in [016 — Cross-Cutting Rules](../016-cross-cutting-rules/spec.md). The rule-file format, ID conventions, and validate enforcement defined here remain authoritative for security rules and serve as the canonical reference for any future rule file. See [§rules](../../framework/constitution.md) in the constitution for the general framing.

This pattern (signpost note for evolved framing) is consistent with how 006 already handles renamed commands ("`/gov:scenario` is now `/gov:amend`," etc.) — a top-of-file note rather than body rewrites.

### Generated Claude commands: regenerate via scripts/gen-claude-commands.sh

CLAUDE.md is explicit: `.claude/commands/gov/*.md` is generated from `framework/commands/`. After editing `analyze.md` and `groom.md` sources, the generator must run. The generator substitutes `{project}` → `gov` and `{cli-config-dir}` → `.claude`. No manual edits to the generated files.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/constitution.md` | Modify | Add `<!-- §rules -->` section between §scenario-promotion and §brownfield-inbox; update §bug-handling intro and decision tree (4 steps); add row to canonical-sources table |
| `framework/commands/analyze.md` | Modify | Rename "Security rules" section heading to "Rules"; rewrite loading prose to parameterize over rule-file list (list itself unchanged) |
| `framework/commands/groom.md` | Modify | Insert new Step 1 (rule-promotion check) into decision-tree walk; renumber existing 1–3 to 2–4 |
| `framework/templates/spec/spec.md` | Modify | Insert optional `## Applicable Rules` comment-prompt section between Acceptance Criteria and Open Questions |
| `specs/008-security-rules/spec.md` | Modify | Add top-of-file signpost noting 008 is the security instance of the general rules tier defined in 016; body untouched |
| `.claude/commands/gov/analyze.md` | Generate | Regenerate from `framework/commands/analyze.md` via `scripts/gen-claude-commands.sh` |
| `.claude/commands/gov/groom.md` | Generate | Regenerate from `framework/commands/groom.md` via `scripts/gen-claude-commands.sh` |
| `specs/016-cross-cutting-rules/plan.md` | Create | This file |
| `specs/016-cross-cutting-rules/tasks.md` | Create | Task breakdown |

No source code, no tests, no migrations. All artifacts are markdown.

## Trade-offs

- **Validate list remains hardcoded.** The cleaner factoring (single source-of-truth list shared between bootstrap manifest and validate's loader) is deferred to a future spec, likely tied to the `/audit` inbox item. This trades a small ongoing maintenance discipline ("update both lists in the same change") for keeping 016's scope tight.
- **§rules duplicates a small amount of 008's content.** The conceptual format summary in §rules names ID, Statement, Rationale, Verification, and RFC 2119 — the same fields described more fully in 008's data-model. This duplication is bounded and intentional (the hybrid resolution to Q2). The constitution back-links for the full schema.
- **008's body remains security-framed.** Its motivation paragraph still reads "Security rules belong at the governance level because they are cross-cutting." The signpost makes the general framing discoverable without rewriting frozen archaeology, but readers who land on 008 and don't notice the signpost may carry forward the security-only mental model. Acceptable — the constitution and 016 are now the canonical entry points.
- **"Applicable Rules" section is decorative without the deferred consistency check.** Authors can skip it without consequence (validate's existing reference check only fires on cited IDs, not on missing citations). Mitigated by inbox capture of the consistency-check work — when that ships, the section becomes load-bearing.
- **Adding §rules to canonical-sources is a small drift surface of its own.** The canonical-sources table grows by one row; future readers must remember to update it when the artifact location changes. No mitigation beyond the existing discipline of editing the table when canonical sources move.

## Open Questions Resolved

- **Naming** — keep "rule" throughout. No new vocabulary introduced.
- **Format definition home** — hybrid: constitution carries the concept summary; 008's data-model is the canonical schema and is back-linked.
- **"Applicable Rules" slot** — optional, comment-prompt section in the spec template.
- **Stronger validate consistency check** — deferred. Captured in `specs/inbox.md`; not in 016's scope.
- **Promotion threshold** — four-indicator checklist (cross-cutting, citable, governance-recognized category, generalizable wording), codified in the §rules section.
