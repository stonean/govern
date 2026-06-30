---
section: "Declaring a reference"
---

# Referenced-service-spec-root

## Context

Spec 030's harvest generator `scripts/gen-cross-service-refs.sh` (tasks.md §3) harvests body-link references into the derived `references:` index by matching each link's repo against a registered `.govern.toml [services]` entry and extracting the linked spec from the URL path. The extraction is a hardcoded `/specs/NNN-slug/` matcher — it assumes the *referenced* service keeps its govern artifacts under `specs/`.

Spec [040-configurable-specs-dir](../../040-configurable-specs-dir/spec.md) made the spec-root directory name an operator setting (`[paths] specs-root`, default `specs`) and made *this* repo's enumeration root-aware, including the runtime `resolve-references` primitive's checkout-side read — which already resolves a *referenced* checkout's root from that checkout's own `.govern.toml` (correct). The harvest-time URL matcher in the generator was out of 040's scope (040's acceptance criteria are scoped to the local repo) and is still `specs/`-only.

The result is a cross-repo gap: a referenced service that renamed its own spec root (e.g. to `governance/`) publishes canonical URLs like `https://github.com/acme/api/blob/main/governance/003-user/spec.md`. The `/specs/NNN-slug/` matcher does not recognize that path, so the reference is never harvested into the `references:` index and its lifecycle status is never surfaced — a silent miss, as if the author had written no reference at all (and crucially *not* a `broken` finding, which 030 reserves for a registered, reachable reference that provably fails to resolve). It surfaces only when both repos rename their roots and reference each other, which is why it was deferred from 040 (surfaced 2026-06-29 implementing 040).

## Behavior

- The harvest matcher in `gen-cross-service-refs.sh` MUST NOT assume the URL spec-root segment is literally `specs/`. The matcher is **two-tier** (per Q1):
  - **Checkout reachable** — for a referenced link whose repo matches a registered `[services]` entry and whose checkout is reachable, it resolves the referenced service's spec-root name the same way the runtime checkout side already does — read the referenced checkout's own `.govern.toml [paths] specs-root` (default `specs` when absent) — and matches `/<that-root>/NNN-slug/` in the URL exactly.
  - **Checkout unreachable** — for a registered service that is not checked out, or an unregistered repo, the referenced root cannot be read, so it falls back to a bounded permissive match of a single well-formed segment, `/<[A-Za-z0-9_-]+>/NNN-slug/spec.md`, and still harvests the reference (with its service alias when registered, null when unregistered). A reference never silently drops from the index; the single-segment bound immediately before `/NNN-slug/spec.md` keeps false positives implausible.
- A referenced service on the default `specs/` root harvests exactly as today: the common single-root case sees no behavior change.
- The markdown-only fallback path and the runtime path resolve the referenced root identically (both from the referenced checkout's `.govern.toml`), so the harvested `references:` index is byte-identical between the two paths — preserving 030's parity invariant.
- The interpolated root reuses 040's `[A-Za-z0-9_-]` well-formedness guarantee, so it is safe to splice into the generator's regex without escaping.

## Edge Cases

- **Registered but not checked out** — the referenced `.govern.toml` is unreachable, so the referenced root cannot be read at harvest time. Resolved (Q1): the matcher's bounded permissive single-segment fallback still harvests the reference under its service alias, so it surfaces as `unknown — not checked out` — consistent with 030's "not checked out → `unknown`, never `broken`, never silent" posture.
- **Unregistered link** — the repo matches no `[services]` entry; per §3 it is recorded with a null service and no `.govern.toml` is readable. Resolved (Q1): the same bounded permissive fallback recognizes the link and records it with a null service exactly as today; the root segment need not be known.
- **Both ends renamed** — the motivating case: this repo renamed its root (covered by 040) *and* a referenced service renamed its root (this scenario). The two roots are independent — this repo's root governs where the harvester writes the index; the referenced root governs URL matching.
- **Branch ref unaffected** — the matcher still ignores the branch segment (`/blob/<ref>/`) per §3; only the spec-root segment changes from a literal to a resolved value.

## Open Questions

*None — all resolved during `/gov:clarify`.*

## Resolved Questions

- **Q1: Root resolution when the referenced checkout is unavailable.** **Resolved — two-tier match.** When the referenced checkout is reachable, read its `.govern.toml [paths] specs-root` (default `specs`) and match `/<root>/NNN-slug/` exactly. When it is not reachable — a registered service that is not checked out, or an unregistered repo — the root is unknowable, so fall back to a bounded permissive match of a single well-formed segment (`/<[A-Za-z0-9_-]+>/NNN-slug/spec.md`) and harvest the reference anyway (with its service alias when registered, null when unregistered). Rejected (a) `specs/`-only fallback: it fails to even recognize a renamed-root, registered-but-not-checked-out reference, silently dropping it from the index and violating 030's "not checked out → `unknown`, never silent, never `broken`" posture. Rejected (c) record-unresolved: same silent drop. (b) keeps every reference visible in all states; the single-segment bound immediately before `/NNN-slug/spec.md` makes a false positive implausible, and a stray unregistered match is benign (status not attempted, plain navigational link).
- **Q2: Does the unregistered path need any change at all?** **Resolved — no separate change; unregistered rides Q1's permissive fallback.** The two-tier matcher already recognizes an unregistered link via the bounded `/<[A-Za-z0-9_-]+>/NNN-slug/spec.md` segment and records it with a null service exactly as today — no `specs/`-only branch and no unregistered-specific logic. This preserves the purpose of recording unregistered references (surfacing a linked-but-unregistered service so the user runs `/gov:link`), which a `specs/`-only path would defeat for a renamed-root service. Rejected the tighter `specs/`-only unregistered match: it would silently miss a renamed-root unregistered reference, and 030 already tolerates benign informational unregistered entries (status not attempted, never blocking, never a finding), so the trade favors visibility.
