#!/usr/bin/env bash
# managed-by: govern
#
# Adopter-side hook installer invoked by /govern.
#
# Idempotent: safe to run repeatedly. Sets core.hooksPath to .githooks/ and
# makes the hook script executable. Skips if core.hooksPath already points
# at a non-.githooks location (the adopter has another hook system) — /govern
# detects this case before invoking this script.

set -euo pipefail
ROOT="$(git rev-parse --show-toplevel 2>/dev/null || pwd)"
cd "$ROOT"

current="$(git config --get core.hooksPath 2>/dev/null || echo "")"
if [ -n "$current" ] && [ "$current" != ".githooks" ]; then
  echo "core.hooksPath is set to '$current' (not .githooks)." >&2
  echo "Refusing to overwrite — adopter has an existing hook system." >&2
  exit 1
fi

git config core.hooksPath .githooks
chmod +x .githooks/pre-commit
echo "Installed govern pre-commit hook (.githooks/pre-commit)"
