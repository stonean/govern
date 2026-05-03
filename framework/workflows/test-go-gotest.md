---
description: Run the Go test suite via the go toolchain.
---

# go test

Run the Go test suite via the go toolchain.

## Instructions

1. Verify a `go.mod` exists at the repository root. If not, report `Not a Go module` and stop.
2. If the project has a `test` task in `Makefile` or `justfile`, mention it but prefer the direct `go test` invocation for predictability.
3. Run `go test ./...` from the repository root by default. If the user specified a package or test name filter, append it (e.g., `go test ./pkg/foo` or `go test ./... -run TestPattern`).
4. If `go` is not on PATH, report and stop.
5. Display the results. `go test` already groups output per package. For each failing test, show the package, test name, and the failure message or panic trace.
6. Summarize: count of packages with passed/failed/skipped tests and total duration.
7. If any tests failed, treat the run as failed.

## What this workflow does NOT do

- Generate or update test fixtures (`*.golden`, snapshot files) without confirmation
- Run race or coverage modes by default — the user adds `-race` or `-cover` explicitly
- Install Go

## Common follow-ups

- Re-run a single failure with `-run "^TestName$"` for fast iteration
- Add `-race` to surface concurrency bugs
- Add `-cover` (or `-coverprofile=`) for coverage output
