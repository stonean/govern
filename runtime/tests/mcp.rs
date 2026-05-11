//! Integration test for the MCP server surface.
//!
//! Starts a `GovRuntimeServer` in-process on one half of a `tokio::io::duplex`
//! pair and connects an `rmcp` client on the other half. The test:
//!
//! 1. Lists tools and asserts every name in `TOOL_NAMES` is present.
//! 2. Asserts the tool set is a superset of `framework/runtime-tools.txt`
//!    (the canonical manifest).
//! 3. Invokes each read-only primitive against the shared fixture repo and
//!    asserts the response is a structured JSON object.
//! 4. Invokes the write primitives against per-test scratch copies of the
//!    fixture so each test is hermetic.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use rmcp::ServiceExt;
use rmcp::model::CallToolRequestParam;
use serde_json::{Value, json};

use govern_runtime::mcp::server::{GovRuntimeServer, TOOL_NAMES};

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
        .call_tool(CallToolRequestParam {
            name: name.to_string().into(),
            arguments: Some(args_obj),
        })
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

    let manifest = fs::read_to_string(workspace_root().join("framework/runtime-tools.txt"))
        .unwrap_or_default();
    let manifest_names: Vec<String> = manifest
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(str::to_string)
        .collect();
    for expected in &manifest_names {
        assert!(
            names.contains(&expected.as_str()),
            "tool {expected} from framework/runtime-tools.txt missing (got {names:?})"
        );
    }
}

#[tokio::test]
async fn read_spec_returns_structured_frontmatter() {
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(
        &client,
        "gov-rt:read-spec",
        json!({"feature": "001-basic", "include-body": false}),
    )
    .await;
    let obj = structured_object(&result);
    assert_eq!(obj["frontmatter"]["status"], "clarified");
    assert_eq!(obj["path"], "specs/001-basic/spec.md");
}

#[tokio::test]
async fn read_tasks_returns_task_list() {
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(
        &client,
        "gov-rt:read-tasks",
        json!({"feature": "001-basic"}),
    )
    .await;
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
        "gov-rt:validate-frontmatter",
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
        "gov-rt:resolve-anchor",
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
        "gov-rt:traverse-deps",
        json!({"feature": "002-dependent"}),
    )
    .await;
    let obj = structured_object(&result);
    assert!(obj["compatible"].is_boolean());
}

#[tokio::test]
async fn check_rule_ids_returns_citations() {
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(
        &client,
        "gov-rt:check-rule-ids",
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
async fn gate_confirm_returns_prompt_payload_without_blocking() {
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(
        &client,
        "gov-rt:gate-confirm",
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
        "gov-rt:mark-task",
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
        "gov-rt:mark-criterion",
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
        "gov-rt:check-stuck",
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
    let result = call_tool(
        &client,
        "gov-rt:derive-boundary",
        json!({"feature": "001-basic"}),
    )
    .await;
    let obj = structured_object(&result);
    assert!(obj["boundary"].is_array());
    assert!(obj["first-commit"].is_string());
}

#[tokio::test]
async fn run_generator_invokes_bash_script() {
    let tmp = tempfile::tempdir().unwrap();
    let script = tmp.path().join("gen.sh");
    fs::write(&script, "#!/usr/bin/env bash\necho ok\nexit 0\n").unwrap();
    let mut perms = fs::metadata(&script).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&script, perms).unwrap();
    let client = start_pair(tmp.path().to_path_buf()).await;
    let result = call_tool(&client, "gov-rt:run-generator", json!({"script": "gen.sh"})).await;
    let obj = structured_object(&result);
    assert_eq!(obj["drift"], false);
    assert_eq!(obj["exit-code"], 0);
}

#[tokio::test]
async fn lint_markdown_returns_violations_array() {
    let client = start_pair(fixture_repo()).await;
    let result = call_tool(
        &client,
        "gov-rt:lint-markdown",
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
        "gov-rt:set-status",
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
