---
description: Run golangci-lint against the Go codebase and surface findings.
---

# golangci-lint

Run golangci-lint against the Go codebase and surface findings.

## Instructions

1. Detect golangci-lint config — look for `.golangci.yml`, `.golangci.yaml`, or `.golangci.toml` at the repository root. If none is found, golangci-lint runs with default linters; warn the user but continue.
2. If the project has a `lint` task in `Makefile` or `justfile`, mention it but prefer the direct `golangci-lint run` invocation for predictability.
3. Run `golangci-lint run ./...` from the repository root. If `golangci-lint` is not on PATH, report `golangci-lint is not installed` and stop — do not silently fall back to `go vet`.
4. Display the results. For each finding, show `file:line:col`, the linter that fired (e.g., `errcheck`, `staticcheck`), and the message.
5. Summarize: count of findings per linter.
6. If findings are present, treat the run as failed.

## What this workflow does NOT do

- Install golangci-lint
- Auto-fix findings — the user runs `--fix` explicitly if desired (only some linters support fix)
- Modify the golangci-lint config

## Common follow-ups

- Re-run with `--fix` after review
- Look up a finding's linter at `https://golangci-lint.run/usage/linters/`
