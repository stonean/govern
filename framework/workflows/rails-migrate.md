---
description: Apply pending Rails database migrations and report the schema changes
---

# Rails migrations

Apply pending Rails database migrations and report the schema changes.

## Instructions

1. Verify this is a Rails application — look for `bin/rails` and `config/application.rb`. If neither is present, report `Not a Rails application` and stop.
2. Determine the target environment. Default to `development` (the `RAILS_ENV` default). Never run against `production` unless the user explicitly asked for it and confirmed — production migrations are a separate, deliberate operation.
3. Show pending migrations first. Run `bin/rails db:migrate:status` (prefer `bin/rails`, fall back to `bundle exec rails`). Display which migrations are `up` versus `down`. If nothing is pending, report `schema is up to date` and stop.
4. Apply the migrations. Run `bin/rails db:migrate` from the repository root. Capture the per-migration output (`migrating` / `migrated` lines and elapsed time).
5. Report the resulting schema version and the files Rails regenerated — `db/schema.rb` (or `db/structure.sql` if the project uses the SQL schema format).
6. If a migration fails, show the failing migration and the error. Treat the run as failed, and warn that a partial failure can leave the schema mid-migration — the user may need to roll back or fix forward.

## What this workflow does NOT do

- Run against `production` without an explicit, confirmed request
- Edit migration files, `db/schema.rb`, or `db/structure.sql` by hand
- Seed (`db:seed`) or reset/drop the database (`db:reset`, `db:drop`) — those are separate, destructive operations
- Install Rails or gems

## Common follow-ups

- Roll back the last migration with `bin/rails db:rollback` (add `STEP=n` for multiple)
- Re-check state with `bin/rails db:migrate:status`
- Commit the regenerated `db/schema.rb` alongside the migration
