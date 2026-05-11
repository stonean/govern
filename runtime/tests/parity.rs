//! Per-command parity-test harness.
//!
//! Each rewritten command has a `parity:` frontmatter field declaring how
//! its runtime-driven output should be compared against the LLM-driven
//! capture under `runtime/tests/parity/<command>/expected.txt`. This
//! harness:
//!
//! 1. Stages each command's fixture (`runtime/tests/fixtures/<fixture>/`)
//!    plus the canonical `framework/commands/<command>.md` into a fresh
//!    tempdir.
//! 2. Runs `runtime exec <command>` against the tempdir and captures
//!    stdout.
//! 3. Asserts the captured stream equals the golden JSONL under
//!    `runtime/tests/golden/<fixture>.jsonl`, byte-for-byte after the
//!    `{{runtime-version}}` placeholder is substituted with the
//!    `CARGO_PKG_VERSION` baked into the binary at build time.
//! 4. Reads the per-command parity bound from the command file's
//!    frontmatter and the parity capture. When the capture is still a
//!    TODO placeholder, the parity comparison is reported SKIPPED via
//!    `eprintln!` and the test still passes — capture is a maintainer
//!    step gated on having an LLM-driven host available. When the
//!    capture is present, the parity bound is recorded for future
//!    application by the harness (host-side rendering is not yet wired
//!    in; see the spec's "parity captures are manual" trade-off).
//!
//! The shape of `ParitySpec` mirrors the `parity:` keys documented in
//! the spec's per-command rewrites — additions here must keep the
//! command files in sync.
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde::Deserialize;

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct CommandFrontmatter {
    #[serde(default)]
    parity: Option<ParitySpec>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[allow(dead_code)] // Fields are surfaced via Debug-formatting in the SKIPPED message and
// by future bound-application code (see the harness module docs).
struct ParitySpec {
    #[serde(default)]
    strict_stdout: Option<bool>,
    #[serde(default)]
    strict_files: Option<Vec<String>>,
    #[serde(default)]
    strict_fields: Option<Vec<String>>,
    #[serde(default)]
    semantic_fields: Option<Vec<String>>,
}

#[test]
fn status_basic_stream_matches_golden() {
    run_parity_case("status", "status-basic");
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("runtime/.. exists")
        .to_path_buf()
}

fn runtime_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/release/runtime")
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
}

fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to);
        } else {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::copy(&from, &to).unwrap();
        }
    }
}

fn stage_fixture(command: &str, fixture: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().unwrap();
    let repo = repo_root();
    let fixture_root = repo.join("runtime/tests/fixtures").join(fixture);
    assert!(
        fixture_root.is_dir(),
        "missing fixture dir: {}",
        fixture_root.display()
    );
    copy_dir_recursive(&fixture_root, tmp.path());

    let command_src = repo
        .join("framework/commands")
        .join(format!("{command}.md"));
    assert!(
        command_src.is_file(),
        "missing canonical command file: {}",
        command_src.display()
    );
    let command_dst_dir = tmp.path().join("framework/commands");
    fs::create_dir_all(&command_dst_dir).unwrap();
    fs::copy(&command_src, command_dst_dir.join(format!("{command}.md"))).unwrap();

    tmp
}

fn read_parity_spec(command: &str) -> ParitySpec {
    let path = repo_root()
        .join("framework/commands")
        .join(format!("{command}.md"));
    let source = fs::read_to_string(&path).unwrap();
    let body = source
        .strip_prefix("---\n")
        .or_else(|| source.strip_prefix("---\r\n"))
        .unwrap_or_else(|| panic!("no frontmatter in {}", path.display()));
    let end = body
        .find("\n---\n")
        .or_else(|| body.find("\n---\r\n"))
        .unwrap_or_else(|| panic!("no closing frontmatter fence in {}", path.display()));
    let frontmatter = &body[..end];
    let parsed: CommandFrontmatter = serde_yaml::from_str(frontmatter).unwrap_or_else(|err| {
        panic!(
            "failed to parse parity frontmatter in {}: {err}",
            path.display()
        )
    });
    parsed.parity.unwrap_or_default()
}

fn read_golden(fixture: &str) -> String {
    let path = repo_root()
        .join("runtime/tests/golden")
        .join(format!("{fixture}.jsonl"));
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn read_parity_capture(command: &str) -> String {
    let path = repo_root()
        .join("runtime/tests/parity")
        .join(command)
        .join("expected.txt");
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn run_parity_case(command: &str, fixture: &str) {
    ensure_binary_built();
    let bin = runtime_binary();
    let staged = stage_fixture(command, fixture);

    let mut child = Command::new(&bin)
        .arg("exec")
        .arg(command)
        .current_dir(staged.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn runtime");

    // No host input — the parity fixtures exercise only deterministic
    // primitives. Tests that need to deliver llm-response or gate-response
    // envelopes write them to stdin before this call.
    if let Some(mut stdin) = child.stdin.take() {
        let _ = stdin.write_all(&[]);
    }

    let output = child.wait_with_output().expect("wait for runtime");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "runtime exec {command} exited with {:?}\nstderr:\n{stderr}",
        output.status
    );

    let actual = String::from_utf8(output.stdout).expect("utf-8 stdout");
    let golden = read_golden(fixture);
    let expanded = golden.replace("{{runtime-version}}", env!("CARGO_PKG_VERSION"));
    assert_eq!(
        actual, expanded,
        "stream mismatch for {fixture}.jsonl — re-bless by overwriting the golden with the captured stdout",
    );

    let spec = read_parity_spec(command);
    let capture = read_parity_capture(command);
    if capture.contains("TODO:") {
        eprintln!(
            "[parity] {command}: expected.txt is still a TODO placeholder — SKIPPED (bound={spec:?})"
        );
    } else {
        // Future: render envelopes through the host renderer and apply
        // the per-bound comparison from `spec`. For now this branch
        // exercises the read path so a captured file at least round-trips.
        eprintln!(
            "[parity] {command}: capture present (len={} bytes); per-bound comparison not yet wired",
            capture.len()
        );
    }
}
