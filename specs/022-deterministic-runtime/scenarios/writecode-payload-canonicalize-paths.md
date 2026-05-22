---
section: "LLM extension points"
---

# Writecode-payload-canonicalize-paths

## Context

Spec 022 task #34 added the writeCode payload bundler, including a read-side secret-exfiltration guard in `runtime/src/interpreter/payload.rs:239-273`. The guard refuses paths matching `.env`, `.env.*`, `*-secrets.*`, `credentials*`, and `.gitignore`d entries — but only checks the basename. After the guard passes, `repo.join(&rel)` and `std::fs::read_to_string(&abs)` run without canonicalizing `abs` or verifying it stays under `repo`. A plan entry of `../../etc/passwd` resolves outside the repo: basename `passwd` matches nothing, libgit2 reports not-ignored, and the file's contents land in the outbound LLM payload. The same gap admits absolute paths (`/etc/passwd`) because `Path::join` lets an absolute joinee replace the base.

A separate but related bypass: `secret_pattern`'s checks are case-sensitive exact-equality on the basename. On case-insensitive filesystems (macOS APFS by default), a plan entry of `.ENV` slips past `secret_pattern` but resolves to `.env` on disk, exfiltrating the file the guard exists to protect.

Origin: `/gov:review` of spec 022 at HEAD `2873aad`, recorded as the SHOULD finding against `BE-INPUT-004` in `specs/022-deterministic-runtime/review.md`. Threat model is prompt-injection-class — a compromised `/gov:plan` LLM, or a malicious plan author bypassing PR review, plants the entry once; the next `/gov:implement` exfiltrates. Defense-in-depth tightening, not a bug in shipped behavior.

## Behavior

- `load_plan_relevant_files` MUST canonicalize each candidate path before reading. After computing `let abs = repo.join(&rel);`, the function canonicalizes both `abs` and `repo` and verifies `canon_abs.starts_with(&canon_repo)`. Paths that fail containment MUST halt the procedure with a structured error envelope; paths that resolve to a non-existent file (planned-new) MUST continue silently as today.
- `PayloadError::SecretExfiltration` grows a `pattern: "out-of-repo"` label for the path-containment failure. The envelope code stays `secret-exfiltration-blocked` for caller compatibility; the `pattern` field is the self-describing surface. (Alternative: a new `PathTraversalBlocked` variant with code `path-traversal-blocked` — chosen at implementation time; the scenario does not pre-decide.)
- `secret_pattern` MUST normalize the basename to ASCII lowercase before its pattern checks. `secret_pattern(".ENV")`, `secret_pattern("Credentials.json")`, and `secret_pattern("DB-Secrets.yaml")` MUST return the same `Some(_)` labels their lowercase counterparts return today.
- Behavior is observable through the `error` envelope and the existing in-process tests. No protocol or schema changes; no LLM-extension-point contract change.

## Edge Cases

- **Relative escape (`../foo/secret.toml`)** — canonicalize resolves out of repo; rejected with `out-of-repo` (or `path-traversal-blocked`).
- **Absolute escape (`/etc/passwd`)** — `repo.join(absolute)` is the absolute path; canonicalize stays absolute; containment check fails; rejected.
- **In-repo relative path (`runtime/src/lib.rs`)** — canonicalizes within `canon_repo`; passes through unchanged.
- **Planned-new file (`runtime/src/primitives/new_thing.rs` not yet on disk)** — `canonicalize()` errors on the missing file; the function `continue`s and omits it from the bundle, matching today's behavior for absent files.
- **Symlink to in-repo target** — canonicalize follows the link to its real path. If the real path stays under `canon_repo`, allow; if it escapes, reject. (Symlinks pointing into the repo are unusual in this codebase; the canonical check is the right floor either way.)
- **Symlink to out-of-repo target** — canonicalize resolves outside `canon_repo`; rejected by the containment check. This is a defense-in-depth win the basename-only guard missed.
- **Case-fold bypass on case-insensitive FS (`.ENV`, `Credentials.json`, `DB-Secrets.yaml`)** — lowercased basename matches the existing patterns; rejected by `secret_pattern` before the containment check runs.
- **Mixed-case escape via case-insensitive FS (`../FOO/.ENV`)** — fails the case-fold bypass first; redundant defense from containment check.
- **Non-canonical `repo` argument (rare; e.g., path with `..` mid-string)** — `repo.canonicalize()` resolves it once at function entry; subsequent containment checks operate on the canonical form.

## Done-when

`load_plan_relevant_files` canonicalizes every candidate path and rejects out-of-repo escapes with a structured error envelope. `secret_pattern` matches case-insensitively against the basename. Five scenarios cover the new behavior — relative escape, absolute escape, in-repo happy path, planned-new file (still skips via the canonicalize-fails-continues branch), and case-fold bypass — implemented as four new regression tests plus the existing planned-new test. The `gvrn` crate ships a patch bump (`0.7.3`; `0.7.2` was already claimed by the unrelated `enforce-manifest` contract trim in 027.5).

## Open Questions

*None — captured during scenario authoring.*

## Resolved Questions

*None yet.*
