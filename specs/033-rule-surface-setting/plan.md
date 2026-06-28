# 033 — Explicit project rule-surface setting Plan

Implements [033 — Explicit project rule-surface setting](spec.md).

## Overview

A markdown-tier change — no runtime code. The feature threads one new `.govern.toml` setting, `[rules] surfaces`, through the two places that already act on rule-file surface:

1. **`framework/bootstrap/govern.md`** (the installer) — resolve `[rules] surfaces` as a project input (prompt when unset, persist), then filter the host-built rule-file manifest entries by the configured surface(s) before `apply-manifest` runs.
2. **`framework/commands/review.md`** (the `/gov:review` loader, spec 024) — consult `[rules] surfaces` in §Behavior step 5; when set, it replaces the detected-stack filter; when unset, stack derivation remains the fallback.

`/gov:analyze` is deliberately untouched: it already loads *every* rule file regardless of stack, and that load-all behavior is what makes cross-surface citation resolution work — so surface selection must not reach it.

## Technical Decisions

### Setting schema — `[rules] surfaces`

`.govern.toml` gains a `[rules]` table with one key:

```toml
[rules]
surfaces = ["backend"]   # list; members ∈ {"backend", "frontend"}; full-stack = both
```

- A **list**, not an enum — composes with the existing suffix model (`-backend`/`-frontend`/`-cross`) and admits future surfaces without a schema change.
- Members are validated against `{"backend", "frontend"}`. `"cross"` is rejected: `-cross.md` files are unconditional and not a selectable surface.
- **Unset** is a first-class state meaning "derive" — the loader falls back to 024's stack detection and the installer installs all rule files (today's behavior), so existing adopters are unaffected until `/govern` next prompts.

### Installer (`framework/bootstrap/govern.md`)

- **Input resolution.** Add `[rules] surfaces` to §Collect Project Inputs as a resolved input: read from `.govern.toml` `[rules] surfaces`; if absent, prompt ("Which rule surfaces does this project need? backend / frontend / both"); persist the answer into `.govern.toml` `[rules]`, preserving every other section (same pattern as `[project] name/description/languages`). On a routine re-run the value is present, so no prompt fires.
- **Manifest filter.** The host already builds `manifest-entries`. Filter the rule-file entries (`framework/rules/*.md` → adopter `specs/rules/*.md`) to those whose suffix matches a configured surface, **plus every `*-cross.md` unconditionally**, before calling `apply-manifest`. Entries for unconfigured surfaces are simply omitted from the manifest — never added to any prune/enforce set, so an already-installed file for a now-unconfigured surface is **left in place** (rule files are not in `enforce-directories`; only slash-command dirs are pruned).
- **Contradiction notice.** When `surfaces` excludes a surface that `[project] languages` clearly implies (e.g., `surfaces=["backend"]` but a frontend language is listed), emit one advisory line; the explicit setting still wins. No prompt.

### Review loader (`framework/commands/review.md`)

§Behavior step 5 currently: discover by suffix → filter by detected stack → apply disabled-files filter. Insert a surface source ahead of the stack filter:

- Read `.govern.toml` `[rules] surfaces`. **If set**, keep rule files whose surface is in `surfaces`, plus every `*-cross.md` and every unrecognized-suffix file; this *replaces* the detected-stack filter. **If unset**, run the detected-stack filter exactly as today.
- The §Inputs section documents `[rules] surfaces` alongside `[review] tech-stack-verified` and `[[review.disabled-rule-files]]`.
- The 025 disabled-files filter runs after, unchanged. A file already excluded by surface needs no opt-out entry.

### `/gov:analyze` and citation resolution

No behavior change. `/gov:analyze` continues to load every discovered rule file for citation resolution regardless of `surfaces` (the constitution's "loads every discovered file regardless of stack" invariant). A short note is added to `framework/commands/analyze.md` making explicit that `[rules] surfaces` scopes `/gov:review` enforcement only, never analyze's citation set.

### Documentation

`README.md` (and any `.govern.toml` reference) documents `[rules] surfaces`: accepted values, the derive-when-unset fallback, and its precedence over stack detection.

## Affected Files

| File | Action | Purpose |
| --- | --- | --- |
| `framework/bootstrap/govern.md` | Modify | Resolve/prompt/persist `[rules] surfaces`; filter manifest rule-file entries by surface (+ always `-cross`); contradiction notice |
| `framework/commands/review.md` | Modify | §Behavior step 5 consults `[rules] surfaces` (replaces stack filter when set, falls back when unset); §Inputs documents the setting |
| `framework/commands/analyze.md` | Modify | Note that `surfaces` scopes review enforcement only, not analyze citation resolution |
| `README.md` | Modify | Document the `[rules] surfaces` setting and its precedence |
| `specs/024-rule-loader/spec.md` | Possibly modify | Cross-spec impact — see below (additive; may not require a change) |
| `specs/020-code-review/spec.md` | Possibly modify | Cross-spec impact — see below |

`.claude/commands/gov/*.md` regenerate from the `framework/commands/*` sources via the pre-commit hook — not edited by hand.

## Trade-offs

- **List vs. single enum** — chose list. An enum (`backend|frontend|fullstack`) is simpler to validate but does not compose with the suffix model or future surfaces; the list mirrors how files are already classified.
- **Prompt-in-`/govern` vs. replace derivation everywhere** — chose: derivation stays the default, the prompt lives only in `/govern`, and the persisted value is an override. Forcing an "ask" into every command would reverse [017-derive-dont-ask](../017-derive-dont-ask/spec.md); confining it to the installer keeps zero-config projects zero-config.
- **Leave vs. prune on surface removal** — chose leave-in-place. Deleting a now-irrelevant rule file is destructive and could drop adopter-pinned edits; omitting it from the manifest (no update, no delete) is reversible and consistent with how `[pinned]` already protects files. A future explicit prune can be added if demand appears.
- **Analyze load-all vs. surface-scoped** — chose load-all (no change). Scoping analyze to `surfaces` would make a legitimate cross-surface citation (a backend spec citing a frontend rule) report as an unknown rule; resolution must stay global while only enforcement is scoped.
- **Known limitation** — the contradiction check is heuristic (compares `surfaces` against `[project] languages`), advisory only, and will not catch every mismatch; the operator's explicit choice is always honored.

## Cross-spec impact

The implementation edits `framework/commands/review.md` (conceptually owned by [020-code-review](../020-code-review/spec.md) and [024-rule-loader](../024-rule-loader/spec.md)) and `framework/bootstrap/govern.md`. The design is **additive**: when `[rules] surfaces` is unset, 024's stack derivation and 020's review behavior are byte-for-byte unchanged. Because 033 layers a new source ahead of an unchanged fallback rather than altering the existing contracts, 024/020 likely do not need a spec edit — but the surface-aware step is a behavior 024 describes, so the final call (a new acceptance criterion or scenario on 024 with a back-link to 033 vs. no change) is made at implement time once the exact `review.md` edit is written. This note is informational and does not block the transition.
