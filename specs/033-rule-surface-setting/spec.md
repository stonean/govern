---
status: clarified
dependencies: [017-derive-dont-ask, 020-code-review, 024-rule-loader, 025-rule-opt-out]
review:
  last-run: null
  reviewed-against: null
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 033 — Explicit project rule-surface setting

A `.govern.toml` setting that declares which rule **surfaces** a project needs — backend, frontend, or both — so that rule-file installation and rule enforcement apply only the relevant surface. `/govern` prompts for the value when it is unset and persists it; `/gov:review` enforces only the configured surface(s); cross-cutting `-cross.md` rules always apply.

## Motivation

Rule files carry a closed-suffix surface signal (`-backend.md`, `-frontend.md`, `-cross.md`). [024-rule-loader](../024-rule-loader/spec.md) already *derives* which files `/gov:review` loads from the project's detected tech stack, and [025-rule-opt-out](../025-rule-opt-out/spec.md) lets a project disable individual files. Two gaps remain:

1. **No persisted, operator-set source of truth for surface.** Derivation re-runs every time and can mis-read a project whose stack is ambiguous or whose intent differs from what the code currently shows (a backend-only API repo that has not yet added any frontend; a frontend app with a thin backend-for-frontend). There is no place for the operator to *state* "this project is backend-only" once and have every command honor it.

2. **`/govern` installs every rule file regardless of surface.** A backend-only project receives `accessibility-frontend.md`, `performance-frontend.md`, and `security-frontend.md` on disk even though they never apply — noise in the tree, and (because `security-frontend.md` carries the only CSRF/secure-cookie rules) a confusing split where frontend-suffixed files hold server-side concerns.

This feature adds an explicit surface setting that both halves read: `/govern` installs and updates only the matching rule files, and the review command enforces only those rules.

## Setting

A new `.govern.toml` `[rules]` section carries a `surfaces` key whose value is a **list** of the surfaces the project needs:

```toml
[rules]
surfaces = ["backend"]          # backend-only
# surfaces = ["frontend"]       # frontend-only
# surfaces = ["backend", "frontend"]   # full-stack
```

A list (rather than a single enum) composes naturally with the suffix model and leaves room for future surfaces. Accepted member values are `"backend"` and `"frontend"`. Cross-cutting (`-cross.md`) rule files are **not a surface** — they are unconditional and always apply, so `"cross"` is not a valid member.

`[rules] surfaces` is the **operator-set source of truth** for surface selection. When it is unset, [024-rule-loader](../024-rule-loader/spec.md)'s stack derivation remains the fallback, preserving [017-derive-dont-ask](../017-derive-dont-ask/spec.md)'s default-derive posture for projects that never set it. The setting does not introduce an "ask" step anywhere except inside `/govern` (see below); no other command prompts for it.

## `/govern` behavior

- **Prompt when unset.** On a `/govern` run where `[rules] surfaces` is absent, `/govern` prompts the operator to choose the project's surface(s) and persists the answer to `.govern.toml`. This is an explicit input prompt, consistent with `/govern`'s existing first-run and agent-selection prompts. The prompt lives only in `/govern`; it does not migrate the default-derive posture of any other command.
- **Selective install/update when set.** When the setting is present, `/govern` fetches, writes, and updates only the rule files whose suffix matches a configured surface, plus all `-cross.md` files. Rule files for unconfigured surfaces are not installed, and the manifest does not flag their absence as drift.
- **Notice on contradiction.** When the explicit `surfaces` contradicts what `/govern` would otherwise detect from the stack (e.g., `["backend"]` set on a repo with obvious frontend code), `/govern` honors the operator's explicit choice but emits a one-line notice recording the discrepancy. The choice is final; the notice prevents a silent mismatch.
- **Surface change.** When `surfaces` gains a value (e.g., `["backend"]` → `["backend", "frontend"]`), the next `/govern` run installs the newly-relevant rule files. Files that became irrelevant (a surface was removed) are **left in place** but no longer updated — removal is destructive and is left to the operator (or a future explicit prune).
- **Composes with `[pinned] files`.** A pinned rule file is never overwritten, regardless of surface.

## Enforcement behavior

- [020-code-review](../020-code-review/spec.md) (`/gov:review`) loads and enforces only the rule files for the configured surface(s) plus `-cross.md`. When `[rules] surfaces` is unset, it falls back to 024's stack derivation.
- `/gov:analyze` is unchanged in its **citation resolution**: it still loads *every* rule file regardless of the configured surface, so a citation to any rule ID resolves (preserving 024's "citation verification spans surfaces" invariant). Surface selection scopes *enforcement and findings*, never citation resolution — a spec that legitimately cites an out-of-surface rule does not produce a spurious "unknown rule" finding.
- The per-file opt-out from [025-rule-opt-out](../025-rule-opt-out/spec.md) composes: surface selection chooses the candidate set, and `[[review.disabled-rule-files]]` removes individual files from it. A file already excluded by surface needs no opt-out entry.

## Acceptance Criteria

- [ ] `.govern.toml` accepts a `[rules] surfaces` list with member values `"backend"` and/or `"frontend"`; `"cross"` is rejected as invalid (cross-cutting files are unconditional).
- [ ] On a `/govern` run with `[rules] surfaces` unset, `/govern` prompts the operator to choose surface(s) and persists the choice to `.govern.toml`.
- [ ] On a `/govern` run with `[rules] surfaces` set, only rule files whose suffix matches a configured surface — plus all `-cross.md` files — are installed/updated; files for unconfigured surfaces are not written and their absence is not reported as drift.
- [ ] When `surfaces` contradicts the detected stack, `/govern` installs per the explicit setting and emits a one-line notice naming the discrepancy.
- [ ] When a surface is added to `surfaces`, the next `/govern` run installs the newly-relevant rule files; a removed surface's files are left in place and no longer updated.
- [ ] `/gov:review` enforces only the configured surface(s) plus `-cross.md` when the setting is set, and falls back to 024 derivation when unset.
- [ ] `/gov:analyze` still resolves rule citations against the full rule-file set regardless of `surfaces`, so an out-of-surface citation does not produce a spurious finding.
- [ ] A pinned rule file (`[pinned] files`) is never overwritten regardless of surface configuration.
- [ ] When `surfaces` is unset, no command outside `/govern` prompts for it and no command errors on its absence (behavior matches 024 derivation today).
- [ ] The `[rules] surfaces` setting, its accepted values, and its precedence relative to 024 derivation are documented in the `.govern.toml` schema documentation and the relevant command sources.

## Resolved Questions

- **Does the `/govern` prompt replace 024's derivation or only make it explicit and persistent?** Resolved: it does **not** replace derivation. Derive by default; prompt *only* inside `/govern`; persist the answer as an override. When `[rules] surfaces` is unset, 024's stack derivation is the fallback — preserving [017-derive-dont-ask](../017-derive-dont-ask/spec.md)'s posture for projects that never set it. No command other than `/govern` introduces an "ask" step.
- **What is the setting's shape and location?** Resolved: a **list**, `[rules] surfaces = ["backend"]`, in a new `[rules]` section. Member values `"backend"` / `"frontend"`; full-stack is both entries. A list composes with the suffix model and future surfaces better than a single enum, and `-cross.md` stays outside the surface set (always applied).
- **How does surface selection interact with `/gov:analyze`'s cross-surface citation verification?** Resolved: `/gov:analyze` still **loads all** rule files for citation resolution (024's "citation verification spans surfaces" invariant is preserved). Only enforcement and findings are surface-scoped; citation resolution is global, so an out-of-surface citation resolves rather than erroring.
- **Should an explicit surface that contradicts the detected stack warn or stay silent?** Resolved: **honor the operator's explicit choice but emit a one-line notice** recording the discrepancy. The choice is final; the notice avoids a silent mismatch (consistent with the framework's "no silent" stance).
- **When `surfaces` changes, does `/govern` install newly-relevant files and remove now-irrelevant ones?** Resolved: the next `/govern` run **installs** newly-relevant files; now-irrelevant files are **left in place** but no longer updated. Removal is destructive and is left to the operator (or a future explicit prune), mirroring how the loader and opt-out avoid deleting operator-touched files.
