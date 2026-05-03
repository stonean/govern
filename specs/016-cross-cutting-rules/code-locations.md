# 016 — Cross-Cutting Rules Code Locations

## AC: `framework/constitution.md` contains a `<!-- §rules -->` section that defines what a rule is, when to write one (the four-indicator promotion checklist: cross-cutting, citable, governance-recognized category, generalizable wording), when not to (situational → scenario; feature-wide → AC), the lifecycle (citation by ID, ID stability, deprecation), and the relationship to specs and scenarios. The section carries a short conceptual summary of the rule-file format (ID, Statement, Rationale, Verification; RFC 2119 language) and back-links to `specs/008-security-rules/data-model.md` for the canonical schema rather than duplicating it

- `framework/constitution.md`

## AC: §bug-handling in the constitution includes a fourth decision-tree route covering cross-cutting concerns. The "Three Tiers" framing above (or its equivalent) is reflected in the section

- `framework/constitution.md`

## AC: `framework/templates/spec/spec.md` includes an optional "Applicable Rules" section with prompt text explaining when to cite rule IDs (e.g., when the spec touches an area covered by a loaded rule file)

- `framework/templates/spec/spec.md`

## AC: `framework/commands/validate.md`'s "Security rules" section is renamed to "Rules." The loading logic discovers any rule file shipped under the manifest, not only `security-backend.md` and `security-frontend.md`. Existing per-rule verification, reference-checking, and severity behavior is unchanged

- `framework/commands/validate.md`
