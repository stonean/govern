---
spec: 022-deterministic-runtime
reviewed-at: 2026-07-11T22:05:00Z
reviewed-against: cdb0348
diff-base: 5f25ebe
must-violations: 9
should-violations: 21
low-confidence: 2
captured-issues: 1
skipped-passes: []
---

# Review — 022-deterministic-runtime

## Summary

Blocking. The backlog burn-down (diff `5f25ebe..cdb0348`, ~110 files) is
audited against the eight backend/cross rule files. **9 MUST violations**
across two clusters: five input-validation gaps in the runtime (slug and
boundary validation, archive decompression bound, outbound-fetch
allowlisting) and four exec-path correctness bugs in the wave-3 command
rewrites, all caused by the parser's last-primitive-span-wins rule silently
dropping a primitive when one numbered step names two. 21 SHOULD (mostly
consolidation the wave-4 refactor missed, plus efficiency hoists and
command-prose restatement) and 2 low-confidence findings are advisory. One
issue was captured to the inbox during the work window. The spec cannot
advance to `done` until the MUST violations are resolved or waived.

## MUST violations (blocking)

### MUST: BE-INPUT-001 — `append-task` never validates its `slug` argument

- **File**: `runtime/src/primitives/append_task.rs:27-52`
- **Rule**: "All input crossing a system boundary MUST be validated server-side against an explicit schema before processing."
- **Finding**: `run()` validates `title`, `done-when`, and every `body` item with `validate_single_line`, but never validates `args.slug` before `render_task_block` interpolates it into `` - [ ] Implement the behavior described in `scenarios/{slug}.md` ``. A slug of `x\n## 99. Phantom\n- [ ] injected` smuggles markdown structure into tasks.md — the same injection class the function already rejects for its other three text arguments. No upstream layer validates it (mcp/server.rs and interpreter dispatch call `run` directly).
- **Auto-fixable**: yes
- **Suggested fix**: call `validate_slug(&slug)` (once BE-INPUT-002 is fixed) plus the single-line check before the feature-dir check, mirroring `create_scenario`.

### MUST: BE-INPUT-002 — `validate_slug` is a denylist that admits newlines and control characters

- **File**: `runtime/src/primitives/mod.rs:647-667`
- **Rule**: "Input validation MUST use allowlists (define what is acceptable) for constrained inputs. Denylists MUST NOT be used as the sole defense."
- **Finding**: `validate_slug` — the sole slug defense for `create-scenario` (writes `scenarios/{slug}.md` and an H1 via `title_from_slug`) and `resolve-feature` — only rejects `/`, `\`, leading `.`, and empty. It admits `\n`, `\r`, and control characters into a written filename and a rendered heading, so `"a\nb"` yields a newline-bearing filename and a corrupted H1.
- **Auto-fixable**: yes
- **Suggested fix**: anchor an allowlist matching the framework's slug grammar (`^[a-z0-9]+(?:-[a-z0-9]+)*$`, the alphabet `create_feature::derive_slug` already produces).

### MUST: BE-INPUT-004 — writeCode boundary screen splits on `/` only, so backslash segments escape on Windows

- **File**: `runtime/src/schema/extensions.rs:521-566`
- **Rule**: "User-supplied values MUST NOT be used directly in filesystem paths. Filesystem operations MUST resolve the canonical path and verify it falls within the expected base directory before opening the file."
- **Finding**: `boundary_rejection_reason` screens LLM-supplied edit paths for absolute forms, drive prefixes, and `.`/`..`/empty segments, but splits on `/` only. `runtime/a\..\..\..\x` passes (its single backslash-laden segment is not `..`) and satisfies `runtime/**`, escaping the write boundary when applied on the `x86_64-pc-windows-msvc` target the release ships. The screen already treats other Windows path forms (leading `\`, `C:`) as in-scope, so embedded backslash separators are a gap in the same defense.
- **Auto-fixable**: yes
- **Suggested fix**: split the segment screen on both `/` and `\`, or reject any edit path containing `\` outright.

### MUST: BE-INPUT-006 — archive extraction has no decompressed-size bound (decompression bomb)

- **File**: `runtime/src/primitives/extract_archive.rs:97-209`
- **Rule**: "Inputs that contribute to resource consumption MUST have explicit upper bounds. Requests exceeding limits MUST be rejected." (also fired under BE-QUERY-003 in the efficiency pass — deduped; security is pass-of-record)
- **Finding**: `extract_tar_gz` and `extract_zip` bound neither cumulative decompressed bytes nor entry count. `fetch-archive`'s `MAX_FETCH_BYTES` caps only the *compressed* body, and gzip/deflate expand up to ~1000×, so a ~256 MiB bomb served through the unverified bootstrap path (`fetch-archive` with `sha256_url: None`, the documented main-tarball case) writes ~250 GB to disk — unbounded untrusted input as disk-exhaustion DoS.
- **Auto-fixable**: yes
- **Suggested fix**: add a named `MAX_EXTRACT_BYTES` (and an entry-count cap), track a running decompressed total across entries via `Read::take`, and error naming the cap — mirroring `read_capped`'s detect-don't-truncate contract.

### MUST: BE-INPUT-007 — `fetch-archive` has no scheme allowlist or internal-range denial (SSRF)

- **File**: `runtime/src/primitives/fetch_archive.rs:99-112`
- **Rule**: "When the server fetches a URL supplied or influenced by user input, the destination MUST be validated against an allowlist of acceptable hosts or schemes, AND outbound requests MUST be denied by default to internal address ranges (loopback, link-local, RFC 1918, cloud metadata endpoints)."
- **Finding**: `fetch_bytes` passes the caller-supplied URL to `reqwest::blocking::get` with no scheme or host allowlist and no internal-range denial: a plain `http://` URL is fetched unencrypted, and with `sha256_url: None` the unverified body is written and extracted with exec bits preserved. Over MCP the primitive will also fetch loopback/link-local/metadata URLs. The archive-network-hardening scenario added the size cap but committed to neither allowlisting nor internal-range denial.
- **Auto-fixable**: yes
- **Suggested fix**: enforce an `https`-only scheme allowlist in `fetch_bytes` (also satisfies BE-DATA-001) and reject URLs resolving to loopback/link-local/RFC-1918/metadata addresses before connecting.

### MUST: quality — `clarify.md` step 2 dispatches `set-status`, not `read-spec`

- **File**: `framework/commands/clarify.md:62-68`
- **Rule**: clarify-command-acceleration Behavior: "numbered steps invoking `read-spec` (gate branch) …"
- **Finding**: Step 2's prose invokes `read-spec` first but the recovery-branch also backticks `set-status`; the parser keeps only the last primitive span (parser/mod.rs:267), so the step binds `set-status`. Verified against the built binary (`gvrn parse` reports `"name":"set-status"` for step 2). On `gvrn exec clarify` with session-seeded `from=draft`/`to=clarified`, the status WRITE fires at the gate-branch step before any question is resolved, `read-spec` never dispatches, and step 9's second `set-status` then halts on status-mismatch. The `.claude/commands/gov/clarify.md` mirror is identical.
- **Auto-fixable**: yes
- **Suggested fix**: split the recovery-branch `set-status` into its own numbered step (or un-backtick it in step 2); consider a parser diagnostic when one step carries multiple primitive spans.

### MUST: quality — `clarify.md` step 9 flips status ungated on the exec path

- **File**: `framework/commands/clarify.md:84`
- **Rule**: clarify-command-acceleration Behavior: "`set-status` (draft → clarified) behind the user-approval gate."
- **Finding**: Step 9 puts the gate phrase ("ask the user to approve the transition") and the `set-status` invocation in one step. The pinned dispatch-wins convention (a primitive step containing the phrase dispatches without gating — interpreter/mod.rs docs, walker.rs test) means the draft→clarified flip emits no `gate-confirm` envelope. plan.md/specify.md use a separate `gate-confirm` step before their write; clarify.md does not.
- **Auto-fixable**: yes
- **Suggested fix**: insert a `gate-confirm` step before step 9's `set-status`, matching plan.md steps 6-7.

### MUST: quality — `groom.md` step 4 drops `create-scenario`

- **File**: `framework/commands/groom.md:42`
- **Rule**: groom-command-acceleration Behavior: "the mechanical consequences invoke … `create-scenario` + `append-task` for a scenario route."
- **Finding**: Step 4 invokes both `create-scenario` and `append-task` in one numbered step; last-span-wins makes it parse as `append-task` (verified: `gvrn parse` reports `"name":"append-task"`), so `create-scenario` is absent from the AST. On the exec path the scenario file is never written while its referencing task is appended — a dangling task pointing at a nonexistent scenario.
- **Auto-fixable**: yes
- **Suggested fix**: split step 4 into two single-primitive steps.

### MUST: quality — `gvrn exec specify` rewrites the session to the stale target

- **File**: `runtime/src/interpreter/mod.rs:240-250` (+ `framework/commands/specify.md:45`)
- **Rule**: specify.md Context: "If `.govern.session.toml` exists, the session target will be overwritten with the new feature."
- **Finding**: On `gvrn exec specify` against a repo whose session already has a target, `feature`/`path` are session-seeded context keys and the merge policy ("a primitive result may never overwrite a seeded key") blocks `create-feature`'s result from replacing them — so step 5's `write-session` binds the stale target and rewrites the session to the old feature with a fresh `set-at`, immediately after the gate told the user the new feature would be targeted. Shown by golden `specify-basic.jsonl` line 2: post-`create-feature` context carries `"feature":"006-specify"` alongside `"created":true` for the new `007-webhook-delivery`; the parity test asserts stdout only, never the session file.
- **Auto-fixable**: yes
- **Suggested fix**: thread `create-feature`'s `feature`/`path` into the `write-session` binding explicitly (mirroring the `process-waivers` `findings`→`fired` special case), or let a `created: true` result override the seeded target keys.

## SHOULD violations (advisory)

### Consolidation the wave-4 refactor missed (reuse)

- **`runtime/src/primitives/resolve_feature.rs:187-192`** — private `StatusOnly`/`read_status` re-implements the shared `primitives::frontmatter_status`. Replace with the helper, delete the struct.
- **`runtime/src/interpreter/payload.rs:589-601`** — `read_spec_status` is a second hand-rolled copy of `frontmatter_status`, differing only in `String` vs `Option<String>`. Use the helper with `.unwrap_or_default()`.
- **`runtime/src/primitives/resolve_feature.rs:197-231`** — duplicates dashboard's `ScenarioFrontmatter` struct and `section.or(spec_ref)` fallback verbatim (comment even says "mirrors"). Lift a shared `read_scenario_section` into mod.rs.
- **`runtime/src/primitives/resolve_feature.rs:148-160`** — the "read_dir → keep dirs → `is_feature_slug` → sort" walk now has three copies (resolve_feature, create_feature, payload::load_available_specs) plus dashboard's pre-existing fourth. Add `list_feature_dirs` to mod.rs.
- **`runtime/src/primitives/resolve_feature.rs:140-142`** — the `NNN-` prefix parse `feature_number` is inlined again in `create_feature::next_feature_number`. Share it.
- **`runtime/src/primitives/create_feature.rs:135-149`** — `resolve_template` hardcodes the same two-candidate order as `payload::load_template`. Extract `template_candidates`.
- **`runtime/src/primitives/check_artifacts.rs:53`** — `PLANNED_OR_LATER` hand-maintains a lifecycle tail that `schema/status.rs` already derives as `COMPATIBLE_STATUSES`. Use it.
- **`runtime/src/primitives/dashboard.rs:32`** — `UNBLOCKING_STATUSES` is a hand-maintained lifecycle subset the status refactor missed; derive it as `ALLOWED_STATUSES.split_at(1).1`.
- **`runtime/src/primitives/append_inbox.rs:127-136`** — `bullet_text` hand-rolls checkbox recognition and *diverges* from the shared `checkbox` grammar (accepts `- [x]no-space` which the shared parser rejects), so dedup matching can disagree with the read/mark side. Reuse `checkbox::parse_checkbox_line`.
- **`runtime/src/primitives/check_artifacts.rs:239-251`** — `scenario_slugs` re-implements dashboard's `scenarios/` walk with *divergent* case handling (case-sensitive `.md` vs dashboard's insensitive), so a `FOO.MD` scenario is counted by one surface and invisible to the other. Share a `list_scenario_files` helper.

### Efficiency

- **`runtime/src/primitives/check_stuck.rs:156-208`** (BE-QUERY-001) — `find_in_progress_commit` memoizes status parses by *commit* OID, but the spec blob is identical across nearly all commits, so an adopter repo with ~50-100k commits re-parses the same few spec versions tens of thousands of times per invocation. Key the cache by the spec's tree-entry blob OID instead.
- **`runtime/src/mcp/server.rs:258-265`** (BE-ASYNC-001) — `check-stuck` (now a full-history revwalk) and the other git-walk tools (`compute-review-scope`, `derive-boundary`) run inline on a tokio worker; route them through the `dispatch_blocking` seam this diff added for exactly this class.
- **`runtime/src/interpreter/payload.rs:1173-1181`** (BE-QUERY-001) — `is_gitignored` calls `Repository::discover` per path inside the plan's `## Affected Files` loop. Hoist the discover above the loop.
- **`runtime/src/interpreter/payload.rs:739-819`** (BE-QUERY-001) — `resolve_assessed_rule` re-reads every rule file per `assessSpecQuality` request (one per rule), and `first_rule_with_verification` rescans the file top-down per heading (O(n²)). Single-pass the fallback; optionally cache rule-file contents for the walk.

### Simplicity

- **`runtime/src/interpreter/payload.rs:206-222`** — the typed-prefix-then-legacy-merge epilogue is copy-pasted across the seven extension builders. Extract `typed_with_legacy_context` / `typed_only` helpers.
- **`runtime/src/interpreter/payload.rs:1060-1064`** — `read_existing_section`'s `_ => &["plan.md","spec.md"]` fallback arm is dead (only plan/specify carry the writeSpecBody marker) and re-encodes the fallback order the extension-request-hygiene scenario removed. Replace with `_ => return None`.
- **`framework/commands/clarify.md:62-68`** — step 2 restates all five Gate-table rows (including verbatim stop messages) instead of pointing at the table above; the messages now live twice and can drift.
- **`framework/commands/clarify.md:74`** — step 5 restates the question-resolution sub-procedure that the preamble already delegates to the reference (appears three times in the file); the prose is not shipped in the typed request. Reduce to the marker + one-round-trip contract + a pointer.
- **`framework/commands/groom.md:44-46`** — steps 5-6 cite reference sections then restate their rules verbatim (the from-guard and cli-config-dir sentences duplicate the reference bullets word-for-word).

### Quality (advisory)

- **`runtime/src/interpreter/mod.rs` dispatch (+ implement.md:77)** — on the exec path, step 11's `mark-criterion` dispatches once with `criterion-index`/`checked: true` pre-seeded and never consults the `verifyCriteria` verdicts, so a `met: false` verdict for the seeded index still flips the checkbox (the decision was made before verification ran). Thread `llm:verifyCriteria.results` into the binding, like `fired`.
- **`runtime/src/primitives/read_spec.rs:102-111`** — `parse_checkboxes` drops wrapped continuation lines, so a multi-line acceptance criterion reaches `verifyCriteria` truncated mid-sentence (visible in golden `implement-basic.jsonl` line 10). Fold indented continuation lines into the criterion text, mirroring `parse_open_questions`; keep index derivation from checkbox lines only.

## Low-confidence findings

- **`runtime/src/interpreter/payload.rs:371-405` (+ specify.md:37)** — confidence 65. specify.md step 1 routes `$ARGUMENTS` to the `title` key, but the writeSpecBody builder reads only `feature-description`, so a host following the command file emits an empty `feature-description` (golden `specify-basic.jsonl` line 2). Fall back to `title`, or seed both keys.
- **`framework/commands/target.md:32`** — confidence 55. Step 2 names "the write-session primitive's clear mode" without a backticked invocation and under `audit:ignore-promotion`, so `--clear` parses as prose and is unreachable on the exec path. Plausibly deliberate (argument-conditional branch the linear walker cannot express; MCP path calls the tool directly), but the scenario says target.md `--clear` invokes the primitive.

## Waived findings

*None.*

## Captured issues (pending /gov:groom)

- Architectural exploration: re-frame the runtime's LLM extension points as named Anthropic-style Skills the host loads at the seam (on hold per user 2026-07-11). Informational — parked for `/gov:groom`, not a review finding.

## Skipped passes

*None — all five dimensions ran.*
