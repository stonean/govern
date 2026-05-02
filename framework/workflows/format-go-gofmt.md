# gofmt

Format the Go codebase with gofmt.

## Instructions

1. Verify a `go.mod` exists at the repository root. If not, report `Not a Go module` and stop.
2. Run `gofmt -l .` from the repository root by default. The `-l` flag lists files that differ from gofmt's output without modifying them. If the user explicitly asked to write changes, run `gofmt -w .` instead.
3. If `gofmt` is not on PATH, try `go fmt ./...` as a fallback (it wraps gofmt and respects module boundaries).
4. Display the results. For check mode, list each file that needs formatting. For write mode, summarize the count of files changed.
5. If check mode lists any files, treat the run as failed.

## What this workflow does NOT do

- Reformat files without an explicit user request — default mode is `-l`
- Run `goimports` (which also reorders imports). Use a separate workflow for that.
- Install Go

## Common follow-ups

- After review, re-run with `gofmt -w .` to apply the formatting
- Diff a single file: `gofmt -d path/to/file.go`
- Switch to `goimports` if the project also wants import grouping
