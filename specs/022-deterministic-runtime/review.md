---
spec: 022-deterministic-runtime
reviewed-at: 2026-05-22T00:00:00Z
reviewed-against: 2873aad8dfac63e4743964693d1ad60af8fc4ea4
diff-base: 5d04421c157c042f7d23e170fbffd9ad8797661b
must-violations: 0
should-violations: 1
low-confidence: 0
skipped-passes: []
---

# Review — 022-deterministic-runtime

## Summary

Whole-spec review at HEAD (`2873aad`). The prior scoped review (at `389b68b`, mark-task-backtick-headings scenario) cleared the runtime through gvrn 0.6.1; this pass picks up the unreviewed delta — primarily task #34 (writeCode payload bundling, gvrn 0.7.0), the gvrn 0.7.1 dependency-major refresh, and the 027.5 contract trim that touches `enforce_manifest`'s docstring.

Stack: text-first markdown + Rust runtime. Loaded rule files: `api-backend.md`, `configuration-cross.md`, `security-backend.md`. Frontend rule files are skipped — no frontend surface in scope. No `[[review.disabled-rule-files]]` entries.

Five-dimension review of the delta:

- **Security**: writeCode bundling adds a new read-side surface — the runtime opens files whose paths come from the targeted feature's `plan.md` Affected Files table. The existing guard (`secret_pattern` + `is_gitignored`) handles the named-file vectors well (`.env`, `.env.*`, `*-secrets.*`, `credentials*`, gitignored entries). However, `load_plan_relevant_files` does not canonicalize the joined path before reading, so a plan with `../` or absolute-path entries can escape the repo root — see SHOULD finding below. No other security surface is touched by the delta; the dep-major refresh sits on well-known crates and the migration was mechanical.
- **Reuse**: writeCode payload assembly correctly delegates to the existing `read_tasks` primitive for task lookup and the existing `resolve-anchor` mechanism for constitution excerpts. No duplicated parsing. The `gvrn 0.7.1` clippy-driven let-chain collapse across `interpreter/payload`, `primitives/{append_task, read_spec, mod}`, and `main` is a defensible idiom-update under MSRV 1.88 — no behavior change, idiom drift removed.
- **Quality**: error propagation from `build_extension_request` through `handle_extension` is clean: `PayloadError::SecretExfiltration` becomes an `error` envelope with code `secret-exfiltration-blocked` and the walk terminates via `WalkOutcome::Errored`. The `WriteCodeRequest` field-order lock (Subtask 34.4) is enforced by a serialization-order test — good. `is_gitignored` swallows libgit2 errors and returns false; the doc-comment is explicit that the gitignore layer is opt-in and the secret-pattern check is the floor.
- **Efficiency**: payload bundling reads each file once; no N+1. The constitution-excerpts loader calls `resolve-anchor` per anchor, but the anchor count is bounded by the command file's `Reference:` line. No concerns.
- **Simplicity**: `runtime/src/interpreter/payload.rs` lands at 873 lines but each function does one thing and the module-doc walks the reader through the flow. The `secret_pattern` helper is small and well-tested. No premature abstraction.

Test posture: per CHANGELOG, 325 tests pass at gvrn 0.7.1; clippy --all-targets -- -D warnings clean; fmt --check clean. The lint suite (lint-procedure-parseability, lint-tool-coverage, lint-frontmatter, markdownlint-cli2 over the 022 spec dir) is clean as of the MD038 fix landed in this session.

**Result**: 0 MUST, 1 SHOULD, 0 low-confidence. `blocking: no`.

## MUST violations (blocking)

_None._

## SHOULD violations (advisory)

### SHOULD: BE-INPUT-004 — `load_plan_relevant_files` lacks canonical-path containment check

- **File**: `runtime/src/interpreter/payload.rs:239-273`
- **Rule**: User-supplied values MUST NOT be used directly in filesystem paths. Filesystem operations MUST resolve the canonical path and verify it falls within the expected base directory before opening the file.
- **Finding**: `load_plan_relevant_files` reads path entries from the targeted feature's `plan.md` Affected Files table and bundles each file's content into the `writeCode` payload sent to the LLM. The existing guard rejects entries matching the secret-bearing patterns (`.env`, `.env.*`, `*-secrets.*`, `credentials*`) and gitignored paths, then calls `repo.join(&rel)` and `std::fs::read_to_string(&abs)` without canonicalizing `abs` or verifying it stays under `repo`. A plan entry of `../../etc/passwd` resolves outside the repo: the basename (`passwd`) does not match any secret pattern, libgit2 reports the path as not-ignored, and the file's contents are bundled into the outbound LLM payload. The same gap applies to absolute paths (`/etc/passwd`) — `Path::join` lets an absolute joinee replace the base. Threat model is prompt-injection-class: a compromised `/gov:plan` LLM (or a malicious plan author bypassing PR review) plants the entry once; `/gov:implement` exfiltrates on the next run. Severity is SHOULD rather than MUST because the input source (plan.md) is project-authored and reviewed, but the canonicalization check is the rule-mandated defense-in-depth and would close the prompt-injection vector deterministically.
- **Auto-fixable**: no (the fix is mechanical but the test coverage needs new fixtures — escape-via-relative, escape-via-absolute, and a happy-path in-repo file — so author judgment is needed for the test scope).
- **Suggested fix**: in `load_plan_relevant_files`, after computing `let abs = repo.join(&rel);`, canonicalize both `abs` and `repo` and verify containment before reading. Sketch:

  ```rust
  let abs = repo.join(&rel);
  let canon_abs = match abs.canonicalize() {
      Ok(p) => p,
      Err(_) => continue, // missing file — planned-new, same as today
  };
  let canon_repo = repo.canonicalize().map_err(|_| PayloadError::SecretExfiltration {
      path: rel.clone(),
      pattern: "non-canonical-repo".into(),
  })?;
  if !canon_abs.starts_with(&canon_repo) {
      return Err(PayloadError::SecretExfiltration {
          path: rel,
          pattern: "out-of-repo".into(),
      });
  }
  let Ok(content) = std::fs::read_to_string(&canon_abs) else { continue; };
  ```

  Add a third pattern label (`"out-of-repo"`) to the `SecretExfiltration` enumeration's documentation so the error envelope's `pattern` field stays self-describing. Regression tests: one plan with `../foo.txt`, one with `/etc/hosts`, one with `subdir/legitimate.rs` (must pass through). Also consider lowercasing the basename before secret-pattern matching to close the related case-insensitive-filesystem gap (`.ENV` on macOS APFS resolves to `.env` but bypasses `secret_pattern`'s exact-equality check). The fix is one `.to_ascii_lowercase()` call on the captured basename; happy to defer if you'd rather keep this finding tight to the path-containment issue.

## Low-confidence findings

_None._

## Waived findings

_None._

## Skipped passes

_None._
