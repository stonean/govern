#!/usr/bin/env bash
# Test surface for .govern/scripts/gen-cross-service-refs.sh.
#
# Builds tiny fixture spec trees (plus a fixture .govern.toml [services]
# registry) under temp dirs, runs the generator against them via the
# `--root=PATH` flag, and asserts on the resulting frontmatter `references:`
# block (or its absence).
#
# Coverage (030 cross-service-references, task 3 done-when + edges):
#   A. a registered link is harvested with its service alias and spec slug
#   B. an unregistered link is recorded with `service: null`
#   C. a link under `## See also` is excluded (no references field)
#   D. branch-ref variations of one target collapse to a single reference
#   E. `dependencies:` is never touched by this generator
#   F. an inline-code (backtick-wrapped) example link is excluded
#   G. a link inside a fenced code block is excluded
#   H. a link on a blockquote line is excluded
#   I. running twice produces no diff (idempotence)
#   J. a spec with no cross-service links carries no references field, and a
#      stale block is removed when its last link is deleted
#   K. --staged rewrites only specs staged in the git index; an unstaged spec
#      whose references have drifted is left untouched (adopter pre-commit path)
#   L. a consumer under a renamed spec root (spec 040) still harvests refs
#
# Root-aware cross-service matching (spec 030 task 13, scenario
# referenced-service-spec-root — the *referenced* service renames its root):
#   M. referenced service checked out under a renamed root → link harvested
#      (tier-1 reads the checkout's own .govern.toml [paths] specs-root)
#   N. checked-out service, body link uses the wrong spec-root segment → not
#      harvested (tier-1 exact match against the resolved root)
#   O. registered but not-checked-out service, renamed-root link → harvested
#      via the permissive fallback (tier-2, root unknowable)
#
# Usage: scripts/tests/test-gen-cross-service-refs.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
GEN="$REPO_ROOT/.govern/scripts/gen-cross-service-refs.sh"

failures=0
pass() { printf '  PASS  %s\n' "$1"; }
fail() { printf '  FAIL  %s\n' "$1" >&2; failures=$((failures + 1)); }

make_fixture() { mktemp -d; }

# Write a minimal spec.md from stdin into the fixture.
write_spec() {
  local tmp="$1" slug="$2"
  mkdir -p "$tmp/specs/$slug"
  cat > "$tmp/specs/$slug/spec.md"
}

# Write a fixture .govern.toml from stdin (the [services] registry).
write_govern_toml() {
  local tmp="$1"
  cat > "$tmp/.govern.toml"
}

# Print the YAML frontmatter region (lines strictly between the first and
# second `---`), so assertions never match body prose.
frontmatter() {
  awk 'BEGIN{n=0} /^---[[:space:]]*$/{n++; next} n==1{print} n>=2{exit}' "$1"
}

fm_has() {
  local file="$1" needle="$2" label="$3"
  if frontmatter "$file" | grep -qF -- "$needle"; then
    pass "$label"
  else
    fail "$label: frontmatter missing '$needle' (got: $(frontmatter "$file" | tr '\n' '|'))"
  fi
}

fm_lacks() {
  local file="$1" needle="$2" label="$3"
  if frontmatter "$file" | grep -qF -- "$needle"; then
    fail "$label: frontmatter unexpectedly contained '$needle'"
  else
    pass "$label"
  fi
}

# ---------- A: registered link harvested ----------

test_A_registered_link_harvested() {
  local tmp; tmp="$(make_fixture)"
  write_govern_toml "$tmp" <<'EOF'
[services.api]
repo = "https://github.com/acme/api"
path = "../api"
EOF
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

Uses the [api user model](https://github.com/acme/api/blob/main/specs/003-user/spec.md).
EOF
  "$GEN" --root="$tmp" > /dev/null
  local f="$tmp/specs/001-alpha/spec.md"
  fm_has "$f" "references:" "A: references field added"
  fm_has "$f" "- service: api" "A: registered service alias recorded"
  fm_has "$f" "spec: 003-user" "A: spec slug recorded"
  rm -rf "$tmp"
}

# ---------- B: unregistered link → null service ----------

test_B_unregistered_link_null_service() {
  local tmp; tmp="$(make_fixture)"
  write_govern_toml "$tmp" <<'EOF'
[services.api]
repo = "https://github.com/acme/api"
path = "../api"
EOF
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

References [orders](https://github.com/other/billing/blob/main/specs/004-orders/spec.md).
EOF
  "$GEN" --root="$tmp" > /dev/null
  local f="$tmp/specs/001-alpha/spec.md"
  fm_has "$f" "- service: null" "B: unregistered repo recorded with null service"
  fm_has "$f" "spec: 004-orders" "B: unregistered spec slug recorded"
  rm -rf "$tmp"
}

# ---------- C: ## See also excluded ----------

test_C_see_also_excluded() {
  local tmp; tmp="$(make_fixture)"
  write_govern_toml "$tmp" <<'EOF'
[services.api]
repo = "https://github.com/acme/api"
path = "../api"
EOF
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

## See also

- [api user](https://github.com/acme/api/blob/main/specs/003-user/spec.md)
EOF
  "$GEN" --root="$tmp" > /dev/null
  fm_lacks "$tmp/specs/001-alpha/spec.md" "references:" "C: ## See also link is not harvested"
  rm -rf "$tmp"
}

# ---------- D: branch-ref variations → single identity ----------

test_D_branch_ref_same_identity() {
  local tmp; tmp="$(make_fixture)"
  write_govern_toml "$tmp" <<'EOF'
[services.api]
repo = "https://github.com/acme/api"
path = "../api"
EOF
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

On main: [a](https://github.com/acme/api/blob/main/specs/003-user/spec.md).
On develop: [b](https://github.com/acme/api/blob/develop/specs/003-user/spec.md).
EOF
  "$GEN" --root="$tmp" > /dev/null
  local f="$tmp/specs/001-alpha/spec.md" count
  count="$(frontmatter "$f" | grep -cF 'spec: 003-user' || true)"
  if [ "$count" -eq 1 ]; then
    pass "D: branch-ref variations collapse to one reference"
  else
    fail "D: expected 1 reference for 003-user, got $count"
  fi
  rm -rf "$tmp"
}

# ---------- E: dependencies untouched ----------

test_E_dependencies_untouched() {
  local tmp; tmp="$(make_fixture)"
  write_govern_toml "$tmp" <<'EOF'
[services.api]
repo = "https://github.com/acme/api"
path = "../api"
EOF
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: [002-beta]
---

# Alpha

Sibling [002-beta](../002-beta/spec.md) and cross-service
[api](https://github.com/acme/api/blob/main/specs/003-user/spec.md).
EOF
  "$GEN" --root="$tmp" > /dev/null
  local f="$tmp/specs/001-alpha/spec.md" deps
  deps="$(awk '/^dependencies:/{print; exit}' "$f")"
  if [ "$deps" = "dependencies: [002-beta]" ]; then
    pass "E: dependencies: line left untouched"
  else
    fail "E: dependencies changed to '$deps'"
  fi
  fm_has "$f" "spec: 003-user" "E: references still harvested alongside dependencies"
  rm -rf "$tmp"
}

# ---------- F: inline-code example excluded ----------

test_F_inline_code_excluded() {
  local tmp; tmp="$(make_fixture)"
  write_govern_toml "$tmp" <<'EOF'
[services.api]
repo = "https://github.com/acme/api"
path = "../api"
EOF
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

For example `[api user](https://github.com/acme/api/blob/main/specs/003-user/spec.md)` declares a reference.
EOF
  "$GEN" --root="$tmp" > /dev/null
  fm_lacks "$tmp/specs/001-alpha/spec.md" "references:" "F: backtick-wrapped example link is not harvested"
  rm -rf "$tmp"
}

# ---------- G: fenced code block excluded ----------

test_G_code_fence_excluded() {
  local tmp; tmp="$(make_fixture)"
  write_govern_toml "$tmp" <<'EOF'
[services.api]
repo = "https://github.com/acme/api"
path = "../api"
EOF
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

```
See [api](https://github.com/acme/api/blob/main/specs/003-user/spec.md) example.
```
EOF
  "$GEN" --root="$tmp" > /dev/null
  fm_lacks "$tmp/specs/001-alpha/spec.md" "references:" "G: fenced-block link is not harvested"
  rm -rf "$tmp"
}

# ---------- H: blockquote excluded ----------

test_H_blockquote_excluded() {
  local tmp; tmp="$(make_fixture)"
  write_govern_toml "$tmp" <<'EOF'
[services.api]
repo = "https://github.com/acme/api"
path = "../api"
EOF
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

> Signpost: [api](https://github.com/acme/api/blob/main/specs/003-user/spec.md)
EOF
  "$GEN" --root="$tmp" > /dev/null
  fm_lacks "$tmp/specs/001-alpha/spec.md" "references:" "H: blockquote link is not harvested"
  rm -rf "$tmp"
}

# ---------- I: idempotence ----------

test_I_idempotent() {
  local tmp; tmp="$(make_fixture)"
  write_govern_toml "$tmp" <<'EOF'
[services.api]
repo = "https://github.com/acme/api"
path = "../api"
EOF
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

Uses [api](https://github.com/acme/api/blob/main/specs/003-user/spec.md).
EOF
  "$GEN" --root="$tmp" > /dev/null
  local f="$tmp/specs/001-alpha/spec.md" first second
  first="$(cat "$f")"
  "$GEN" --root="$tmp" > /dev/null
  second="$(cat "$f")"
  if [ "$first" = "$second" ]; then
    pass "I: idempotent — second run produces no diff"
  else
    fail "I: second run produced a diff"
  fi
  rm -rf "$tmp"
}

# ---------- J: absent-when-empty / stale block removed ----------

test_J_stale_block_removed() {
  local tmp; tmp="$(make_fixture)"
  write_govern_toml "$tmp" <<'EOF'
[services.api]
repo = "https://github.com/acme/api"
path = "../api"
EOF
  # The spec carries a stale references block but its body has no
  # cross-service links — the generator must strip the block.
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
references:
  - service: api
    spec: 003-user
---

# Alpha

No cross-service links here.
EOF
  "$GEN" --root="$tmp" > /dev/null
  local f="$tmp/specs/001-alpha/spec.md"
  fm_lacks "$f" "references:" "J: stale references block removed when no links remain"
  fm_has "$f" "dependencies: []" "J: dependencies preserved while removing references"
  rm -rf "$tmp"
}

# ---------- K: --staged scopes the rewrite to staged specs ----------

test_K_staged_scopes_rewrite() {
  local tmp; tmp="$(make_fixture)"
  git -C "$tmp" init -q
  git -C "$tmp" config user.email t@t
  git -C "$tmp" config user.name t
  write_govern_toml "$tmp" <<'EOF'
[services.api]
repo = "https://github.com/acme/api"
path = "../api"
EOF
  # beta: committed; body links api but frontmatter has no references (drift).
  write_spec "$tmp" "002-beta" <<'EOF'
---
status: clarified
dependencies: []
---

# Beta

Uses the [api model](https://github.com/acme/api/blob/main/specs/003-user/spec.md).
EOF
  git -C "$tmp" add -A
  git -C "$tmp" commit -qm init
  # alpha: new, staged, carries its own cross-service link.
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

Uses the [api model](https://github.com/acme/api/blob/main/specs/003-user/spec.md).
EOF
  git -C "$tmp" add specs/001-alpha

  "$GEN" --staged --root="$tmp" > /dev/null

  local a="$tmp/specs/001-alpha/spec.md" b="$tmp/specs/002-beta/spec.md"
  fm_has  "$a" "spec: 003-user" "K: staged spec is rewritten under --staged"
  fm_lacks "$b" "references:"   "K: unstaged drifted spec is left untouched"
  rm -rf "$tmp"
}

# ---------- L: configurable spec-root (spec 040) ----------

test_L_configured_specs_root() {
  local tmp; tmp="$(make_fixture)"
  # Consumer renames its spec root to `governance`; the registered service URL
  # keeps its own `specs/` layout (a different repo's convention, unchanged).
  write_govern_toml "$tmp" <<'EOF'
[paths]
specs-root = "governance"

[services.api]
repo = "https://github.com/acme/api"
path = "../api"
EOF
  mkdir -p "$tmp/governance/001-alpha"
  cat > "$tmp/governance/001-alpha/spec.md" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

Uses the [api user model](https://github.com/acme/api/blob/main/specs/003-user/spec.md).
EOF
  "$GEN" --root="$tmp" > /dev/null
  local f="$tmp/governance/001-alpha/spec.md"
  fm_has "$f" "references:" "L: references harvested from a spec under renamed root"
  fm_has "$f" "spec: 003-user" "L: target spec slug recorded under renamed root"
  rm -rf "$tmp"
}

# ---------- M: referenced service renamed its root, checkout reachable ----------

test_M_referenced_renamed_root_checked_out() {
  local tmp; tmp="$(make_fixture)"
  # The referenced service `api` is checked out locally and has renamed its own
  # spec root to `governance`; its canonical URLs therefore carry
  # `/governance/NNN-slug/`. Tier-1 (checkout reachable): the matcher reads the
  # checkout's .govern.toml and harvests the renamed-root link.
  write_govern_toml "$tmp" <<'EOF'
[services.api]
repo = "https://github.com/acme/api"
path = "checkouts/api"
EOF
  mkdir -p "$tmp/checkouts/api"
  cat > "$tmp/checkouts/api/.govern.toml" <<'EOF'
[paths]
specs-root = "governance"
EOF
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

Uses the [api user model](https://github.com/acme/api/blob/main/governance/003-user/spec.md).
EOF
  "$GEN" --root="$tmp" > /dev/null
  local f="$tmp/specs/001-alpha/spec.md"
  fm_has "$f" "- service: api" "M: renamed-root reference harvested under service alias"
  fm_has "$f" "spec: 003-user" "M: renamed-root spec slug recorded"
  rm -rf "$tmp"
}

# ---------- N: checked-out service, wrong root segment not harvested ----------

test_N_checked_out_wrong_root_skipped() {
  local tmp; tmp="$(make_fixture)"
  # api is checked out and rooted at `governance`; a body link that uses the
  # wrong `/specs/` segment does not point at api's real spec root, so tier-1
  # (exact match against the checkout's resolved root) does not harvest it.
  write_govern_toml "$tmp" <<'EOF'
[services.api]
repo = "https://github.com/acme/api"
path = "checkouts/api"
EOF
  mkdir -p "$tmp/checkouts/api"
  cat > "$tmp/checkouts/api/.govern.toml" <<'EOF'
[paths]
specs-root = "governance"
EOF
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

Wrong root: [api](https://github.com/acme/api/blob/main/specs/003-user/spec.md).
EOF
  "$GEN" --root="$tmp" > /dev/null
  fm_lacks "$tmp/specs/001-alpha/spec.md" "references:" \
    "N: checked-out service link with wrong spec-root segment is not harvested"
  rm -rf "$tmp"
}

# ---------- O: registered but not checked out, renamed root (permissive) ----------

test_O_not_checked_out_renamed_root_harvested() {
  local tmp; tmp="$(make_fixture)"
  # api is registered but NOT checked out (the `path` is absent on disk), so its
  # spec-root is unknowable at harvest time. Tier-2: the permissive fallback
  # still harvests the renamed-root link so the reference never silently drops
  # (it resolves later to `unknown — not checked out`).
  write_govern_toml "$tmp" <<'EOF'
[services.api]
repo = "https://github.com/acme/api"
path = "checkouts/api"
EOF
  # checkouts/api intentionally absent — not checked out.
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

Uses the [api user model](https://github.com/acme/api/blob/main/governance/003-user/spec.md).
EOF
  "$GEN" --root="$tmp" > /dev/null
  local f="$tmp/specs/001-alpha/spec.md"
  fm_has "$f" "- service: api" "O: not-checked-out renamed-root reference harvested (permissive)"
  fm_has "$f" "spec: 003-user" "O: not-checked-out renamed-root spec slug recorded"
  rm -rf "$tmp"
}

# ---------- runner ----------

run_all() {
  echo "Running gen-cross-service-refs tests..."
  test_A_registered_link_harvested
  test_B_unregistered_link_null_service
  test_C_see_also_excluded
  test_D_branch_ref_same_identity
  test_E_dependencies_untouched
  test_F_inline_code_excluded
  test_G_code_fence_excluded
  test_H_blockquote_excluded
  test_I_idempotent
  test_J_stale_block_removed
  test_K_staged_scopes_rewrite
  test_L_configured_specs_root
  test_M_referenced_renamed_root_checked_out
  test_N_checked_out_wrong_root_skipped
  test_O_not_checked_out_renamed_root_harvested

  if [ "$failures" -gt 0 ]; then
    echo "$failures test(s) failed" >&2
    exit 1
  fi
  echo "All gen-cross-service-refs tests passed"
}

run_all
