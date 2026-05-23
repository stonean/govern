#!/usr/bin/env bash
# lint-procedure-parseability.sh — invoke `runtime parse --check` on every
# slash command markdown file, honoring runtime/legacy-prose-commands.txt
# as the allowlist of files not yet rewritten to the new conventions.
#
# Workflow-local: this script builds the runtime binary in --release mode
# at runtime/target/release/gvrn and invokes it via that relative
# path. It does NOT add the binary to PATH, so the opt-in invariant
# check (step (a) in markdown-only-pipeline.yml) remains intact — the
# parseability check is a workflow-private compile, not a runtime
# install.
#
# Exit codes:
#   0  — every file is either parseable as a Procedure OR present in
#        the legacy allowlist
#   1  — at least one file failed to parse AND is not in the allowlist
#   2  — internal error (binary build failure, missing inputs, etc.)

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
runtime_dir="$repo_root/runtime"
runtime_bin="$runtime_dir/target/release/gvrn"
allowlist="$runtime_dir/legacy-prose-commands.txt"

if [[ ! -d "$runtime_dir" ]]; then
  echo "::error::runtime/ not found at $runtime_dir" >&2
  exit 2
fi

if [[ ! -f "$allowlist" ]]; then
  echo "::error::legacy-prose allowlist not found at $allowlist" >&2
  exit 2
fi

# Always invoke `cargo build --release` so cargo's incremental check
# rebuilds when source/Cargo.lock changed but the cached binary is stale.
# Cargo itself is a no-op when nothing changed (cache-warm CI runs stay
# fast); skipping the build only when the binary file exists missed the
# case where the restore-keys fallback in CI brought back an older
# binary from a prior runtime version (#26342549657 — stale 0.8.1
# binary failed to parse a 0.9.0 procedure that referenced
# write-session).
(cd "$runtime_dir" && cargo build --release --quiet) || {
  echo "::error::cargo build --release failed" >&2
  exit 2
}

# Load allowlist into an associative-array-friendly grep pattern.
allow_paths=()
while IFS= read -r raw; do
  line="${raw%%#*}"
  line="${line%%[[:space:]]*}"
  [[ -z "$line" ]] && continue
  allow_paths+=("$line")
done < "$allowlist"

is_allowed() {
  local target="$1"
  local entry
  for entry in "${allow_paths[@]}"; do
    if [[ "$target" == "$entry" ]]; then
      return 0
    fi
  done
  return 1
}

shopt -s nullglob
command_files=("$repo_root"/framework/commands/*.md)
if [[ ${#command_files[@]} -eq 0 ]]; then
  echo "::error::no framework/commands/*.md files found" >&2
  exit 2
fi

fail=0
for abs_path in "${command_files[@]}"; do
  rel_path="${abs_path#"$repo_root/"}"
  set +e
  output=$("$runtime_bin" parse --check "$abs_path" 2>&1)
  rc=$?
  set -e
  case "$rc" in
    0)
      # Parseable, or legacy-prose (--check exits 0 for either). When the
      # file is in the allowlist, allow the legacy case explicitly; when
      # it isn't, accept the result either way and let the parser's
      # own rules govern.
      :
      ;;
    *)
      if is_allowed "$rel_path"; then
        echo "::notice::$rel_path is on the legacy allowlist — skipping ($output)"
      else
        echo "::error::$rel_path failed parseability check"
        echo "$output" | sed 's/^/  /'
        fail=1
      fi
      ;;
  esac
done

exit "$fail"
