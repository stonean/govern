---
section: "Follow-on scenarios"
---

# Applicable-rules-consistency-check

## Context

Spec 016 introduced the optional `## Applicable Rules` section on the spec template so authors can cite rule IDs that constrain the surface their spec touches. `/gov:analyze` already enforces one direction of the consistency invariant — per `framework/commands/analyze.md`, the per-rule verification check catches "a rule's Verification trigger fires against spec X, but spec X doesn't address the rule." That's the "missing citation" case.

The inverse direction is unenforced today: a spec can list `BE-AUTHN-001` under `## Applicable Rules` without the rule's Verification trigger actually firing against any of the spec's content. The citation is decorative, not load-bearing. Without enforcement, authors learn over time that the section is performative — citing a rule means nothing — which defeats the section's purpose. This was deferred from 016's original scope so 016 could ship the section's introduction first and let the enforcement check be designed against real usage.

## Behavior

`/gov:analyze` extends its rule-citation audit with a second direction:

- **Existing direction (kept).** For every rule whose Verification trigger fires against spec X's artifacts but which spec X's `## Applicable Rules` does not cite, emit a finding ("rule fires; not cited").
- **New direction (this scenario).** For every rule ID listed in spec X's `## Applicable Rules` section whose Verification trigger does NOT fire against any of spec X's artifacts, emit a finding ("cited; rule does not fire").

The finding makes citations earn their place. Authors who cite a rule are committing to that rule applying to their spec; if it later turns out the rule doesn't actually fire, the author either removes the citation or extends the spec to bring the cited surface into scope. Either resolution keeps the section honest.

Implementation notes (resolved during plan, not here):

- The check reuses `/gov:analyze`'s existing rule-loading logic — every loaded rule's Verification trigger is already evaluated against the target spec; the new direction is "did I evaluate every citation?" plus a comparison against the section's list.
- Citations that reference rule IDs that don't exist in any loaded rule file are a separate failure mode already caught by `/gov:analyze`'s existing rule-integrity check. The new check assumes every citation resolves to a real rule.

## Edge Cases

- **Spec has no `## Applicable Rules` section at all.** The section is optional (per 016). No section means no citations to police; the new check is a no-op for that spec. The existing direction (missing citations for rules that fire) still applies.
- **Citation is forward-looking.** An author may list a rule the spec doesn't currently address because they intend to address it in a future scenario. Today this would fire the new check. Whether to suppress that case (e.g., via a `# TODO` comment next to the citation) is a downstream refinement; v1 just emits the finding and lets the author choose the resolution path.
- **Rule rename / removal.** Rule IDs are permanent per `specs/008-security-rules/data-model.md`. A citation to a removed rule fails the existing rule-integrity check before this new check sees it. A renamed rule is a different rule (different ID), so the citation is stale and fails the existing check. The new check operates only on citations whose IDs resolve to a loaded rule — it doesn't have to handle the rename / removal case.

## Open Questions

*None — all resolved.*

## Resolved Questions

- **Severity: blocking or advisory?** **Advisory** in v1. Promotion criterion: promote to blocking when a single `/gov:analyze --all` run reports 5 or more stale citations across the repo, with the threshold met on two consecutive runs (the second run guards against transient mid-implement states where citations land before the AC that exercises them). Rationale: forward-looking citations are a useful planning signal, blocking them would push citations to land only after implementation when they're least useful; we have no baseline distribution to calibrate a blocking threshold against; symmetric with the existing "rule fires but not cited" direction, which is also advisory. The two-run guard intentionally puts the operational signal in the hands of someone running `/gov:analyze --all` regularly (CI or maintainer cadence) — if no one runs `--all`, the drift is invisible and the promotion criterion is moot, which is the right behavior since this is a maintainer concern.
