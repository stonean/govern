---
status: clarified
dependencies: []
references:
  - service: api
    spec: 003-user
  - service: api
    spec: 099-ghost
  - service: api
    spec: 004-nostatus
  - service: frontend
    spec: 010-page
  - spec: 050-unregistered
---

# 001 — Consumer

Fixture consumer spec whose derived `references:` index exercises every
`resolve-references` outcome, in this order:

1. [api User model](https://github.com/acme/api/blob/main/specs/003-user/spec.md)
   — **ok**: `api` is registered and checked out, the target resolves, and
   its `status` is in the allowed set (`done`).
2. [api Ghost spec](https://github.com/acme/api/blob/main/specs/099-ghost/spec.md)
   — **broken**: `api` is reachable but the target spec is absent upstream.
3. [api No-status spec](https://github.com/acme/api/blob/main/specs/004-nostatus/spec.md)
   — **status-unreadable**: the target exists but its `status` is out of the
   allowed set.
4. [frontend Page](https://github.com/acme/frontend/blob/main/specs/010-page/spec.md)
   — **not-checked-out**: `frontend` is registered but its checkout is absent.
5. [other Thing](https://github.com/other/svc/blob/main/specs/050-unregistered/spec.md)
   — **unregistered**: the repo matches no `[services]` entry, so the index
   entry carries a null `service`.

The body links are illustrative; `resolve-references` reads the derived
`references:` frontmatter index above.
