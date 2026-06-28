# 033 — Explicit project rule-surface setting Tasks

Tasks derived from the [plan](plan.md). Complete in order.

## 1. Add `[rules] surfaces` input resolution to the installer

- [ ] In `framework/bootstrap/govern.md` §Collect Project Inputs, add `[rules] surfaces` as a resolved input: read from `.govern.toml` `[rules] surfaces`, else prompt ("backend / frontend / both"), else persist the answer into `.govern.toml` `[rules]` (preserving every other section).
- [ ] Document accepted members `{"backend", "frontend"}`, rejection of `"cross"`, and that unset means "derive / install all".
- Done when: a first scaffold prompts once and persists `[rules] surfaces`; a routine re-run with the key present prompts nothing.

## 2. Filter installer manifest rule-file entries by surface

- [ ] In `framework/bootstrap/govern.md`, filter the rule-file manifest entries to suffixes matching a configured surface plus every `*-cross.md`, before `apply-manifest`.
- [ ] Ensure omitted (unconfigured-surface) entries are never added to a prune/enforce set — already-installed files for a now-unconfigured surface are left in place.
- [ ] Add the one-line contradiction notice when `surfaces` excludes a surface implied by `[project] languages`.
- Done when: with `surfaces=["backend"]`, only `*-backend.md` + `*-cross.md` rule files are applied; pre-existing `*-frontend.md` files are not deleted; a contradiction emits one advisory line and still honors the setting.

## 3. Make `/gov:review` consult `[rules] surfaces`

- [ ] In `framework/commands/review.md` §Behavior step 5, read `.govern.toml` `[rules] surfaces`. When set, keep rule files whose surface is in `surfaces` plus every `*-cross.md` and unrecognized-suffix file (replacing the detected-stack filter); when unset, run the detected-stack filter as today. The 025 disabled-files filter runs after, unchanged.
- [ ] Document `[rules] surfaces` in §Inputs alongside `tech-stack-verified` and `disabled-rule-files`.
- Done when: with `surfaces` set, `/gov:review` enforces only the configured surface(s) + `-cross`; with it unset, behavior is identical to today.

## 4. Note analyze is unaffected

- [ ] In `framework/commands/analyze.md`, add a short note that `[rules] surfaces` scopes `/gov:review` enforcement only and never prunes the rule-file set `/gov:analyze` loads for citation resolution.
- Done when: the note is present and analyze's load-all behavior is documented as independent of `surfaces`.

## 5. Document the setting

- [ ] Document `[rules] surfaces` in `README.md` (and any `.govern.toml` reference): accepted values, derive-when-unset fallback, precedence over stack detection.
- Done when: a reader can configure `[rules] surfaces` from the docs without reading the command sources.

## 6. Resolve cross-spec impact on 024 / 020

- [ ] Once the `review.md` edit is final, decide whether [024-rule-loader](../024-rule-loader/spec.md) (and/or [020-code-review](../020-code-review/spec.md)) needs a new acceptance criterion or scenario describing the surface-aware source, with a back-link to this spec — or whether the additive, fallback-preserving design needs no change. Route any update through `/gov:amend`.
- Done when: the decision is recorded (a back-linked AC/scenario on 024/020, or an explicit "no change — additive layering" note here).

## 7. Validate

- [ ] Pre-commit generators regenerate `.claude/commands/gov/*.md` from the edited `framework/commands/*` sources cleanly.
- [ ] `npx markdownlint-cli2`, `scripts/lint-*.sh`, and `scripts/audit/*` pass.
- [ ] Run `/gov:review` over the change set; resolve any MUST findings before `done`.
- Done when: all lints/audits pass and `/gov:review` reports no blocking violations.
