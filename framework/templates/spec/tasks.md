# {NNN} — {Feature Name} Tasks

Tasks derived from the [plan](plan.md). Complete in order.

<!-- Each task should be small enough to implement and verify independently.
     Mark tasks as they are completed.

     Optional `[simple]` tier marker:
     A task header MAY end with `[simple]` to signal that the task is trivial
     (single small file, no logic, no schema change, no new behavior). No marker
     means the default tier — whatever the adopter's platform config maps to
     "standard". Only one tier is defined; `[complex]` is intentionally absent
     because the default is already "use the strongest model." Markers live on
     individual task headers, not on the file as a whole, so different tasks in
     the same feature can have different tiers. `/gov:plan` proposes markers
     during planning; the user adds, removes, or accepts them before approving.
     `/gov:implement` reads the marker and surfaces a suggested model — it does
     not auto-switch.

     Example:

## 1. Create sessions table migration

- [ ] Write SQL migration for `sessions` table
- [ ] Run migration and verify schema

## 2. Implement session store

- [ ] Create `shared/auth/session.go` with Create, Get, Delete methods
- [ ] Write store integration tests against real PostgreSQL

## 3. Update README link to migration guide [simple]

- [ ] Edit `README.md` to point at the new path

-->
