---
status: in-progress
dependencies: []
review:
  last-run: 2026-06-14T23:33:11Z
  reviewed-against: d285861e4524ea7b8438dd8c3f9df37238997494
  must-violations: 0
  should-violations: 0
  low-confidence: 0
  blocking: false
---

# 030 — Cross-Service References

When a project spans multiple services — each its own repo with its own `govern` install — specs in one service routinely relate to specs in another: a frontend page uses a backend data model; a backend model is used by particular frontend pages. `govern` has no way to express that relationship across repos. The dependency graph, [§cross-spec-impact](../../framework/constitution.md#cross-spec-impact), and `/{project}:analyze`'s reference checks are all sibling-only (`scripts/gen-spec-deps.sh` resolves `](../NNN-slug/…)` and nothing else).

This feature adds **informative cross-service references**: a spec can link a spec in another service, in either direction, and — when that service is checked out locally and registered — `govern` shows the linked spec's lifecycle status next to the reference. The references are informative, never dependencies: they do not gate completion, do not enter the blocking dependency graph, and play no part in readiness or build ordering. The most `govern` does is surface the linked spec's status; nothing more.

## Informative, not a dependency

- A cross-service reference is the `## See also` navigational class extended across repos. It records a relationship for context — it is never a constraint.
- It does not enter `dependencies:`, does not affect readiness, ordering, or completion, and never blocks a gate. One project's spec never waits on another project's spec to reach `done`.
- It is **bidirectional and author's-discretion**: a frontend page may reference the backend model it uses; a backend model may reference the frontend pages that use it. The same mechanism serves both directions.
- There is **no completeness requirement and no auto-discovery**. The producer maintains no derived consumer list; references in either direction are simply links the author chose to add. A forgotten reference is a missing navigational pointer, not a silent correctness gap — which is why author-authored references are acceptable here and do not trip the no-human-diligence Design Principle in [AGENTS.md](../../AGENTS.md) (that principle only bites when completeness is load-bearing).

## Behavior

### Declaring a reference

A spec declares a cross-service reference with a **standard markdown link to the linked spec's canonical repo URL** — for example `[api User model](https://github.com/acme/api/blob/main/specs/003-user/spec.md)`. This stays within [§text-first-artifacts](../../framework/constitution.md#text-first-artifacts)' "standard markdown links, not wiki-links": the link is real and resolves for a human in GitHub or any viewer. The URL is **identity and navigation only — it is never fetched**; `govern` reads the linked spec from its local checkout, never over the network.

References are harvested into a derived **cross-service references index** (body links authoritative; the index is generated, never hand-authored), kept distinct from `dependencies:` so they never enter the blocking dependency graph.

### Resolving the linked status

A service is registered in `.govern.toml [services]` with its canonical repo and its local, reachable checkout path:

```toml
[services.api]
repo = "https://github.com/acme/api"
path = "../api"
```

When the referenced service is registered, its checkout is reachable, and the linked spec resolves, `govern` reads that spec's lifecycle `status` from its frontmatter — live, never cached — and surfaces it alongside the reference (e.g., in `/{project}:status` or when a spec's references are listed; the exact surface is a plan-phase detail). That is the entire payload: `draft`, `clarified`, `planned`, `in-progress`, or `done`. There is no baseline, no change detection, and no diff.

Otherwise the outcome depends on **what can be proven**, not a single catch-all "unknown" — the registry is **required for status resolution, optional for referencing**:

- **Unregistered** — the URL's repo matches no `[services]` entry. By design (referencing without the registry): a plain navigational link, status not attempted. Not an error.
- **Not checked out** — registered, but the local `path` is missing or not a usable checkout. Status `unknown — not checked out`: informational, never blocking, and **never reported as broken** (with no checkout, nothing can be proven).
- **Broken reference** — registered and reachable, but the reference does not resolve to a spec (malformed URL, or the spec was renamed, moved, deleted, or mistyped upstream). This is provable, so it is a **`/{project}:analyze` finding** — the cross-repo analog of a broken sibling link and a fixable defect in this spec — not a silent "unknown."
- **Status unreadable** — the linked file exists but its `status` cannot be read (no frontmatter, malformed YAML, no `status` field, a value outside the allowed set, or the link targets a scenario, which has no status). Status `unknown — status unreadable`: surfaced, never silent; the defect is upstream's.

A self-reference — a URL pointing at the consumer's own repo — is a degenerate case left to the plan phase.

Reading a frontmatter `status` field from a local file is deterministic and runtime-eligible per [§runtime-boundary](../../framework/constitution.md#runtime-boundary); the markdown-only path reads the same file with the host's file tools. No semantic judgment is involved, and distinguishing "not checked out" (cannot prove) from "broken reference" (proven) is likewise a mechanical determination.

### Brownfield adoption

Introducing a reference into an existing spec is a body edit. Because references are informative — not dependencies, acceptance criteria, or behavior — adding or removing one is a non-reopening edit: a `done` spec stays `done`. This joins the mechanical-class edits in [§spec-lifecycle](../../framework/constitution.md#spec-lifecycle) and requires an explicit carve-out there (landed when this feature is implemented).

## Non-Goals

- **No change detection.** Only the linked spec's current lifecycle status is surfaced — no baseline, no "changed since you referenced it," no diffs, no breakage assessment.
- **No remote fetch or remote checkout.** The URL is never fetched; status is read from the local checkout. `govern` adds no machinery to fetch or clone a repo (the tarball-fetch primitives were considered for an earlier, heavier framing and are excluded; see [See also](#see-also)).
- **No CI-specific cross-project functionality.** Status resolution runs only where the linked service is already locally present — a developer workstation, or a CI job that has itself checked out the sibling repo. Headless resolution without a local checkout shows status as unknown; building anything that reaches into other repos in CI is out of scope.
- **No producer-maintained or auto-discovered consumer list.** References are author-chosen, in either direction; `govern` does not enumerate or scan repos to discover who references whom.

## Acceptance Criteria

- [ ] A spec declares a cross-service reference as a standard markdown link to the linked spec's canonical repo URL; the URL is never fetched.
- [ ] References are informative: they never enter `dependencies:`, never affect readiness, ordering, or completion, and never block a gate.
- [ ] References are bidirectional — a spec may reference a spec in another service in either direction — and are author's-discretion, with no completeness requirement and no auto-discovered consumer list.
- [ ] When the linked spec's service is registered in `.govern.toml [services]` and its checkout is reachable, `govern` surfaces the linked spec's lifecycle status (read live from frontmatter) next to the reference; no baseline, change detection, or diff is produced.
- [ ] An unregistered reference (repo not in `[services]`) is a plain navigational link with status not attempted — not an error.
- [ ] A registered reference whose checkout is missing or unusable shows status `unknown — not checked out`: informational, never blocking, never reported as broken.
- [ ] A registered, reachable reference that does not resolve to a spec (malformed URL, or the spec renamed/moved/deleted/mistyped upstream) is reported as a broken-reference `/{project}:analyze` finding — distinct from an unknown status.
- [ ] A registered, reachable reference whose linked file exists but whose `status` is unreadable (no or malformed frontmatter, missing or out-of-set `status`, or a scenario target) shows status `unknown — status unreadable`: surfaced, never silent.
- [ ] References are harvested into a derived index, distinct from `dependencies:`, and never hand-authored in frontmatter.
- [ ] The deterministic work — harvest references, resolve via `[services]`, read the linked status, classify the outcome — runs through `gvrn` MCP primitives when the runtime is installed, and completes identically via the markdown-only path (host file tools) when it is not; `gvrn` is never a prerequisite, and the no-runtime CI job exercises the fallback end-to-end.
- [ ] Adding or removing an informative cross-service reference does not reopen a `done` spec — it is a non-reopening (mechanical-class) edit under §spec-lifecycle.
- [ ] A single-service adopter that declares no cross-service references sees no behavior change and creates no new configuration.

## Resolved Questions

- **Nature of a reference — informative, not a dependency.** References are navigational (the `## See also` class across repos), bidirectional, and author's-discretion. They never gate completion or enter the blocking dependency graph. The producer-side "who references me" view needs no auto-discovery or workspace enumeration — it is just the same informative reference used in the other direction, so the earlier producer-aggregate question dissolves.
- **Payload — the linked spec's lifecycle status only.** `govern` surfaces the referenced spec's current `status` and nothing more: no baseline, no change detection, no diff. (This supersedes the earlier change-record and baseline-SHA framing.)
- **Source of state — local checkout, required reachable.** Status is read live from the producer's local checkout resolved through the registry; the canonical URL is never fetched. Requiring local reachability keeps resolution deterministic and CI simple.
- **Reference syntax and the registry — canonical-URL links plus a registry required only for status.** A reference is a standard markdown link to the linked spec's canonical repo URL; `.govern.toml [services]` maps a service to its `repo` and local `path`. A URL link alone is a navigational pointer; the same link plus a matching `[services]` entry resolves the linked status from the local `path`. The registry is required for status resolution, optional for referencing.
- **Adding or removing an informative reference does not reopen a `done` spec.** Cross-service references are navigational, so their add/remove joins the mechanical/non-reopening edit class in [§spec-lifecycle](../../framework/constitution.md#spec-lifecycle) — a `done` spec stays `done`. This requires an explicit carve-out in §spec-lifecycle (a constitution edit, landed as part of implementing this feature), worded so the exemption is determinable from the diff alone — a changed or added link whose target is a registered cross-service reference — consistent with the existing mechanical-vs-meaningful test.
- **Behavior when the linked status can't be read — distinguish "can't check" from "proven broken."** Five conditions, dispositioned by what can be proven: *unregistered* → not attempted (by design); *checkout missing/unusable* → `unknown — not checked out` (informational, never flagged broken); *reachable but the reference doesn't resolve to a spec* → a broken-reference `/{project}:analyze` finding (a provable defect in this spec); *file present but `status` unreadable* → `unknown — status unreadable` (surfaced, upstream's defect); *self-reference* → deferred to the plan phase. The load-bearing distinction is between "can't check" (an informational unknown) and "provably broken" (a finding), so a broken link never hides behind a benign unknown.

## Open Questions

None remaining — all resolved above; ready for `/{project}:clarify`.

## See also

- [specs/README.md](../README.md) — §future-considerations records the cross-version upgrade-impact item. This feature deliberately does **not** realize that item; change-tracking across versions remains deferred. This is the lighter informative-references piece only.
- [015-tarball-fetch](../015-tarball-fetch/spec.md) — the remote-fetch mechanism considered for the earlier heavier framing and excluded (see Non-Goals). Links in this section are navigational and induce no dependency edge.
