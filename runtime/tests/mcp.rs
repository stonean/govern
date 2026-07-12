//! Integration test for the MCP server surface.
//!
//! Starts a `GovRuntimeServer` in-process on one half of a `tokio::io::duplex`
//! pair and connects an `rmcp` client on the other half. The test:
//!
//! 1. Lists tools and asserts every name in `TOOL_NAMES` is present.
//! 2. Asserts `framework/runtime-tools.txt` (the shipped manifest) is
//!    set-equal to `TOOL_NAMES` (the canonical primitive registry).
//! 3. Invokes each read-only primitive against the shared fixture repo and
//!    asserts the response is a structured JSON object.
//! 4. Invokes the write primitives against per-test scratch copies of the
//!    fixture so each test is hermetic.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParams;
use serde_json::{Value, json};

use gvrn::mcp::server::{GovRuntimeServer, TOOL_NAMES};

fn fixture_repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/primitives/sample-repo")
}

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is the runtime crate; the workspace root is its parent.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

fn copy_recursively(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let ty = entry.file_type().unwrap();
        let dst_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_recursively(&entry.path(), &dst_path);
        } else {
            fs::copy(entry.path(), dst_path).unwrap();
        }
    }
}

async fn start_pair(repo: PathBuf) -> rmcp::service::RunningService<rmcp::service::RoleClient, ()> {
    let (server_side, client_side) = tokio::io::duplex(64 * 1024);
    let server = GovRuntimeServer::new(repo);
    tokio::spawn(async move {
        match server.serve(server_side).await {
            Ok(service) => {
                let _ = service.waiting().await;
            }
            Err(err) => eprintln!("server failed: {err}"),
        }
    });
    ().serve(client_side).await.unwrap()
}

async fn call_tool(
    client: &rmcp::service::RunningService<rmcp::service::RoleClient, ()>,
    name: &str,
    arguments: Value,
) -> rmcp::model::CallToolResult {
    let args_obj = arguments
        .as_object()
        .cloned()
        .expect("arguments must be a JSON object");
    client
        .call_tool(CallToolRequestParams::new(name.to_string()).with_arguments(args_obj))
        .await
        .unwrap_or_else(|err| panic!("call_tool({name}) failed: {err}"))
}

fn structured_object(result: &rmcp::model::CallToolResult) -> &serde_json::Map<String, Value> {
    result
        .structured_content
        .as_ref()
        .unwrap_or_else(|| panic!("tool returned no structured content: {result:?}"))
        .as_object()
        .expect("structured content is a JSON object")
}

#[tokio::test]
async fn lists_every_manifest_tool_and_canonical_set() {
    let client = start_pair(fixture_repo()).await;
    let tools = client.list_tools(Option::default()).await.unwrap();
    let names: Vec<&str> = tools.tools.iter().map(|t| t.name.as_ref()).collect();

    for expected in TOOL_NAMES {
        assert!(
            names.contains(expected),
            "tool {expected} missing from list_tools (got {names:?})"
        );
    }

    // The shipped manifest must be set-EQUAL to the canonical registry
    // (`TOOL_NAMES` is defined from `schema::registry::PRIMITIVE_REGISTRY`):
    // a manifest entry with no tool is a phantom, and a tool missing from
    // the manifest escapes the markdown-only pipeline's PATH assertion and
    // the graceful-fallback lint.
    let manifest = fs::read_to_string(workspace_root().join("framework/runtime-tools.txt"))
        .expect("framework/runtime-tools.txt must exist (canonical manifest)");
    let manifest_names: std::collections::BTreeSet<&str> = manifest
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect();
    let registry_names: std::collections::BTreeSet<&str> = TOOL_NAMES.iter().copied().collect();
    assert_eq!(
        manifest_names, registry_names,
        "framework/runtime-tools.txt diverged from the primitive registry"
    );
}

/// `schemars` stamps OpenAPI-style numeric `format` hints (`uint32`,
/// `uint64`, `uint8`, …) onto Rust integer/float fields. JSON Schema
/// defines no `format` for numeric types, so strict MCP clients (opencode)
/// log `unknown format "uint32" ignored` for each one. The server strips
/// them at construction; assert none survive on any served tool schema.
#[tokio::test]
async fn no_tool_schema_carries_a_nonstandard_numeric_format() {
    fn collect_formats(value: &Value, out: &mut Vec<String>) {
        match value {
            Value::Object(map) => {
                if let Some(Value::String(fmt)) = map.get("format") {
                    out.push(fmt.clone());
                }
                for child in map.values() {
                    collect_formats(child, out);
                }
            }
            Value::Array(items) => items.iter().for_each(|v| collect_formats(v, out)),
            _ => {}
        }
    }

    let is_numeric_format =
        |f: &str| f.starts_with("int") || f.starts_with("uint") || matches!(f, "float" | "double");

    let client = start_pair(fixture_repo()).await;
    let tools = client.list_tools(Option::default()).await.unwrap();

    for tool in &tools.tools {
        let mut formats = Vec::new();
        collect_formats(&Value::Object((*tool.input_schema).clone()), &mut formats);
        if let Some(output) = &tool.output_schema {
            collect_formats(&Value::Object((**output).clone()), &mut formats);
        }
        let offenders: Vec<&String> = formats.iter().filter(|f| is_numeric_format(f)).collect();
        assert!(
            offenders.is_empty(),
            "tool {} exposes non-standard numeric format(s): {offenders:?}",
            tool.name
        );
    }
}

#[tokio::test]
async fn read_spec_returns_structured_frontmatter() {
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(
        &client,
        "read-spec",
        json!({"feature": "001-basic", "include-body": false}),
    )
    .await;
    let obj = structured_object(&result);
    assert_eq!(obj["frontmatter"]["status"], "clarified");
    assert_eq!(obj["path"], "specs/001-basic/spec.md");
}

/// Task 65 (scenarios/mcp-arg-unknown-field-strictness.md), MCP surface: an
/// argument key outside the primitive's known field set — here the
/// `snake_case` misspelling `include_body` for the kebab `include-body` — is
/// rejected with an error naming the unknown field, rather than silently
/// dropped and run with that field's default. The exec-surface half of the
/// contract (the subprocess interpreter's superset-context binding stays
/// lenient) is asserted by
/// `interpreter::tests::exec_path_ignores_unknown_argument_key`.
#[tokio::test]
async fn mcp_surface_rejects_unknown_argument_field() {
    let client = start_pair(fixture_repo()).await;
    let args = json!({"feature": "001-basic", "include_body": false})
        .as_object()
        .cloned()
        .unwrap();
    let err = client
        .call_tool(CallToolRequestParams::new("read-spec".to_string()).with_arguments(args))
        .await
        .expect_err("a misspelled kebab field must be rejected, not silently dropped");
    let rendered = format!("{err:?}");
    assert!(
        rendered.contains("include_body"),
        "rejection should name the unknown field, got: {rendered}"
    );
}

#[tokio::test]
async fn read_tasks_returns_task_list() {
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(&client, "read-tasks", json!({"feature": "001-basic"})).await;
    let obj = structured_object(&result);
    let tasks = obj["tasks"].as_array().unwrap();
    assert_eq!(tasks.len(), 2);
    assert_eq!(tasks[0]["number"], "1");
}

#[tokio::test]
async fn validate_frontmatter_reports_clean_fixture() {
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(
        &client,
        "validate-frontmatter",
        json!({"path": "specs/001-basic/spec.md"}),
    )
    .await;
    let obj = structured_object(&result);
    assert_eq!(obj["clean"], true);
    assert_eq!(obj["findings"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn resolve_anchor_returns_references() {
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(
        &client,
        "resolve-anchor",
        json!({"path": "framework/constitution.md"}),
    )
    .await;
    let obj = structured_object(&result);
    assert!(obj["references"].is_array());
}

#[tokio::test]
async fn traverse_deps_returns_compatibility() {
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(
        &client,
        "traverse-deps",
        json!({"feature": "002-dependent"}),
    )
    .await;
    let obj = structured_object(&result);
    assert!(obj["compatible"].is_boolean());
    // Acyclic fixture must carry an empty `cycles` array — confirms the
    // schema field is wired through the MCP surface and stays empty when
    // the reachable subgraph has no SCCs.
    assert!(obj["cycles"].is_array());
    assert_eq!(obj["cycles"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn traverse_deps_surfaces_two_cycle_via_mcp() {
    // Parity coverage for spec 022's `traverse-deps-cycle-check` scenario:
    // the MCP surface (called by adopter hosts in markdown-only mode and
    // by every slash command host) returns the same `cycles` payload the
    // primitive's unit tests produce. Asserts the end-to-end wiring from
    // the rmcp tool handler through to the response envelope.
    let tmp = tempfile::tempdir().unwrap();
    let specs = tmp.path().join("specs");
    fs::create_dir_all(&specs).unwrap();
    for (slug, dep) in [("200-a", "201-b"), ("201-b", "200-a")] {
        let dir = specs.join(slug);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("spec.md"),
            format!("---\nstatus: planned\ndependencies: [{dep}]\n---\n\n# {slug}\n"),
        )
        .unwrap();
    }

    let client = start_pair(tmp.path().to_path_buf()).await;
    let result = call_tool(&client, "traverse-deps", json!({"feature": "200-a"})).await;
    let obj = structured_object(&result);
    assert_eq!(obj["compatible"], false, "cycle flips compatible to false");
    let cycles = obj["cycles"].as_array().expect("cycles array").clone();
    assert_eq!(cycles.len(), 1, "exactly one SCC for the two-node cycle");
    let slugs: Vec<String> = cycles[0]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap().to_string())
        .collect();
    assert!(slugs.contains(&"200-a".to_string()));
    assert!(slugs.contains(&"201-b".to_string()));
}

#[tokio::test]
async fn check_rule_ids_returns_citations() {
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(
        &client,
        "check-rule-ids",
        json!({
            "path": "specs/001-basic/spec.md",
            "rule-files": ["framework/rules/security-backend.md"],
        }),
    )
    .await;
    let obj = structured_object(&result);
    assert!(obj["citations"].is_array());
}

#[tokio::test]
async fn resolve_feature_resolves_number_against_fixture() {
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(&client, "resolve-feature", json!({"identifier": "1"})).await;
    let obj = structured_object(&result);
    assert_eq!(obj["outcome"], "resolved");
    assert_eq!(obj["feature"], "001-basic");
    assert_eq!(obj["path"], "specs/001-basic");
    assert_eq!(obj["status"], "clarified");
}

#[tokio::test]
async fn resolve_feature_reports_not_found_as_domain_outcome() {
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(
        &client,
        "resolve-feature",
        json!({"identifier": "no-such-slug"}),
    )
    .await;
    let obj = structured_object(&result);
    assert_eq!(obj["outcome"], "not-found");
    assert_eq!(obj["candidates"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn check_artifacts_reports_clean_fixture() {
    // 001-basic is `clarified` with a well-formed tasks.md and no
    // scenarios — every family passes.
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(&client, "check-artifacts", json!({"feature": "001-basic"})).await;
    let obj = structured_object(&result);
    assert_eq!(obj["clean"], true);
    assert_eq!(obj["status"], "clarified");
    assert_eq!(obj["findings"].as_array().unwrap().len(), 0);
    assert_eq!(obj["path"], "specs/001-basic/spec.md");
}

#[tokio::test]
async fn check_artifacts_flags_planned_spec_missing_artifacts() {
    // 002-dependent is `planned` with no plan.md / tasks.md → two
    // blocking artifact-completeness findings.
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(
        &client,
        "check-artifacts",
        json!({"feature": "002-dependent"}),
    )
    .await;
    let obj = structured_object(&result);
    assert_eq!(obj["clean"], false);
    let findings = obj["findings"].as_array().unwrap();
    assert_eq!(findings.len(), 2, "{findings:?}");
    for finding in findings {
        assert_eq!(finding["family"], "artifact-completeness");
        assert_eq!(finding["severity"], "blocking");
    }
}

#[tokio::test]
async fn gate_confirm_returns_prompt_payload_without_blocking() {
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(
        &client,
        "gate-confirm",
        json!({"gate": "plan-finalize-status", "prompt": "Advance status?"}),
    )
    .await;
    let obj = structured_object(&result);
    assert_eq!(obj["gate"], "plan-finalize-status");
    assert_eq!(obj["prompt"], "Advance status?");
    assert!(
        obj["request-id"].as_str().unwrap().starts_with("gate-"),
        "request-id present and prefixed"
    );
}

#[tokio::test]
async fn mark_task_against_scratch_copy_flips_checkbox() {
    let tmp = tempfile::tempdir().unwrap();
    copy_recursively(&fixture_repo(), tmp.path());
    let client = start_pair(tmp.path().to_path_buf()).await;
    let result = call_tool(
        &client,
        "mark-task",
        json!({
            "feature": "001-basic",
            "task-number": "1",
            "subtask-index": 1,
            "checked": true,
        }),
    )
    .await;
    let obj = structured_object(&result);
    assert_eq!(obj["current"], true);
    let body = fs::read_to_string(tmp.path().join("specs/001-basic/tasks.md")).unwrap();
    assert!(body.contains("- [x] Subtask two — pending."));
}

#[tokio::test]
async fn mark_criterion_against_scratch_copy_flips_checkbox() {
    let tmp = tempfile::tempdir().unwrap();
    copy_recursively(&fixture_repo(), tmp.path());
    let client = start_pair(tmp.path().to_path_buf()).await;
    let result = call_tool(
        &client,
        "mark-criterion",
        json!({
            "feature": "001-basic",
            "criterion-index": 0,
            "checked": true,
        }),
    )
    .await;
    let obj = structured_object(&result);
    assert_eq!(obj["current"], true);
    assert_eq!(obj["previous"], false);
}

/// Build a tempdir copy of the fixture and `git init` + `git commit -m init`
/// against it so primitives that walk git history (`check-stuck`,
/// `derive-boundary`) have a real repo to operate on.
fn init_git_fixture() -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    copy_recursively(&fixture_repo(), tmp.path());
    let repo = git2::Repository::init(tmp.path()).unwrap();
    let mut index = repo.index().unwrap();
    index
        .add_all(["*"], git2::IndexAddOption::DEFAULT, None)
        .unwrap();
    index.write().unwrap();
    let tree_id = index.write_tree().unwrap();
    let tree = repo.find_tree(tree_id).unwrap();
    let sig = git2::Signature::now("Test", "test@example.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .unwrap();
    tmp
}

#[tokio::test]
async fn check_stuck_returns_commit_count() {
    let tmp = init_git_fixture();
    let client = start_pair(tmp.path().to_path_buf()).await;
    let result = call_tool(
        &client,
        "check-stuck",
        json!({"feature": "001-basic", "threshold": 3}),
    )
    .await;
    let obj = structured_object(&result);
    assert!(obj["commit-count"].is_number());
    assert_eq!(obj["threshold"], 3);
}

#[tokio::test]
async fn derive_boundary_returns_diff_paths() {
    let tmp = init_git_fixture();
    let client = start_pair(tmp.path().to_path_buf()).await;
    let result = call_tool(&client, "derive-boundary", json!({"feature": "001-basic"})).await;
    let obj = structured_object(&result);
    assert!(obj["boundary"].is_array());
    assert!(obj["first-commit"].is_string());
}

// Requires bash and Unix permission bits; the production run-generator
// path is likewise Unix-oriented (see run_generator.rs's unix-gated tests).
#[cfg(unix)]
#[tokio::test]
async fn run_generator_invokes_bash_script() {
    let tmp = tempfile::tempdir().unwrap();
    let script = tmp.path().join("gen.sh");
    fs::write(&script, "#!/usr/bin/env bash\necho ok\nexit 0\n").unwrap();
    let mut perms = fs::metadata(&script).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script, perms).unwrap();
    let client = start_pair(tmp.path().to_path_buf()).await;
    let result = call_tool(&client, "run-generator", json!({"script": "gen.sh"})).await;
    let obj = structured_object(&result);
    assert_eq!(obj["drift"], false);
    assert_eq!(obj["exit-code"], 0);
}

#[tokio::test]
async fn lint_markdown_returns_violations_array() {
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(
        &client,
        "lint-markdown",
        json!({"paths": ["specs/001-basic/spec.md"], "fix": false}),
    )
    .await;
    let obj = structured_object(&result);
    // The fixture spec may or may not be clean depending on the local
    // markdownlint config; assert only the response shape is valid.
    assert!(obj["violations"].is_array());
    assert!(obj["clean"].is_boolean());
    assert!(obj["exit-code"].is_number());
}

#[tokio::test]
async fn set_status_against_scratch_copy_updates_field() {
    let tmp = tempfile::tempdir().unwrap();
    copy_recursively(&fixture_repo(), tmp.path());
    let client = start_pair(tmp.path().to_path_buf()).await;
    let result = call_tool(
        &client,
        "set-status",
        json!({
            "feature": "001-basic",
            "from": "clarified",
            "to": "planned",
        }),
    )
    .await;
    let obj = structured_object(&result);
    assert_eq!(obj["previous"], "clarified");
    assert_eq!(obj["current"], "planned");
}

// -- review-runtime-acceleration primitives (spec 022 scenario) --------------

#[tokio::test]
async fn discover_rule_files_selects_via_mcp() {
    // The shared fixture repo ships framework/rules/security-backend.md.
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(&client, "discover-rule-files", json!({})).await;
    let obj = structured_object(&result);
    let selected: Vec<&str> = obj["selected"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(
        selected.iter().any(|s| s.contains("security-backend")),
        "expected security-backend.md in {selected:?}"
    );
    assert!(obj["notices"].is_array());
}

#[tokio::test]
async fn process_waivers_applies_via_mcp() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("specs/001-x");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        "---\nstatus: in-progress\ndependencies: []\nreview:\n  waivers:\n    \
         - rule: SEC-BE-014\n      file: src/x.ts\n      reason: internal-only endpoint behind mTLS\n      \
         waived-at: 2026-01-01T00:00:00Z\n      waived-by: dev@example.com\n---\n\n# x\n",
    )
    .unwrap();
    fs::create_dir_all(tmp.path().join("src")).unwrap();
    fs::write(tmp.path().join("src/x.ts"), "code\n").unwrap();

    let client = start_pair(tmp.path().to_path_buf()).await;
    let result = call_tool(
        &client,
        "process-waivers",
        json!({
            "feature": "001-x",
            "fired": [{"rule": "SEC-BE-014", "file": "src/x.ts"}],
        }),
    )
    .await;
    let obj = structured_object(&result);
    let applied = obj["applied"].as_array().unwrap();
    assert_eq!(applied.len(), 1);
    assert_eq!(applied[0]["rule"], "SEC-BE-014");
    assert!(obj["expired"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn compute_review_scope_returns_structured_scope_via_mcp() {
    use git2::{IndexAddOption, Repository, Signature};
    let tmp = tempfile::tempdir().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();
    let dir = tmp.path().join("specs/001-x");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        "---\nstatus: planned\ndependencies: []\n---\n\n# x\n",
    )
    .unwrap();
    let mut index = repo.index().unwrap();
    index.add_all(["*"], IndexAddOption::DEFAULT, None).unwrap();
    index.write().unwrap();
    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    let sig = Signature::now("Test", "test@example.com").unwrap();
    repo.commit(Some("HEAD"), &sig, &sig, "init", &tree, &[])
        .unwrap();

    let client = start_pair(tmp.path().to_path_buf()).await;
    let result = call_tool(&client, "compute-review-scope", json!({"feature": "001-x"})).await;
    let obj = structured_object(&result);
    // Never reached in-progress → empty diff-base and scope, but the wire
    // returns the structured shape.
    assert!(obj["diff-base"].is_string());
    assert!(obj["scope"].is_array());
    assert!(obj["captured-issues"].is_array());
}

#[tokio::test]
async fn write_review_renders_report_via_mcp() {
    let tmp = tempfile::tempdir().unwrap();
    let dir = tmp.path().join("specs/001-x");
    fs::create_dir_all(&dir).unwrap();
    fs::write(
        dir.join("spec.md"),
        "---\nstatus: in-progress\ndependencies: []\n---\n\n# x\n",
    )
    .unwrap();

    let client = start_pair(tmp.path().to_path_buf()).await;
    let result = call_tool(
        &client,
        "write-review",
        json!({
            "feature": "001-x",
            "reviewed-at": "2026-07-06T00:00:00Z",
            "reviewed-against": "abc1234",
            "diff-base": "def5678",
            "findings": [{
                "rule": "SEC-BE-001", "severity": "must", "file": "src/a.rs",
                "line-range": "1-5", "confidence": "high"
            }],
        }),
    )
    .await;
    let obj = structured_object(&result);
    assert_eq!(obj["must-violations"], 1);
    assert_eq!(obj["blocking"], true);
    assert!(
        tmp.path().join("specs/001-x/review.md").exists(),
        "write-review must render review.md"
    );
}
