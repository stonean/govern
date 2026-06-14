---
status: done
dependencies: []
---

# 003 — User Model

Fixture-only stand-in for a spec in a *registered, locally checked-out*
sibling service (`api`). The `analyze-basic` consumer (`003-analyze`)
references this spec; `resolve-references` reads the `status: done` here
and classifies the reference as `ok` — the clean reference.

The companion `099-ghost` reference points at a spec that does not exist
under this checkout, so it classifies as `broken` — the Advisory finding
`/gov:analyze` raises. This file lives under `checkouts/`, not `specs/`,
so the analyze walk over the feature directory never picks it up.
