---
status: done
dependencies: []
---

# 003 — User Model

Fixture-only stand-in for a spec in a *registered, locally checked-out*
sibling service (`api`). The `status-basic` consumer (`001-basic`)
references this spec; `resolve-references` reads the `status: done` here
and classifies the reference as `ok`.

This file lives under `checkouts/`, not `specs/`, so the `dashboard`
primitive's `specs/` walk never picks it up — it exists only for the
cross-service reference readout.
