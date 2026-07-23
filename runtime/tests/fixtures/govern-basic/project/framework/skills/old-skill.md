# legacy skill

Pre-seeded by the fixture. Represents the old `framework/skills/`
convention that adopters cleaned up via the bootstrap's `enforce-manifest`
loop before spec 027 moved adopter-cleanup into the registry-driven
`## Pre-run Migrations` flow. `enforce-manifest` is now scoped to the
per-agent slash-command directory and must NOT touch this file — the
registry-driven Pre-run Migrations loop (`framework/migrations.toml`,
currently via the `workflows-sunset` entry) is the sole owner of that
cleanup.
