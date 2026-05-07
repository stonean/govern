#!/usr/bin/env bash
# Install govern repo's git hooks by setting core.hooksPath.
#
# Idempotent: safe to run repeatedly. The actual hook scripts live in
# .githooks/ and are part of the repo.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

current="$(git config --get core.hooksPath || echo "")"
if [ "$current" = ".githooks" ]; then
  echo "core.hooksPath already set to .githooks (no change)"
  exit 0
fi

if [ -n "$current" ] && [ "$current" != ".githooks" ]; then
  echo "Warning: core.hooksPath is currently set to '$current'." >&2
  echo "Overwriting with '.githooks'." >&2
fi

git config core.hooksPath .githooks
echo "Set core.hooksPath = .githooks"

# Make sure hook scripts are executable.
chmod +x "$ROOT/.githooks/pre-commit"
echo "Hooks installed. The next commit will run all generators."
