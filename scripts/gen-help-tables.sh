#!/usr/bin/env bash
# Regenerate the five command-group tables in framework/commands/help.md
# from each command's frontmatter `description:`.
#
# Marker pairs:
#   <!-- generated:commands-pipeline:{start,end} -->
#   <!-- generated:commands-refine:{start,end} -->
#   <!-- generated:commands-brownfield:{start,end} -->
#   <!-- generated:commands-orient:{start,end} -->
#   <!-- generated:commands-bootstrap:{start,end} -->
#
# Pipeline group has an extra "Pipeline Gate" column (gate values are
# static pipeline facts hardcoded below). All other groups are
# (Command, Description) two-column tables.
#
# Exits non-zero if any expected marker is missing or any referenced
# command source file is absent.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
HELP="$ROOT/framework/commands/help.md"

dry_run=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) dry_run=1 ;;
    -h|--help)
      sed -n '2,18p' "$0" | sed 's/^# \{0,1\}//'
      echo
      echo "Usage: $(basename "$0") [--dry-run]"
      echo "  --dry-run  Report what would change; exit 1 if help.md needs updating."
      exit 0
      ;;
    *) echo "Unknown argument: $arg" >&2; exit 2 ;;
  esac
done

# Read the `description:` frontmatter field from a markdown file.
read_description() {
  local file="$1"
  if [ ! -f "$file" ]; then
    echo "Missing source file: $file" >&2
    exit 4
  fi
  awk '
    BEGIN { fm_seen = 0; in_fm = 0 }
    /^---[[:space:]]*$/ {
      if (!fm_seen) { in_fm = 1; fm_seen = 1; next }
      if (in_fm)    { in_fm = 0; exit }
    }
    in_fm && /^description:[[:space:]]/ {
      sub(/^description:[[:space:]]*/, "", $0)
      # Strip surrounding quotes if present
      gsub(/^"|"$/, "", $0)
      print $0
      exit
    }
  ' "$file"
}

# Build a two-column table: Command | Description
build_two_col_table() {
  printf '| Command | Description |\n'
  printf '| --- | --- |\n'
  while [ $# -gt 0 ]; do
    local label="$1"; shift
    local source="$1"; shift
    local desc
    desc="$(read_description "$source")"
    printf '| `%s` | %s |\n' "$label" "$desc"
  done
}

# Build the pipeline table: Command | Pipeline Gate | Description
build_pipeline_table() {
  printf '| Command | Pipeline Gate | Description |\n'
  printf '| --- | --- | --- |\n'
  while [ $# -gt 0 ]; do
    local label="$1"; shift
    local gate="$1"; shift
    local source="$1"; shift
    local desc
    desc="$(read_description "$source")"
    printf '| `%s` | %s | %s |\n' "$label" "$gate" "$desc"
  done
}

CMD_DIR="$ROOT/framework/commands"
BOOTSTRAP_DIR="$ROOT/framework/bootstrap"

pipeline_table="$(build_pipeline_table \
  '/{project}:specify'   '→ draft'                            "$CMD_DIR/specify.md" \
  '/{project}:clarify'   'draft → clarified'                  "$CMD_DIR/clarify.md" \
  '/{project}:plan'      'clarified → planned'                "$CMD_DIR/plan.md" \
  '/{project}:implement' 'planned → in-progress → done'       "$CMD_DIR/implement.md" \
  '/{project}:review'    'blocks `done` (MUST violations)'    "$CMD_DIR/review.md" \
  '/{project}:analyze'   '—'                                  "$CMD_DIR/analyze.md" \
)"

refine_table="$(build_two_col_table \
  '/{project}:ask' "$CMD_DIR/ask.md" \
)"

brownfield_table="$(build_two_col_table \
  '/{project}:log'   "$CMD_DIR/log.md" \
  '/{project}:groom' "$CMD_DIR/groom.md" \
)"

orient_table="$(build_two_col_table \
  '/{project}:target' "$CMD_DIR/target.md" \
  '/{project}:link'   "$CMD_DIR/link.md" \
  '/{project}:status' "$CMD_DIR/status.md" \
  '/{project}:help'   "$CMD_DIR/help.md" \
)"

bootstrap_table="$(build_two_col_table \
  '/govern'              "$BOOTSTRAP_DIR/govern.md" \
  '/{project}:configure' "$BOOTSTRAP_DIR/configure/claude.md" \
)"

# Splice each table between its markers. Fail if any marker is missing.
splice() {
  local marker_name="$1"
  local table_file="$2"
  local file="$3"
  if ! grep -q "<!-- generated:${marker_name}:start -->" "$file"; then
    echo "Missing marker <!-- generated:${marker_name}:start --> in $file" >&2
    return 5
  fi
  if ! grep -q "<!-- generated:${marker_name}:end -->" "$file"; then
    echo "Missing marker <!-- generated:${marker_name}:end --> in $file" >&2
    return 5
  fi
  awk -v marker="$marker_name" -v table_file="$table_file" '
    $0 ~ ("<!-- generated:" marker ":start -->") {
      print
      print ""
      while ((getline line < table_file) > 0) print line
      close(table_file)
      print ""
      in_block = 1
      next
    }
    $0 ~ ("<!-- generated:" marker ":end -->") {
      in_block = 0
      print
      next
    }
    !in_block { print }
  ' "$file"
}

tmp="$(mktemp)"
cp "$HELP" "$tmp"

# Write each table to its own temp file so awk can read multi-line content via getline.
write_table() {
  local f
  f="$(mktemp)"
  printf '%s\n' "$1" > "$f"
  echo "$f"
}

p_file="$(write_table "$pipeline_table")"
r_file="$(write_table "$refine_table")"
b_file="$(write_table "$brownfield_table")"
o_file="$(write_table "$orient_table")"
boot_file="$(write_table "$bootstrap_table")"

for pair in \
  "commands-pipeline|$p_file" \
  "commands-refine|$r_file" \
  "commands-brownfield|$b_file" \
  "commands-orient|$o_file" \
  "commands-bootstrap|$boot_file"
do
  marker="${pair%%|*}"
  table_file="${pair#*|}"
  next_tmp="$(mktemp)"
  if ! splice "$marker" "$table_file" "$tmp" > "$next_tmp"; then
    rm "$tmp" "$next_tmp" "$p_file" "$r_file" "$b_file" "$o_file" "$boot_file"
    exit 5
  fi
  mv "$next_tmp" "$tmp"
done

rm "$p_file" "$r_file" "$b_file" "$o_file" "$boot_file"

if cmp -s "$HELP" "$tmp"; then
  rm "$tmp"
  echo "No changes (help.md in sync)"
  exit 0
fi

if [ "$dry_run" -eq 1 ]; then
  rm "$tmp"
  echo "Would update $HELP"
  exit 1
fi

mv "$tmp" "$HELP"
echo "Updated $HELP"
