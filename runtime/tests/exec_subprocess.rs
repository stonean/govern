//! End-to-end test for the `runtime exec <command>` subprocess
//! interpreter surface. Spawns the release binary as a real subprocess,
//! pipes stdin/stdout, and exercises a synthetic procedure file from
//! a tempfile repo.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

mod common;
use common::copy_dir_recursive;

fn runtime_binary() -> PathBuf {
    // CARGO_MANIFEST_DIR is the runtime crate. The release binary lives
    // under target/release/gvrn relative to it.
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/release/gvrn")
}

fn write_procedure_repo(tmp: &Path, command_name: &str, body: &str) {
    let cmd_dir = tmp.join("framework/commands");
    fs::create_dir_all(&cmd_dir).unwrap();
    fs::write(cmd_dir.join(format!("{command_name}.md")), body).unwrap();
}

fn ensure_binary_built() {
    let binary = runtime_binary();
    if binary.exists() {
        return;
    }
    let status = Command::new("cargo")
        .args(["build", "--release"])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .status()
        .expect("cargo build --release must succeed");
    assert!(status.success(), "cargo build failed");
    assert!(
        binary.exists(),
        "binary not produced at {}",
        binary.display()
    );
}

#[test]
fn exec_drives_a_deterministic_procedure_to_complete() {
    ensure_binary_built();

    // Set up a fresh repo that contains:
    //   - framework/commands/smoke.md — a parseable procedure
    //   - a fixture feature `001-basic` with spec + tasks so `read-spec`
    //     and `read-tasks` succeed.
    let tmp = tempfile::tempdir().unwrap();
    write_procedure_repo(
        tmp.path(),
        "smoke",
        "# /gov:smoke\n\n## Instructions\n\n1. Invoke `read-spec` against the target.\n2. Invoke `read-tasks` against the target.\n",
    );
    let feature_dir = tmp.path().join("specs/001-basic");
    fs::create_dir_all(&feature_dir).unwrap();
    fs::write(
        feature_dir.join("spec.md"),
        "---\nstatus: clarified\ndependencies: []\n---\n\n# 001\n\nbody.\n",
    )
    .unwrap();
    fs::write(
        feature_dir.join("tasks.md"),
        "# 001\n\n## 1. First\n\n- [ ] Only subtask.\n- **Done when**: done.\n",
    )
    .unwrap();

    let mut child = Command::new(runtime_binary())
        .arg("exec")
        .arg("smoke")
        .arg("feature=001-basic")
        .current_dir(tmp.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn runtime");

    // No input needed — the procedure has no extension points or gates.
    drop(child.stdin.take());

    let stdout = child.stdout.take().unwrap();
    let mut envelopes: Vec<Value> = Vec::new();
    for line in BufReader::new(stdout).lines() {
        let line = line.unwrap();
        if line.trim().is_empty() {
            continue;
        }
        envelopes.push(serde_json::from_str(&line).unwrap());
    }
    let status = child.wait().unwrap();
    assert!(status.success(), "exit: {status:?}");

    // Expect: progress(read-spec), progress(read-tasks), complete.
    let types: Vec<&str> = envelopes
        .iter()
        .map(|v| v["type"].as_str().unwrap())
        .collect();
    assert_eq!(types, vec!["progress", "progress", "complete"]);
    assert_eq!(envelopes[0]["primitive"], "read-spec");
    assert_eq!(envelopes[1]["primitive"], "read-tasks");
    assert!(envelopes[2]["runtime-version"].is_string());
}

#[test]
fn exec_reads_extension_response_from_stdin() {
    ensure_binary_built();
    let tmp = tempfile::tempdir().unwrap();
    write_procedure_repo(
        tmp.path(),
        "ext",
        "# /gov:ext\n\n## Instructions\n\n1. <!-- llm:writeCode --> Ask the LLM to write code.\n",
    );

    let mut child = Command::new(runtime_binary())
        .arg("exec")
        .arg("ext")
        .current_dir(tmp.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn runtime");

    // Pre-write the host response. The walker generates request-id `req-1`
    // deterministically for the first extension point. The payload must
    // satisfy WriteCodeResponse — edits + summary are both required.
    let mut stdin = child.stdin.take().unwrap();
    stdin
        .write_all(
            b"{\"type\":\"llm-response\",\"request-id\":\"req-1\",\"response\":{\"edits\":[],\"summary\":\"no-op\"}}\n",
        )
        .unwrap();
    drop(stdin);

    let stdout = child.stdout.take().unwrap();
    let mut envelopes: Vec<Value> = Vec::new();
    for line in BufReader::new(stdout).lines() {
        let line = line.unwrap();
        if line.trim().is_empty() {
            continue;
        }
        envelopes.push(serde_json::from_str(&line).unwrap());
    }
    let status = child.wait().unwrap();
    assert!(status.success(), "exit: {status:?}");

    let types: Vec<&str> = envelopes
        .iter()
        .map(|v| v["type"].as_str().unwrap())
        .collect();
    assert_eq!(types, vec!["llm-request", "progress", "complete"]);
    assert_eq!(envelopes[0]["extension-point"], "writeCode");
    assert_eq!(envelopes[0]["request-id"], "req-1");
}

#[test]
fn exec_chains_bootstrap_primitives_extract_substitute_merge() {
    ensure_binary_built();
    // Walks the back half of the bootstrap procedure end-to-end:
    // a synthetic gvrn-exec target invokes extract-archive on a
    // committed-shape (test-built) tarball, then substitute-templates
    // over the staged tree, then merge-claude-md against CLAUDE.md.
    // The full procedure also includes fetch-archive (HTTP); that
    // first step needs a mock server and is deferred — its unit tests
    // cover hash-verification independently.
    let tmp = tempfile::tempdir().unwrap();

    // Build a tiny tarball inline so the fixture stays text-only in git.
    let tarball_path = tmp.path().join("framework.tar.gz");
    {
        let file = fs::File::create(&tarball_path).unwrap();
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(encoder);
        let template = b"# {project}\n\nProject: {project}\n";
        let mut header = tar::Header::new_gnu();
        header.set_size(template.len() as u64);
        header.set_mode(0o644);
        header.set_cksum();
        builder
            .append_data(&mut header, "README.md", &template[..])
            .unwrap();
        builder.into_inner().unwrap().finish().unwrap();
    }

    // Write the synthetic procedure file under framework/bootstrap/.
    let bootstrap_dir = tmp.path().join("framework/bootstrap");
    fs::create_dir_all(&bootstrap_dir).unwrap();
    fs::write(
        bootstrap_dir.join("install.md"),
        "# /install\n\n## Instructions\n\n1. Invoke `extract-archive` against the staged tarball.\n2. Invoke `substitute-templates` to materialize the project files.\n3. Invoke `merge-claude-md` to install the managed block.\n",
    )
    .unwrap();

    // Seed `.govern.session.toml` with every arg the three primitives
    // need. Post-consolidation, the walker reads this single repo-root
    // file regardless of AI CLI / project name.
    let session_toml = format!(
        "archive = {archive:?}\n\
         dest = {dest:?}\n\
         source-dir = {source_dir:?}\n\
         target-dir = {target_dir:?}\n\
         path = {path:?}\n\
         block = \"framework managed block\\nproject = anvil\"\n\
         \n\
         [substitutions]\n\
         project = \"anvil\"\n",
        archive = tarball_path.to_string_lossy().to_string(),
        dest = tmp.path().join("staging").to_string_lossy().to_string(),
        source_dir = tmp.path().join("staging").to_string_lossy().to_string(),
        target_dir = tmp.path().join("project").to_string_lossy().to_string(),
        path = tmp.path().join("CLAUDE.md").to_string_lossy().to_string(),
    );
    let session_path = tmp.path().join(".govern.session.toml");
    let mut sf = fs::File::create(&session_path).unwrap();
    sf.write_all(session_toml.as_bytes()).unwrap();

    let child = Command::new(runtime_binary())
        .arg("exec")
        .arg("install")
        .current_dir(tmp.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("spawn runtime");
    let stderr = String::from_utf8_lossy(&child.stderr);
    assert!(
        child.status.success(),
        "exit {:?}\nstderr:\n{stderr}\nstdout:\n{}",
        child.status,
        String::from_utf8_lossy(&child.stdout)
    );

    // Verify the chain's observable effects:
    // - extract-archive wrote README.md into the staging dir
    // - substitute-templates wrote a substituted copy into the project dir
    // - merge-claude-md created CLAUDE.md with the managed block
    assert!(tmp.path().join("staging/README.md").exists());
    let written = fs::read_to_string(tmp.path().join("project/README.md")).unwrap();
    assert!(
        written.contains("# anvil") && written.contains("Project: anvil"),
        "substitution didn't take effect: {written}"
    );
    let claude = fs::read_to_string(tmp.path().join("CLAUDE.md")).unwrap();
    assert!(
        claude.contains("<!-- BEGIN govern-managed -->")
            && claude.contains("framework managed block")
            && claude.contains("project = anvil"),
        "CLAUDE.md missing managed block:\n{claude}"
    );
}

#[test]
fn exec_resolves_bootstrap_procedure_under_framework_bootstrap() {
    ensure_binary_built();
    // Bootstrap procedures live at framework/bootstrap/<name>.md so the
    // /govern installer can be invoked before any framework/commands/
    // files exist in the adopter's project. The runtime falls back to
    // this third candidate path when the first two don't resolve.
    let tmp = tempfile::tempdir().unwrap();
    let bootstrap_dir = tmp.path().join("framework/bootstrap");
    fs::create_dir_all(&bootstrap_dir).unwrap();
    fs::write(
        bootstrap_dir.join("govern.md"),
        "# /govern\n\n## Instructions\n\n1. Invoke `read-spec` against the targeted feature.\n",
    )
    .unwrap();

    let feature_dir = tmp.path().join("specs/001-basic");
    fs::create_dir_all(&feature_dir).unwrap();
    fs::write(
        feature_dir.join("spec.md"),
        "---\nstatus: clarified\ndependencies: []\n---\n\n# 001\n\nbody.\n",
    )
    .unwrap();

    let mut child = Command::new(runtime_binary())
        .arg("exec")
        .arg("govern")
        .arg("feature=001-basic")
        .current_dir(tmp.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn runtime");
    drop(child.stdin.take());

    let stdout = child.stdout.take().unwrap();
    let mut envelopes: Vec<Value> = Vec::new();
    for line in BufReader::new(stdout).lines() {
        let line = line.unwrap();
        if line.trim().is_empty() {
            continue;
        }
        envelopes.push(serde_json::from_str(&line).unwrap());
    }
    let status = child.wait().unwrap();
    assert!(status.success(), "exit: {status:?}");
    let types: Vec<&str> = envelopes
        .iter()
        .map(|v| v["type"].as_str().unwrap())
        .collect();
    assert_eq!(types, vec!["progress", "complete"]);
    assert_eq!(envelopes[0]["primitive"], "read-spec");
}

#[test]
fn exec_resolves_command_via_parameterized_host_block() {
    ensure_binary_built();
    // Auggie-shaped adopter project — no `framework/commands/` tree,
    // command file at `.augment/commands/anvil/smoke.md`, `.govern.toml`
    // declares `[host] cli-config-dir = ".augment"` and `project =
    // "anvil"`. With the parameterized lookup wired up, the runtime
    // reads the [host] block and resolves the second candidate path
    // accordingly; without it, the second candidate would be the
    // hardcoded `.claude/commands/gov/smoke.md` (which doesn't exist
    // in this fixture) and the run would fail with "command file not
    // found".
    let tmp = tempfile::tempdir().unwrap();
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/exec-auggie");
    copy_dir_recursive(&fixture, tmp.path());

    let mut child = Command::new(runtime_binary())
        .arg("exec")
        .arg("smoke")
        .arg("feature=001-basic")
        .current_dir(tmp.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn runtime");
    drop(child.stdin.take());

    let stdout = child.stdout.take().unwrap();
    let mut envelopes: Vec<Value> = Vec::new();
    for line in BufReader::new(stdout).lines() {
        let line = line.unwrap();
        if line.trim().is_empty() {
            continue;
        }
        envelopes.push(serde_json::from_str(&line).unwrap());
    }
    let stderr_buf = child.stderr.take().unwrap();
    let stderr_str: String = BufReader::new(stderr_buf)
        .lines()
        .map_while(Result::ok)
        .collect::<Vec<_>>()
        .join("\n");
    let status = child.wait().unwrap();
    assert!(
        status.success(),
        "exit: {status:?}\nstderr:\n{stderr_str}\nenvelopes: {envelopes:?}"
    );
    let types: Vec<&str> = envelopes
        .iter()
        .map(|v| v["type"].as_str().unwrap())
        .collect();
    assert_eq!(types, vec!["progress", "complete"]);
    assert_eq!(envelopes[0]["primitive"], "read-spec");
}

#[test]
fn exec_returns_nonzero_when_command_file_missing() {
    ensure_binary_built();
    let tmp = tempfile::tempdir().unwrap();
    let status = Command::new(runtime_binary())
        .arg("exec")
        .arg("nonexistent")
        .current_dir(tmp.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .status()
        .expect("spawn runtime");
    assert!(
        !status.success(),
        "expected nonzero exit on missing command"
    );
}
