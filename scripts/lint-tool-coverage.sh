#!/usr/bin/env bash
# Verify every reference to a runtime tool in framework/commands/*.md
# is paired with a graceful-fallback marker within 20 lines.
#
# Tool names are read from framework/runtime-tools.txt (one per non-blank,
# non-comment line; case-sensitive exact match). Fallback markers are
# any case-insensitive occurrence of: Otherwise, Fallback, If unavailable,
# markdown-only path. The proximity scan trades false-positive risk for a
# fully derived check — no author-supplied markers required.
#
# Tool references inside the `## Markdown-only reference` section of a
# command file are skipped — that section *is* the fallback, so any
# mention there does not require a paired fallback marker.
#
# Source of truth: framework/constitution.md §runtime-boundary
# Consumed by: .github/workflows/markdown-only-pipeline.yml

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

WINDOW=20
MANIFEST="$ROOT/framework/runtime-tools.txt"

for arg in "$@"; do
  case "$arg" in
    -h|--help)
      sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
      echo
      echo "Usage: $(basename "$0")"
      echo "  Exits 0 when every runtime-tool reference has a fallback within $WINDOW lines."
      echo "  Exits 1 when any reference lacks a fallback (errors printed to stdout)."
      exit 0
      ;;
    *) echo "Unknown argument: $arg" >&2; exit 2 ;;
  esac
done

if [ ! -f "$MANIFEST" ]; then
  echo "Manifest not found: $MANIFEST" >&2
  exit 2
fi

# Load tool names: skip blank lines and lines starting with '#'.
# Strip a full run of leading/trailing whitespace (spaces or tabs) so a
# stray indent in the manifest can't slip through with embedded
# whitespace that would silently fail to match in prose.
tools=()
while IFS= read -r raw; do
  line="${raw%%#*}"
  line="${line#"${line%%[![:space:]]*}"}"
  line="${line%"${line##*[![:space:]]}"}"
  [ -z "$line" ] && continue
  tools+=("$line")
done < "$MANIFEST"

if [ "${#tools[@]}" -eq 0 ]; then
  exit 0
fi

shopt -s nullglob
errors=0
for cmd in "$ROOT"/framework/commands/*.md; do
  # Find the line of the `## Markdown-only reference` heading, if any. Any
  # tool reference at or after that line is itself part of the fallback
  # path and does not need a paired fallback marker.
  md_only_start="$(grep -n '^## Markdown-only reference' "$cmd" | head -n1 | cut -d: -f1 || true)"
  for tool in "${tools[@]}"; do
    while IFS=: read -r lineno _; do
      [ -z "$lineno" ] && continue
      if [ -n "$md_only_start" ] && [ "$lineno" -ge "$md_only_start" ]; then
        continue
      fi
      end=$((lineno + WINDOW))
      window="$(sed -n "${lineno},${end}p" "$cmd")"
      if ! printf '%s\n' "$window" | grep -qiE 'Otherwise|Fallback|If unavailable|markdown-only path'; then
        echo "${cmd#"$ROOT"/}:${lineno}: missing fallback for tool '${tool}'"
        errors=$((errors + 1))
      fi
    done < <(grep -nF -- "$tool" "$cmd" || true)
  done
done

if [ "$errors" -gt 0 ]; then
  exit 1
fi
exit 0
