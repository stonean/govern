---
section: "Config file: `.govern/config.toml`"
---

# Provenance-tags-name-resolved-config-path

## Context

Spec 042 moved per-project config to `.govern/config.toml`, with the legacy root `.govern.toml` retained only as a read fallback. Two runtime-emitted display literals still hardcode the legacy filename in their disabled-rule-file provenance tags:

- `discover_rule_files.rs:273` — `disabled-rule-file: … (.govern.toml)`
- `dashboard.rs:234` — `disabled rule files: {N} (.govern.toml) — …`

Both literals are mirrored verbatim in the command docs — `review.md:197` and `status.md:59` — under the doc↔runtime message-parity contract, so a doc-only edit (or a runtime-only edit) breaks parity.

## Behavior

- The disabled-rule-file provenance tags emitted by `discover-rule-files` and `dashboard` name the **resolved** config path — the active file the config resolver selected (`.govern/config.toml` post-migration, the legacy root `.govern.toml` on a pre-migration project) — rather than a hardcoded `(.govern.toml)` literal.
- The doc mirrors in `review.md` and `status.md` are updated in the same change to match the runtime wording, preserving doc↔runtime message parity.
- Any parity goldens that assert the old literal are regenerated/updated in the same change, so the parity suite stays green.

## Edge Cases

- **Pre-migration project (legacy layout):** the resolver selects root `.govern.toml`, so the tag names that path — the tag is truthful on both layouts, not merely rewritten to the new literal.
- **Doc mirror wording:** if the docs cannot render a dynamic path, they document the tag shape (e.g., `({resolved config path})`) rather than hardcoding either filename.

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
