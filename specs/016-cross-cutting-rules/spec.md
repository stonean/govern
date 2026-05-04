---
title: "016-cross-cutting-rules — spec"
status: done
dependencies: [006-bug-workflow, 008-security-rules]
tags: [format, process, pipeline]
---

# 016 — Cross-Cutting Rules

Promote rules to a first-class artifact tier alongside specs and scenarios. The rule infrastructure already exists but is framed as security-specific; this spec generalizes the framing, the bug decision tree, the validate plumbing, and the spec template so cross-cutting concerns of any kind (security, performance, concurrency, observability, accessibility, etc.) land in a single, consistent home.

## Motivation

`govern` already has working rule infrastructure. Spec 008 defined a strict rule format (ID, Statement, Rationale, Verification), wired per-rule verification into `/gov:validate`, hooked brownfield adoption into the inbox, and shipped two rule files (`security-backend.md`, `security-frontend.md`).

What 008 did *not* do was name **rules** as a general artifact tier. The constitution has §spec-phase, §scenarios, and §bug-handling, but no §rules. The bug decision tree routes a bug to a missing spec, an ambiguous spec, or a missing scenario — but never to a missing rule. The `/gov:validate` section enforcing rules is titled "Security rules." The spec template has no slot for citing applicable rules. The result: rules behave like a security feature rather than the artifact tier they actually are.

The user-facing consequence is that cross-cutting concerns outside security (perf budgets, concurrency discipline, observability commitments, accessibility minimums) have no obvious home. They get re-derived per feature as acceptance criteria, scattered across specs that should reference a shared rule.

## The Three Tiers

After this spec, every requirement in `govern` lands at one of three levels of abstraction:

| Tier | Scope | Artifact |
| --- | --- | --- |
| **Rule** | Cross-cutting (applies across many features) | A rule file under `specs/{rule-set}.md`, cited by ID from the specs that depend on it |
| **Spec / acceptance criterion** | Feature-wide (one feature, broad property) | A section or AC in the feature's `spec.md` |
| **Scenario** | Situational (a specific condition with concrete behavior) | A file in the feature's `scenarios/` directory |

Bugs route to the tier that matches the *scope* of the missing or violated requirement. A perf bug that affects every API endpoint promotes to a rule. A perf bug specific to one feature becomes an acceptance criterion. A perf bug that only manifests under a specific concurrency condition becomes a scenario.

## Scope

The artifact tier and validate plumbing already exist. This spec generalizes the framing and adds the missing decision-tree route.

### In scope

- **Constitution** — add a §rules section that defines the artifact, its lifecycle, when to write one, when not to, and how it relates to specs and scenarios. Reference 008 for the canonical rule-file format rather than duplicating it.
- **Bug decision tree** — add a fourth route to §bug-handling: cross-cutting concerns with no covering rule promote to a new or amended rule, then the code is fixed.
- **Validate** — rename the "Security rules" section in `framework/commands/validate.md` to "Rules". Generalize the loading logic to discover any rule file shipped under the manifest, not only the two security files.
- **Spec template** — add an optional "Applicable Rules" section to `framework/templates/spec/spec.md` that prompts the author to cite rule IDs the spec relies on.
- **Groom** — update `/gov:groom`'s decision-tree walk so cross-cutting items can be routed to rule promotion alongside the existing spec/scenario routes.
- **008 reframing** — add a top-of-file signpost to `specs/008-security-rules/spec.md` clarifying that 008 is the *security instance* of the general rules tier defined here. The 008 body is not rewritten (per the constitution's frozen-archaeology rule).

### Out of scope

- Shipping a non-security rule file (e.g., `rules/observability.md`). The infrastructure should be ready for it, but the first non-security rule file is its own spec.
- Changing the rule file format. 008's format (ID, Statement, Rationale, Verification, RFC 2119 language) stands.
- Changing validate's enforcement logic. Only the section framing and discovery loop change — per-rule verification, reference-checking, and the brownfield audit hook remain as 008 defined them.
- Auto-promoting scenarios or acceptance criteria to rules. Promotion is a user decision.
- Inventing a new ID-prefix scheme for non-security rule sets. Defer to the spec that introduces the first non-security rule file.

## Edge Cases

- **Pinned files in adopting projects** — projects that pin `framework/commands/validate.md` or the spec template in `.govern.toml` will not pick up the renamed "Rules" section or the "Applicable Rules" slot on the next `/govern` re-run. This is the documented consequence of pinning (per the README's "Pinning files with .govern.toml" section); 016 introduces no special migration path. Adopters who unpin will pick up the changes; those who stay pinned keep their customized copies.
- **Manifest discovery scope** — the validate generalization discovers rule files via the manifest, not by globbing `specs/*.md`. This avoids false-matching `system.md`, `errors.md`, `events.md`, or any other top-level spec file. New rule files require a manifest entry to be picked up; this is intentional — rule files are governance-distributed, not project-authored.
- **Existing "Applicable Rules" sections in adopting-project specs** — if an adopter has independently added an `## Applicable Rules` section before 016 ships, validate's existing reference check (which scans for inline rule-ID patterns regardless of section heading) continues to work without modification. The new template prompt is additive, not normative.
- **008's body becoming inconsistent with the new framing** — 008 is `done` and the constitution's frozen-archaeology rule prevents rewriting its body. The signpost added by 016 is the only mutation; readers who want the general framing follow the signpost to the constitution and to 016.
- **Authors who cite a rule before any rule file ships in a domain** — for example, citing a hypothetical `OBS-LATENCY-001` before any observability rule file exists. Validate's existing unknown-reference check (`validate.md:138`) flags this as blocking, which is correct: a citation to a non-existent rule is a drift indicator regardless of intent.

## Acceptance Criteria

- [x] `framework/constitution.md` contains a `<!-- §rules -->` section that defines what a rule is, when to write one (the four-indicator promotion checklist: cross-cutting, citable, governance-recognized category, generalizable wording), when not to (situational → scenario; feature-wide → AC), the lifecycle (citation by ID, ID stability, deprecation), and the relationship to specs and scenarios. The section carries a short conceptual summary of the rule-file format (ID, Statement, Rationale, Verification; RFC 2119 language) and back-links to `specs/008-security-rules/data-model.md` for the canonical schema rather than duplicating it.
- [x] §bug-handling in the constitution includes a fourth decision-tree route covering cross-cutting concerns. The "Three Tiers" framing above (or its equivalent) is reflected in the section.
- [x] `framework/commands/validate.md`'s "Security rules" section is renamed to "Rules." The loading logic discovers any rule file shipped under the manifest, not only `security-backend.md` and `security-frontend.md`. Existing per-rule verification, reference-checking, and severity behavior is unchanged.
- [x] `framework/templates/spec/spec.md` includes an optional "Applicable Rules" section with prompt text explaining when to cite rule IDs (e.g., when the spec touches an area covered by a loaded rule file).
- [x] `framework/commands/groom.md` includes "promote to rule" as a routing option in its decision-tree walk, alongside the existing spec/scenario routes.
- [x] `specs/008-security-rules/spec.md` carries a top-of-file signpost noting it is the security instance of the general rules tier defined in 016.
- [x] All modified markdown files pass `npx markdownlint-cli2`.
- [x] `/gov:validate --all` passes against the modified govern repo (no new findings introduced by the spec/template/validate changes themselves).

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Naming** — keep "rule." These are binding RFC-2119 statements that `/gov:validate` enforces, not guidelines. "Policy" reads softer and would understate the enforcement intent; it would also fork vocabulary already established by 008, the `framework/rules/` directory, and the constitution's "Secure" principle reference.
- **Where does the rule-file format definition live?** — hybrid. The constitution's §rules section carries a short conceptual summary (rules are ID'd, citable, RFC-2119, validate-checked, with the four required fields named) and back-links to `specs/008-security-rules/data-model.md` for the full schema. 008's data-model remains the canonical declaration that validate's integrity check refers to. This avoids the documentation cliff of (a) while preserving canonical-source discipline against (b).
- **Required vs optional "Applicable Rules" slot in the spec template** — optional, surfaced as a comment-prompt section in the template (matching the existing convention used for the scenarios hint and the `## {Section}` placeholder). An optional section traces back to validate's existing unknown-rule-reference check (`validate.md:136-139`); making it required would imply a stronger validate consistency check that Q4 explicitly defers. Specs that don't touch a ruled area stay clean.
- **Validate alignment** — defer the stronger check (validate comparing triggered rules against listed Applicable Rules) out of 016's scope. The existing per-rule verification check (`validate.md:130-134`) already catches rules whose triggers fire against an unaddressing spec, so 016 ships with the optional template section + existing reference check. The deferred work is captured in `specs/inbox.md` so it surfaces in a future `/gov:groom` pass — keeps 016 focused without losing the follow-up signal.
- **Promotion threshold** — promotion checklist (mirrors §scenario-promotion's structure). The constitution's §rules section enumerates indicators rather than a numeric bar: (1) the concern applies to multiple existing or anticipated features (cross-cutting test), (2) verification can be expressed as a step a reviewer or validate can check (citable test), (3) the concern belongs to a governance-recognized category (security, performance, concurrency, observability, accessibility, audit/compliance, data handling) rather than a single feature's domain, (4) the same wording would make sense in any spec touching the area. Pure principle is too thin; numeric thresholds are arbitrary and game-able. The checklist still requires judgment, which matches the framework's stance everywhere else.
