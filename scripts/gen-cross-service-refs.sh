#!/usr/bin/env bash
# Regenerate frontmatter `references:` for every feature spec from inline
# body links to cross-service specs (absolute canonical-repo URLs).
#
# Walks tracked specs/NNN-*/spec.md (and spec-and-plan.md), finds inline
# markdown links whose href is an absolute http(s) URL pointing at another
# govern project's spec (a `/<spec-root>/NNN-slug/spec.md` path, where the
# <spec-root> segment is the referenced service's own spec-root name — see
# "Root-aware matching" below), outside fenced code blocks, outside
# blockquote-prefixed lines, and outside any `## See also`
# section (the same navigational opt-out gen-spec-deps.sh honors). Each such
# link is harvested into the derived `references:` frontmatter index, keyed
# {service, spec}:
#
#   * spec    — the `NNN-slug` segment of the URL (the stable identity).
#   * service — the URL's repo (everything before any `/blob/<ref>/` or
#               `/tree/<ref>/` branch segment, which is ignored) matched
#               against `.govern.toml` [services]. A matched repo records the
#               service alias; an unmatched repo records `service: null` (the
#               `unregistered` outcome, surfaced later at resolution time).
#
# Root-aware matching (scenario referenced-service-spec-root). The <spec-root>
# segment is NOT hardcoded to `specs`: a referenced service may rename its own
# spec root (spec 040). Two tiers, decided per registered service:
#   * Checkout reachable — the service is registered in [services] and its
#     local `path` resolves; the matcher reads that checkout's own .govern.toml
#     [paths] specs-root (default `specs`) and accepts only that exact segment.
#   * Checkout unreachable — a registered service that is not checked out, or
#     an unregistered repo, has an unknowable root, so the matcher accepts any
#     single [A-Za-z0-9_-] segment (the `/spec.md` anchor keeps an owner/repo
#     pair that looks like NNN-slug from false-matching). The reference is
#     still harvested, so it never silently drops from the index.
#
# The branch ref is never part of identity, so two links differing only in
# branch resolve to the same reference. `references:` is absent-when-empty:
# a spec with no cross-service links carries no field, and a stale block is
# removed when its last link is deleted.
#
# This index is kept strictly distinct from `dependencies:` — the generator
# never reads or writes that field. Sibling `../NNN-slug/` links are relative,
# never match the absolute-URL predicate, and stay the exclusive domain of
# gen-spec-deps.sh. Body inline links are authoritative; the frontmatter is a
# derived index.

set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"

dry_run=0
staged_only=0
for arg in "$@"; do
  case "$arg" in
    --dry-run) dry_run=1 ;;
    --staged)  staged_only=1 ;;
    --root=*)  ROOT="$(cd "${arg#--root=}" && pwd)" ;;
    -h|--help)
      sed -n '2,29p' "$0" | sed 's/^# \{0,1\}//'
      echo
      echo "Usage: $(basename "$0") [--dry-run] [--staged] [--root=PATH]"
      echo "  --dry-run    Report what would change; exit 1 if any spec needs updating."
      echo "  --staged     Only rewrite specs staged in the git index (the pending"
      echo "               commit), instead of every tracked spec. For pre-commit use."
      echo "  --root=PATH  Run against PATH as the repo root (default: script's parent dir)."
      exit 0
      ;;
    *) echo "Unknown argument: $arg" >&2; exit 2 ;;
  esac
done

shopt -s nullglob

# Resolve the spec-root directory name (spec 040), shared with gen-spec-deps.sh
# via a sourced helper — one definition, no drift (040 review). This resolves
# THIS repo's spec root for enumeration. The cross-service URL matcher below is
# root-aware too, but resolves the *referenced* service's root from its own
# checkout (or accepts any segment when unknowable) — see "Root-aware matching"
# in the header and the registry build below.
# shellcheck source=lib/specs-root.sh
. "$(dirname "$0")/lib/specs-root.sh"
SPECS_ROOT="$(resolve_specs_root)"

# list_specs / staged_specs come from lib/specs-root.sh (sourced above). Each
# spec's `references:` is a pure function of its own body, so --staged needs
# only the staged set — there is no cross-spec graph (unlike gen-spec-deps.sh,
# whose cycle check spans the full list).

# The set of specs to rewrite: only the staged ones under --staged, else all.
enumerate_specs() {
  if [ "$staged_only" -eq 1 ]; then staged_specs; else list_specs; fi
}

# Parse .govern.toml [services] into "normalized-repo<TAB>alias<TAB>root"
# records — the registry the harvest matcher consults. A missing file or absent
# [services] table yields an empty registry (every reference then resolves to
# `unregistered`).
#
# `root` is each service's resolved spec-root name: when its local checkout is
# reachable, read from that checkout's own .govern.toml (default `specs`); when
# the service is not checked out, left empty — the tier where the matcher
# accepts any spec-root segment (scenario referenced-service-spec-root, Q1). The
# two-step parse — awk extracts alias/repo/path, then bash resolves the
# per-service root — keeps the filesystem probe out of awk.
reg_file="$(mktemp)"
trap 'rm -f "$reg_file"' EXIT
if [ -f "$ROOT/.govern.toml" ]; then
  svc_raw="$(mktemp)"
  awk '
    function norm(v) { sub(/\/+$/, "", v); sub(/\.git$/, "", v); return v }
    function unq(v) {
      sub(/^[^=]*=[[:space:]]*/, "", v)
      if (substr(v, 1, 1) == "\"") { v = substr(v, 2); sub(/".*$/, "", v) }
      else { sub(/[[:space:]].*$/, "", v) }
      return v
    }
    function flush() { if (cur != "" && repo != "") print cur "\t" norm(repo) "\t" path }
    /^\[services\.[^]]+\][[:space:]]*$/ {
      flush()
      alias = $0
      sub(/^\[services\./, "", alias); sub(/\][[:space:]]*$/, "", alias)
      gsub(/^"|"$/, "", alias)
      cur = alias; repo = ""; path = ""; in_svc = 1; next
    }
    /^\[/ { flush(); cur = ""; repo = ""; path = ""; in_svc = 0; next }
    in_svc && /^[[:space:]]*repo[[:space:]]*=/ { repo = unq($0); next }
    in_svc && /^[[:space:]]*path[[:space:]]*=/ { path = unq($0); next }
    END { flush() }
  ' "$ROOT/.govern.toml" > "$svc_raw"

  # Resolve each service's spec-root from its local checkout when reachable;
  # otherwise leave it empty (the not-checked-out / permissive tier). `path`
  # is relative to the repo root or absolute (030 plan, D1).
  while IFS=$'\t' read -r alias repo path; do
    [ -n "$repo" ] || continue
    root=""
    if [ -n "$path" ]; then
      case "$path" in
        /*) checkout="$path" ;;
        *)  checkout="$ROOT/$path" ;;
      esac
      [ -d "$checkout" ] && root="$(specs_root_of "$checkout/.govern.toml")"
    fi
    printf '%s\t%s\t%s\n' "$repo" "$alias" "$root" >> "$reg_file"
  done < "$svc_raw"
  rm -f "$svc_raw"
fi

# Harvest (service<TAB>spec) records — sorted, deduped — from one spec body.
harvest() {
  awk -v REG="$reg_file" '
    function norm(v) { sub(/\/+$/, "", v); sub(/\.git$/, "", v); return v }
    function strip_ref(b) {
      # Drop a trailing /blob/<ref>, /tree/<ref>, or /-/blob|tree/<ref> branch
      # segment so branch-ref variations collapse to the same repo identity.
      # String-form regex: a "/" inside a bracket expression terminates an
      # awk /.../ literal under BSD awk, so the pattern is a dynamic regex.
      if (match(b, "/(-/)?(blob|tree)/[^/]+$")) b = substr(b, 1, RSTART - 1)
      return b
    }
    BEGIN {
      # Registry lines are "repo<TAB>alias<TAB>root" (root empty when the
      # service is not checked out). reg_alias keys the registered repos;
      # reg_root[repo] is the resolved spec-root, or "" for the permissive tier.
      while ((getline line < REG) > 0) {
        t1 = index(line, "\t")
        if (t1 == 0) continue
        repo = substr(line, 1, t1 - 1)
        rest = substr(line, t1 + 1)
        t2 = index(rest, "\t")
        if (t2 > 0) {
          reg_alias[repo] = substr(rest, 1, t2 - 1)
          reg_root[repo] = substr(rest, t2 + 1)
        } else {
          reg_alias[repo] = rest
          reg_root[repo] = ""
        }
      }
      close(REG)
      fm_seen = 0; in_fm = 0; in_fence = 0; in_see_also = 0; n = 0
    }
    /^---[[:space:]]*$/ {
      if (!fm_seen) { in_fm = 1; fm_seen = 1; next }
      if (in_fm)    { in_fm = 0; next }
    }
    in_fm { next }
    /^[[:space:]]*```/ { in_fence = !in_fence; next }
    in_fence { next }
    /^[[:space:]]*>/ { next }
    /^#+[[:space:]]/ {
      hashes = 0
      for (i = 1; i <= length($0); i++) {
        if (substr($0, i, 1) == "#") hashes++
        else break
      }
      if (hashes <= 2) {
        htxt = substr($0, hashes + 1)
        sub(/^[[:space:]]+/, "", htxt); sub(/[[:space:]]+$/, "", htxt)
        in_see_also = (tolower(htxt) == "see also") ? 1 : 0
      }
    }
    in_see_also { next }
    {
      line = $0
      # Strip inline-code spans before scanning: a link wrapped in backticks
      # renders as literal text, not a clickable link, so by the spec it is an
      # illustrative example, not a reference.
      gsub(/`[^`]*`/, "", line)
      while (match(line, /\]\(https?:\/\/[^)[:space:]]+\)/)) {
        m = substr(line, RSTART, RLENGTH)
        line = substr(line, RSTART + RLENGTH)
        url = substr(m, 3, length(m) - 3)
        # Candidate spec link: a single well-formed path segment (the referenced
        # service spec-root name) immediately before an NNN-slug spec dir and
        # its spec(-and-plan).md file. The leading segment is a wildcard so a
        # renamed referenced root still matches; the /spec.md anchor keeps an
        # owner/repo pair that looks like NNN-slug from false-matching.
        if (match(url, /\/[A-Za-z0-9_-]+\/[0-9][0-9][0-9]-[a-z0-9-]+\/spec(-and-plan)?\.md/)) {
          seg = substr(url, RSTART, RLENGTH)
          before = substr(url, 1, RSTART - 1)
          root_seg = seg; sub(/^\//, "", root_seg); sub(/\/.*$/, "", root_seg)
          slug = seg; sub(/^\/[A-Za-z0-9_-]+\//, "", slug); sub(/\/spec(-and-plan)?\.md$/, "", slug)
          repo = norm(strip_ref(before))
          if (repo in reg_alias) {
            svc = reg_alias[repo]
            # Checkout reachable (root known): accept only the real resolved
            # spec-root segment. Not checked out (root ""): accept any segment.
            if (reg_root[repo] != "" && root_seg != reg_root[repo]) continue
          } else {
            svc = ""    # unregistered: spec-root unknowable, accept any segment
          }
          key = slug SUBSEP svc
          if (!(key in seen)) { seen[key] = 1; n++; ks[n] = slug; vs[n] = svc }
        }
      }
    }
    END {
      for (i = 2; i <= n; i++) {
        s = ks[i]; v = vs[i]; j = i - 1
        while (j > 0 && (ks[j] > s || (ks[j] == s && vs[j] > v))) {
          ks[j + 1] = ks[j]; vs[j + 1] = vs[j]; j--
        }
        ks[j + 1] = s; vs[j + 1] = v
      }
      # `|` (a non-whitespace delimiter) keeps an empty service field intact
      # when the bash reader splits — a tab would be trimmed as IFS whitespace.
      for (i = 1; i <= n; i++) print vs[i] "|" ks[i]
    }
  ' "$1"
}

changed=0
while IFS= read -r spec; do
  [ -f "$spec" ] || continue

  # Build the desired `references:` block (empty when the spec has no refs).
  block_file="$(mktemp)"
  : > "$block_file"
  records="$(harvest "$spec")"
  if [ -n "$records" ]; then
    {
      echo "references:"
      while IFS='|' read -r svc slug; do
        [ -n "$slug" ] || continue
        if [ -n "$svc" ]; then
          echo "  - service: $svc"
        else
          echo "  - service: null"
        fi
        echo "    spec: $slug"
      done <<< "$records"
    } > "$block_file"
  fi

  # Splice the block into the frontmatter: strip any existing `references:`
  # block, then re-insert the desired block (when non-empty) right after the
  # `dependencies:` line — or before the closing `---` if there is none.
  tmp="$(mktemp)"
  awk -v BLOCK="$block_file" '
    function emit_block(   l) {
      if (has_block) { while ((getline l < BLOCK) > 0) print l; close(BLOCK) }
    }
    BEGIN {
      fm_seen = 0; in_fm = 0; skipping = 0; inserted = 0
      # has_block: BLOCK file is non-empty.
      has_block = 0
      if ((getline probe < BLOCK) > 0) has_block = 1
      close(BLOCK)
    }
    {
      line = $0
      if (line ~ /^---[[:space:]]*$/) {
        if (!fm_seen) { fm_seen = 1; in_fm = 1; print line; next }
        if (in_fm) {
          if (has_block && !inserted) { emit_block(); inserted = 1 }
          in_fm = 0; skipping = 0; print line; next
        }
        print line; next
      }
      if (in_fm) {
        if (skipping) {
          if (line ~ /^[[:space:]]/) next
          skipping = 0
        }
        if (line ~ /^references:/) { skipping = 1; next }
        if (!inserted && has_block && line ~ /^dependencies:/) {
          print line; emit_block(); inserted = 1; next
        }
      }
      print line
    }
  ' "$spec" > "$tmp"

  rm -f "$block_file"

  if ! cmp -s "$spec" "$tmp"; then
    if [ "$dry_run" -eq 1 ]; then
      echo "Would update $spec"
      rm -f "$tmp"
    else
      mv "$tmp" "$spec"
      echo "Updated $spec"
    fi
    changed=$((changed + 1))
  else
    rm -f "$tmp"
  fi
done < <(enumerate_specs)

if [ "$changed" -eq 0 ]; then
  echo "No changes (all references in sync)"
fi

if [ "$changed" -gt 0 ] && [ "$dry_run" -eq 1 ]; then
  exit 1
fi
