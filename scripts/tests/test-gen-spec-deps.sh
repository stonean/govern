#!/usr/bin/env bash
# Test surface for scripts/gen-spec-deps.sh.
#
# Builds tiny fixture spec trees under temp dirs, runs the generator against
# them via the `--root=PATH` flag, and asserts on the resulting frontmatter
# `dependencies:` line (or on exit status / stderr for cycle-detection tests).
#
# Coverage:
#   skip-prose-cross-references (spec 017 scenario):
#     A. link under `## See also` produces no edge
#     B. link outside `## See also` produces an edge
#     C. link inside a code fence produces no edge (regression)
#     D. See-also region ends at next level-2 heading
#     E. `## References` still produces edges (13-spec migration invariant)
#     F. deeper subheading inside `## See also` inherits the opt-out
#     G. running twice on an opt-out tree produces no diff (idempotence)
#
#   detect-dependency-cycles (spec 017 scenario):
#     H. acyclic graph exits 0 with no cycle output
#     I. 2-cycle exits non-zero and names both slugs on stderr
#     J. 3-cycle is reported as a single SCC, not three 2-cycles
#     K. graph with cycle and acyclic subgraph reports only the cycle
#     L. self-cycle reported as a 1-cycle
#     M. multiple disjoint cycles all reported in one run
#
#   tracked-specs-not-worktree (spec 017 scenario):
#     N. untracked draft specs are skipped; only git-tracked specs are processed
#
# Usage: scripts/tests/test-gen-spec-deps.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
GEN="$REPO_ROOT/scripts/gen-spec-deps.sh"

failures=0
pass() { printf '  PASS  %s\n' "$1"; }
fail() { printf '  FAIL  %s\n' "$1" >&2; failures=$((failures + 1)); }

read_deps() {
  awk '/^dependencies:/ { print; exit }' "$1"
}

assert_deps() {
  local file="$1" expected="$2" label="$3" actual
  actual="$(read_deps "$file")"
  if [ "$actual" = "$expected" ]; then
    pass "$label"
  else
    fail "$label: expected '$expected', got '$actual'"
  fi
}

assert_nonzero() {
  local rc="$1" label="$2"
  if [ "$rc" -ne 0 ]; then
    pass "$label (exit=$rc)"
  else
    fail "$label: expected non-zero exit, got 0"
  fi
}

assert_zero() {
  local rc="$1" label="$2"
  if [ "$rc" -eq 0 ]; then
    pass "$label"
  else
    fail "$label: expected exit 0, got $rc"
  fi
}

assert_stderr_contains() {
  local stderr_file="$1" needle="$2" label="$3"
  if grep -qF -- "$needle" "$stderr_file"; then
    pass "$label"
  else
    fail "$label: stderr did not contain '$needle' (got: $(tr '\n' '|' < "$stderr_file"))"
  fi
}

assert_stderr_not_contains() {
  local stderr_file="$1" needle="$2" label="$3"
  if ! grep -qF -- "$needle" "$stderr_file"; then
    pass "$label"
  else
    fail "$label: stderr unexpectedly contained '$needle'"
  fi
}

make_fixture() {
  mktemp -d
}

# Write a minimal spec.md with given body content into the fixture.
# Usage: write_spec <tmp> <slug> <heredoc-marker> ... body ... <marker>
write_spec() {
  local tmp="$1" slug="$2"
  mkdir -p "$tmp/specs/$slug"
  cat > "$tmp/specs/$slug/spec.md"
}

# ---------- skip-prose-cross-references ----------

test_A_see_also_skips_edge() {
  local tmp; tmp="$(make_fixture)"
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

## See also

- [002-beta](../002-beta/spec.md)
EOF
  write_spec "$tmp" "002-beta" <<'EOF'
---
status: clarified
dependencies: []
---

# Beta
EOF
  "$GEN" --root="$tmp" > /dev/null
  assert_deps "$tmp/specs/001-alpha/spec.md" "dependencies: []" "A: link under ## See also produces no edge"
  rm -rf "$tmp"
}

test_B_normal_link_produces_edge() {
  local tmp; tmp="$(make_fixture)"
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

Depends on [002-beta](../002-beta/spec.md) for X.
EOF
  write_spec "$tmp" "002-beta" <<'EOF'
---
status: clarified
dependencies: []
---

# Beta
EOF
  "$GEN" --root="$tmp" > /dev/null
  assert_deps "$tmp/specs/001-alpha/spec.md" "dependencies: [002-beta]" "B: normal link produces an edge"
  rm -rf "$tmp"
}

test_C_code_fence_skips_edge() {
  local tmp; tmp="$(make_fixture)"
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

```
See [002-beta](../002-beta/spec.md) example.
```
EOF
  write_spec "$tmp" "002-beta" <<'EOF'
---
status: clarified
dependencies: []
---

# Beta
EOF
  "$GEN" --root="$tmp" > /dev/null
  assert_deps "$tmp/specs/001-alpha/spec.md" "dependencies: []" "C: link inside a code fence produces no edge"
  rm -rf "$tmp"
}

test_D_see_also_region_ends_at_next_heading() {
  local tmp; tmp="$(make_fixture)"
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

## See also

- [002-beta](../002-beta/spec.md)

## Notes

Depends on [003-gamma](../003-gamma/spec.md).
EOF
  write_spec "$tmp" "002-beta" <<'EOF'
---
status: clarified
dependencies: []
---

# Beta
EOF
  write_spec "$tmp" "003-gamma" <<'EOF'
---
status: clarified
dependencies: []
---

# Gamma
EOF
  "$GEN" --root="$tmp" > /dev/null
  assert_deps "$tmp/specs/001-alpha/spec.md" "dependencies: [003-gamma]" "D: See-also region ends at next level-2 heading"
  rm -rf "$tmp"
}

test_E_references_section_produces_edges() {
  local tmp; tmp="$(make_fixture)"
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

## References

- [002-beta](../002-beta/spec.md)
EOF
  write_spec "$tmp" "002-beta" <<'EOF'
---
status: clarified
dependencies: []
---

# Beta
EOF
  "$GEN" --root="$tmp" > /dev/null
  assert_deps "$tmp/specs/001-alpha/spec.md" "dependencies: [002-beta]" "E: ## References still produces edges"
  rm -rf "$tmp"
}

test_F_deeper_subheading_inherits_optout() {
  local tmp; tmp="$(make_fixture)"
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

## See also

### Related

- [002-beta](../002-beta/spec.md)
EOF
  write_spec "$tmp" "002-beta" <<'EOF'
---
status: clarified
dependencies: []
---

# Beta
EOF
  "$GEN" --root="$tmp" > /dev/null
  assert_deps "$tmp/specs/001-alpha/spec.md" "dependencies: []" "F: ### subheading inside ## See also inherits opt-out"
  rm -rf "$tmp"
}

test_G_idempotent_with_optout() {
  local tmp; tmp="$(make_fixture)"
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

Depends on [002-beta](../002-beta/spec.md).

## See also

- [003-gamma](../003-gamma/spec.md)
EOF
  write_spec "$tmp" "002-beta" <<'EOF'
---
status: clarified
dependencies: []
---

# Beta
EOF
  write_spec "$tmp" "003-gamma" <<'EOF'
---
status: clarified
dependencies: []
---

# Gamma
EOF
  "$GEN" --root="$tmp" > /dev/null
  local after_first; after_first="$(cat "$tmp/specs/001-alpha/spec.md")"
  "$GEN" --root="$tmp" > /dev/null
  local after_second; after_second="$(cat "$tmp/specs/001-alpha/spec.md")"
  if [ "$after_first" = "$after_second" ]; then
    pass "G: idempotent — second run produces no diff"
  else
    fail "G: second run produced a diff"
  fi
  rm -rf "$tmp"
}

# ---------- detect-dependency-cycles ----------

# Run the generator capturing stdout, stderr, and exit code.
# Sets globals: GEN_STDOUT, GEN_STDERR, GEN_RC.
run_gen() {
  local tmp="$1" out err rc
  out="$(mktemp)"
  err="$(mktemp)"
  rc=0
  "$GEN" --root="$tmp" >"$out" 2>"$err" || rc=$?
  GEN_STDOUT="$(cat "$out")"
  GEN_STDERR="$(cat "$err")"
  GEN_RC=$rc
  rm -f "$out" "$err"
}

# Helper: write a minimal acyclic spec with given outgoing links to other specs.
write_basic_spec() {
  local tmp="$1" slug="$2"; shift 2
  mkdir -p "$tmp/specs/$slug"
  {
    echo '---'
    echo 'status: clarified'
    echo 'dependencies: []'
    echo '---'
    echo ''
    echo "# ${slug#*-}"
    echo ''
    for target in "$@"; do
      echo "Depends on [$target](../$target/spec.md)."
    done
  } > "$tmp/specs/$slug/spec.md"
}

test_H_acyclic_exits_zero() {
  local tmp; tmp="$(make_fixture)"
  write_basic_spec "$tmp" "001-alpha"
  write_basic_spec "$tmp" "002-beta" "001-alpha"
  write_basic_spec "$tmp" "003-gamma" "001-alpha" "002-beta"
  run_gen "$tmp"
  assert_zero "$GEN_RC" "H: acyclic graph exits 0"
  assert_stderr_not_contains <(echo "$GEN_STDERR") "cycle:" "H: no cycle output on acyclic graph"
  rm -rf "$tmp"
}

test_I_two_cycle_detected() {
  local tmp; tmp="$(make_fixture)"
  write_basic_spec "$tmp" "001-alpha" "002-beta"
  write_basic_spec "$tmp" "002-beta" "001-alpha"
  run_gen "$tmp"
  assert_nonzero "$GEN_RC" "I: 2-cycle exits non-zero"
  local stderr_tmp; stderr_tmp="$(mktemp)"
  printf '%s\n' "$GEN_STDERR" > "$stderr_tmp"
  assert_stderr_contains "$stderr_tmp" "001-alpha" "I: 2-cycle names 001-alpha on stderr"
  assert_stderr_contains "$stderr_tmp" "002-beta" "I: 2-cycle names 002-beta on stderr"
  assert_stderr_contains "$stderr_tmp" "cycle:" "I: 2-cycle has 'cycle:' label"
  rm -f "$stderr_tmp"
  rm -rf "$tmp"
}

test_J_three_cycle_single_scc() {
  local tmp; tmp="$(make_fixture)"
  write_basic_spec "$tmp" "001-alpha" "002-beta"
  write_basic_spec "$tmp" "002-beta" "003-gamma"
  write_basic_spec "$tmp" "003-gamma" "001-alpha"
  run_gen "$tmp"
  assert_nonzero "$GEN_RC" "J: 3-cycle exits non-zero"
  local cycle_lines
  cycle_lines="$(printf '%s\n' "$GEN_STDERR" | grep -c '^cycle:' || true)"
  if [ "$cycle_lines" -eq 1 ]; then
    pass "J: 3-cycle reported as single SCC (one cycle line)"
  else
    fail "J: expected 1 cycle line, got $cycle_lines: $(printf '%s' "$GEN_STDERR" | tr '\n' '|')"
  fi
  local stderr_tmp; stderr_tmp="$(mktemp)"
  printf '%s\n' "$GEN_STDERR" > "$stderr_tmp"
  assert_stderr_contains "$stderr_tmp" "001-alpha" "J: 3-cycle names 001-alpha"
  assert_stderr_contains "$stderr_tmp" "002-beta" "J: 3-cycle names 002-beta"
  assert_stderr_contains "$stderr_tmp" "003-gamma" "J: 3-cycle names 003-gamma"
  rm -f "$stderr_tmp"
  rm -rf "$tmp"
}

test_K_mixed_reports_only_cycle() {
  local tmp; tmp="$(make_fixture)"
  # 001 ↔ 002 cycle, 003 → 004 acyclic, 005 standalone
  write_basic_spec "$tmp" "001-alpha" "002-beta"
  write_basic_spec "$tmp" "002-beta" "001-alpha"
  write_basic_spec "$tmp" "003-gamma" "004-delta"
  write_basic_spec "$tmp" "004-delta"
  write_basic_spec "$tmp" "005-epsilon"
  run_gen "$tmp"
  assert_nonzero "$GEN_RC" "K: mixed acyclic+cyclic exits non-zero"
  local cycle_lines
  cycle_lines="$(printf '%s\n' "$GEN_STDERR" | grep -c '^cycle:' || true)"
  if [ "$cycle_lines" -eq 1 ]; then
    pass "K: only the cycle reported (1 line); acyclic subgraph silent"
  else
    fail "K: expected 1 cycle line, got $cycle_lines"
  fi
  local stderr_tmp; stderr_tmp="$(mktemp)"
  printf '%s\n' "$GEN_STDERR" > "$stderr_tmp"
  assert_stderr_not_contains "$stderr_tmp" "003-gamma" "K: 003-gamma (acyclic) not in cycle report"
  assert_stderr_not_contains "$stderr_tmp" "004-delta" "K: 004-delta (acyclic) not in cycle report"
  assert_stderr_not_contains "$stderr_tmp" "005-epsilon" "K: 005-epsilon (standalone) not in cycle report"
  rm -f "$stderr_tmp"
  rm -rf "$tmp"
}

test_L_self_cycle() {
  local tmp; tmp="$(make_fixture)"
  write_basic_spec "$tmp" "001-alpha" "001-alpha"
  run_gen "$tmp"
  assert_nonzero "$GEN_RC" "L: self-cycle exits non-zero"
  local stderr_tmp; stderr_tmp="$(mktemp)"
  printf '%s\n' "$GEN_STDERR" > "$stderr_tmp"
  assert_stderr_contains "$stderr_tmp" "cycle: 001-alpha -> 001-alpha" "L: self-cycle reported as 1-cycle"
  rm -f "$stderr_tmp"
  rm -rf "$tmp"
}

test_M_multiple_disjoint_cycles() {
  local tmp; tmp="$(make_fixture)"
  # Two disjoint 2-cycles
  write_basic_spec "$tmp" "001-alpha" "002-beta"
  write_basic_spec "$tmp" "002-beta" "001-alpha"
  write_basic_spec "$tmp" "003-gamma" "004-delta"
  write_basic_spec "$tmp" "004-delta" "003-gamma"
  run_gen "$tmp"
  assert_nonzero "$GEN_RC" "M: disjoint cycles exit non-zero"
  local cycle_lines
  cycle_lines="$(printf '%s\n' "$GEN_STDERR" | grep -c '^cycle:' || true)"
  if [ "$cycle_lines" -eq 2 ]; then
    pass "M: both disjoint cycles reported (2 cycle lines)"
  else
    fail "M: expected 2 cycle lines, got $cycle_lines: $(printf '%s' "$GEN_STDERR" | tr '\n' '|')"
  fi
  rm -rf "$tmp"
}

# ---------- tracked-specs-not-worktree ----------

# When run inside a git repo, the generator processes only git-tracked (indexed)
# specs; an untracked, in-progress draft sitting in the worktree is left
# untouched and never enters the dependency graph. Outside a git repo the
# worktree-glob fallback covers every spec (the path tests A–M exercise).
test_N_untracked_draft_ignored() {
  local tmp; tmp="$(make_fixture)"
  git -C "$tmp" init -q
  git -C "$tmp" config user.email "test@example.com"
  git -C "$tmp" config user.name "test"

  # 001 (tracked) and 002 (untracked draft) both link to 003 in their bodies.
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

Depends on [003-gamma](../003-gamma/spec.md).
EOF
  write_spec "$tmp" "003-gamma" <<'EOF'
---
status: clarified
dependencies: []
---

# Gamma
EOF
  git -C "$tmp" add specs/001-alpha/spec.md specs/003-gamma/spec.md
  git -C "$tmp" commit -q -m init

  # 002-beta is written but never `git add`ed — an in-progress draft.
  write_spec "$tmp" "002-beta" <<'EOF'
---
status: draft
dependencies: []
---

# Beta

Depends on [003-gamma](../003-gamma/spec.md).
EOF

  "$GEN" --root="$tmp" > /dev/null
  assert_deps "$tmp/specs/001-alpha/spec.md" "dependencies: [003-gamma]" \
    "N: tracked spec is processed"
  assert_deps "$tmp/specs/002-beta/spec.md" "dependencies: []" \
    "N: untracked draft is left untouched (deps not derived)"
  rm -rf "$tmp"
}

# --staged: only specs staged for the pending commit are rewritten; an unstaged
# (committed) spec whose derived deps have drifted is left untouched. This is
# the adopter pre-commit path — committing one spec must not restage others.
test_O_staged_scopes_rewrite() {
  local tmp; tmp="$(make_fixture)"
  git -C "$tmp" init -q
  git -C "$tmp" config user.email "test@example.com"
  git -C "$tmp" config user.name "test"

  # 002-beta: committed with drifted deps (body links 003, frontmatter is []).
  write_spec "$tmp" "002-beta" <<'EOF'
---
status: clarified
dependencies: []
---

# Beta

Depends on [003-gamma](../003-gamma/spec.md).
EOF
  write_spec "$tmp" "003-gamma" <<'EOF'
---
status: clarified
dependencies: []
---

# Gamma
EOF
  git -C "$tmp" add -A
  git -C "$tmp" commit -q -m init

  # 001-alpha: new, staged, links 003.
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

Depends on [003-gamma](../003-gamma/spec.md).
EOF
  git -C "$tmp" add specs/001-alpha/spec.md

  "$GEN" --staged --root="$tmp" > /dev/null
  assert_deps "$tmp/specs/001-alpha/spec.md" "dependencies: [003-gamma]" \
    "O: staged spec is rewritten under --staged"
  assert_deps "$tmp/specs/002-beta/spec.md" "dependencies: []" \
    "O: unstaged drifted spec is left untouched"
  rm -rf "$tmp"
}

# --staged still runs the cycle check over the FULL graph: a staged spec whose
# new edge closes a cycle through an unstaged (committed) spec must still fail.
test_P_staged_cycle_spans_full_graph() {
  local tmp; tmp="$(make_fixture)"
  git -C "$tmp" init -q
  git -C "$tmp" config user.email "test@example.com"
  git -C "$tmp" config user.name "test"

  # beta -> alpha, committed (and not part of the pending change).
  write_spec "$tmp" "002-beta" <<'EOF'
---
status: clarified
dependencies: [001-alpha]
---

# Beta

Depends on [001-alpha](../001-alpha/spec.md).
EOF
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha
EOF
  git -C "$tmp" add -A
  git -C "$tmp" commit -q -m init

  # Add alpha -> beta, closing the cycle. Stage ONLY alpha.
  write_spec "$tmp" "001-alpha" <<'EOF'
---
status: clarified
dependencies: []
---

# Alpha

Now depends on [002-beta](../002-beta/spec.md).
EOF
  git -C "$tmp" add specs/001-alpha/spec.md

  local rc=0 err; err="$(mktemp)"
  "$GEN" --staged --root="$tmp" > /dev/null 2>"$err" || rc=$?
  assert_nonzero "$rc" "P: cycle closed through an unstaged spec is caught under --staged"
  assert_stderr_contains "$err" "cycle:" "P: cycle reported on stderr"
  rm -f "$err"
  rm -rf "$tmp"
}

# ---------- runner ----------

run_all() {
  echo "Running gen-spec-deps tests..."
  test_A_see_also_skips_edge
  test_B_normal_link_produces_edge
  test_C_code_fence_skips_edge
  test_D_see_also_region_ends_at_next_heading
  test_E_references_section_produces_edges
  test_F_deeper_subheading_inherits_optout
  test_G_idempotent_with_optout
  test_H_acyclic_exits_zero
  test_I_two_cycle_detected
  test_J_three_cycle_single_scc
  test_K_mixed_reports_only_cycle
  test_L_self_cycle
  test_M_multiple_disjoint_cycles
  test_N_untracked_draft_ignored
  test_O_staged_scopes_rewrite
  test_P_staged_cycle_spans_full_graph

  if [ "$failures" -gt 0 ]; then
    echo "$failures test(s) failed" >&2
    exit 1
  fi
  echo "All gen-spec-deps tests passed"
}

run_all
