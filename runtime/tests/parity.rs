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

mod common;
use common::copy_dir_recursive;

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

#[test]
fn target_basic_stream_matches_golden() {
    run_parity_case("target", "target-basic");
}

#[test]
fn analyze_basic_stream_matches_golden() {
    run_parity_case("analyze", "analyze-basic");
}

#[test]
fn implement_basic_stream_matches_golden() {
    run_parity_case("implement", "implement-basic");
}

#[test]
fn plan_basic_stream_matches_golden() {
    run_parity_case("plan", "plan-basic");
}

#[test]
fn review_basic_stream_matches_golden() {
    run_parity_case("review", "review-basic");
}

#[test]
fn specify_basic_stream_matches_golden() {
    run_parity_case("specify", "specify-basic");
}

#[test]
fn govern_basic_stream_matches_golden() {
    run_parity_case("install", "govern-basic");
}

#[test]
fn govern_basic_post_run_filesystem_state_matches_expectations() {
    // Companion to govern_basic_stream_matches_golden: that test asserts the
    // JSONL stream is byte-correct, this one walks the post-run on-disk
    // state to verify every primitive's effects landed as designed.
    //
    // Per-entry expectations against
    // runtime/tests/fixtures/govern-basic/.govern.session.toml:
    //
    // - update strategy + substitution → specify.md, feature.md with
    //   `{project}` → "anvil"
    // - skip-if-conflict strategy → AGENTS.md with `{project}` LEFT
    //   LITERAL (substitution suppressed by strategy)
    // - pinned dest → framework/constitution.md preserved verbatim from
    //   the fixture's pre-seeded copy (apply-manifest's `skipped-pinned`)
    // - keep-literals entry → .claude/commands/govern.md with `{project}`
    //   and `{cli-config-dir}` kept LITERAL even though substitutions
    //   are applied to other entries
    // - merge-managed-block (line-prefix style) → .gitignore created
    //   with the `# govern` block
    // - enforce-manifest → legacy-cmd.md REMOVED from the target dir
    ensure_binary_built();
    let bin = runtime_binary();
    let staged = stage_fixture("install", "govern-basic");

    let mut child = Command::new(&bin)
        .arg("exec")
        .arg("install")
        .current_dir(staged.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn runtime");
    // The bootstrap procedure no longer has a confirmation gate — invoking
    // `/govern` is itself the consent. Close stdin without writing.
    drop(child.stdin.take());
    let output = child.wait_with_output().expect("wait");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "runtime exec install failed: {:?}\nstderr:\n{stderr}",
        output.status
    );

    let project = staged.path().join("project");
    let read = |rel: &str| {
        fs::read_to_string(project.join(rel))
            .unwrap_or_else(|err| panic!("read project/{rel}: {err}"))
    };

    // update strategy + substitution applied.
    let specify = read("framework/commands/specify.md");
    assert!(
        specify.contains("# /anvil:specify"),
        "specify.md must substitute {{project}} → anvil: {specify:?}"
    );
    assert!(
        !specify.contains("{project}"),
        "specify.md must have no remaining placeholders: {specify:?}"
    );

    // create strategy + substitution applied (dest absent at start).
    let feature = read("framework/templates/feature.md");
    assert!(
        feature.contains("# anvil feature template"),
        "feature.md must substitute {{project}}: {feature:?}"
    );

    // skip-if-conflict: substitution NOT applied.
    let agents = read("AGENTS.md");
    assert!(
        agents.contains("{project}"),
        "AGENTS.md must keep {{project}} LITERAL (skip-if-conflict suppresses substitution): {agents:?}"
    );

    // Pinned: pre-seeded adopter copy preserved verbatim.
    let constitution = read("framework/constitution.md");
    assert!(
        constitution.contains("Adopter-edited constitution"),
        "pinned constitution must keep adopter content; got: {constitution:?}"
    );
    assert!(
        !constitution.contains("Framework-owned governance rules"),
        "pinned constitution must NOT be overwritten with the staged framework copy"
    );

    // keep-literals entry: placeholders preserved even though
    // substitutions are applied to other entries in the same run.
    let govern_md = read(".claude/commands/govern.md");
    assert!(
        govern_md.contains("{project}") && govern_md.contains("{cli-config-dir}"),
        "govern.md must keep {{project}} and {{cli-config-dir}} LITERAL: {govern_md:?}"
    );

    // merge-managed-block (line-prefix) created .gitignore.
    let gitignore = read(".gitignore");
    assert!(
        gitignore.starts_with("# govern\n"),
        ".gitignore must start with the # govern marker line: {gitignore:?}"
    );
    assert!(
        gitignore.contains(".cache/") && gitignore.contains("staging/"),
        ".gitignore must contain the block body: {gitignore:?}"
    );

    // enforce-manifest pruned the pre-seeded legacy file.
    assert!(
        !project.join("framework/commands/legacy-cmd.md").exists(),
        "enforce-manifest must remove framework/commands/legacy-cmd.md"
    );
    // And kept the file in the expected list.
    assert!(
        project.join("framework/commands/specify.md").exists(),
        "enforce-manifest must keep framework/commands/specify.md (in expected list)"
    );
    // Legacy `framework/skills/` files survive — adopter-cleanup of
    // historical conventions moved out of enforce-manifest's contract
    // into the registry-driven Pre-run Migrations loop (spec 027). The
    // primitive now operates only on the per-agent slash-command
    // directory; any directory outside that scope is untouched.
    assert!(
        project.join("framework/skills/old-skill.md").exists(),
        "enforce-manifest must NOT touch framework/skills/old-skill.md \
         (legacy-cleanup is owned by framework/migrations.toml)"
    );
}

#[test]
fn traverse_deps_cycle_check_surfaces_two_cycle_via_cli() {
    // Parity coverage for spec 022's `traverse-deps-cycle-check` scenario:
    // both the markdown-only walker (the agent reading frontmatter via
    // host tools and feeding traverse-deps via MCP) and the runtime
    // walker (`gvrn traverse-deps` subprocess used by `/gov:analyze` step
    // 3) surface the same finding shape. This test exercises the CLI
    // subprocess surface against a hand-built 2-cycle fixture and
    // asserts the JSON envelope carries the expected `cycles` payload.
    // The companion MCP integration test in `runtime/tests/mcp.rs`
    // exercises the rmcp tool-handler surface against the same shape,
    // so both paths an adopter host might use are covered.
    ensure_binary_built();
    let bin = runtime_binary();
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

    let output = Command::new(&bin)
        .args(["traverse-deps", "--feature", "200-a"])
        .current_dir(tmp.path())
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("traverse-deps subprocess");
    assert!(
        output.status.success(),
        "traverse-deps exited non-zero: {:?}\nstderr: {}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    let json: serde_json::Value =
        serde_json::from_str(stdout.trim()).expect("traverse-deps emits a single JSON object");
    assert_eq!(
        json["compatible"], false,
        "cycle must flip overall compatibility to false"
    );
    let cycles = json["cycles"].as_array().expect("cycles is an array");
    assert_eq!(cycles.len(), 1, "exactly one SCC for the two-node cycle");
    let scc: Vec<String> = cycles[0]
        .as_array()
        .expect("first cycle is an array")
        .iter()
        .map(|v| v.as_str().expect("slug is a string").to_string())
        .collect();
    assert_eq!(scc.len(), 2);
    assert!(scc.contains(&"200-a".to_string()));
    assert!(scc.contains(&"201-b".to_string()));
}

#[test]
fn implement_rejects_out_of_boundary_write_code_edit() {
    ensure_binary_built();
    let bin = runtime_binary();
    let staged = stage_fixture("implement", "implement-basic");

    // Replace the staged stdin.jsonl with a malicious writeCode response
    // that edits a file outside the write boundary. The implement procedure
    // has no planned → in-progress gate, so the writeCode request is
    // `req-1`; its edit's path escapes `specs/004-implement/**` and
    // `runtime/**`.
    let stdin = "{\"type\":\"llm-response\",\"request-id\":\"req-1\",\"response\":{\"edits\":[{\"path\":\"framework/constitution.md\",\"action\":\"edit\",\"content\":\"malicious\"}],\"summary\":\"escape the boundary\"}}\n";

    let mut child = Command::new(&bin)
        .arg("exec")
        .arg("implement")
        .current_dir(staged.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("spawn runtime");
    {
        let mut child_stdin = child.stdin.take().expect("stdin");
        child_stdin
            .write_all(stdin.as_bytes())
            .expect("stdin write");
    }
    let output = child.wait_with_output().expect("wait");
    assert!(
        !output.status.success(),
        "expected non-zero exit on out-of-boundary edit; stdout:\n{}",
        String::from_utf8_lossy(&output.stdout)
    );
    let stdout = String::from_utf8(output.stdout).expect("utf-8 stdout");
    let last_line = stdout.lines().last().unwrap_or_default();
    let envelope: serde_json::Value = serde_json::from_str(last_line)
        .unwrap_or_else(|err| panic!("last line is not JSON: {err}\n{stdout}"));
    assert_eq!(envelope["type"], "error", "final envelope is error");
    assert_eq!(
        envelope["code"], "out-of-boundary-edit",
        "error code surfaces the boundary violation"
    );
    assert!(
        envelope["message"]
            .as_str()
            .unwrap_or_default()
            .contains("framework/constitution.md"),
        "error message names the offending path"
    );
}

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("runtime/.. exists")
        .to_path_buf()
}

fn runtime_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target/release/gvrn")
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

    // Two paths to provide the procedure file the runtime resolves:
    // (a) repo-canonical `framework/commands/<cmd>.md` — copy it in;
    // (b) fixture-local under `framework/bootstrap/<cmd>.md` (for
    //     bootstrap-namespaced procedures the runtime resolves via the
    //     third candidate path) — already copied by copy_dir_recursive.
    let canonical = repo
        .join("framework/commands")
        .join(format!("{command}.md"));
    if canonical.is_file() {
        let command_dst_dir = tmp.path().join("framework/commands");
        fs::create_dir_all(&command_dst_dir).unwrap();
        fs::copy(&canonical, command_dst_dir.join(format!("{command}.md"))).unwrap();
    } else {
        // Confirm the fixture supplied its own procedure file somewhere
        // the runtime knows about.
        let bootstrap = tmp
            .path()
            .join("framework/bootstrap")
            .join(format!("{command}.md"));
        let project_local = tmp
            .path()
            .join(".claude/commands/gov")
            .join(format!("{command}.md"));
        assert!(
            bootstrap.is_file() || project_local.is_file(),
            "no canonical or fixture-local procedure file for {command}"
        );
    }

    // Mock-HTTP setup: when a fixture ships a `mock-http/staging/`
    // subtree, build a tarball at test time, compute its sha256,
    // start a localhost HTTP server, and substitute `{MOCK_HTTP}` in
    // the staged session JSON with the dynamic URL.
    let staging = tmp.path().join("mock-http/staging");
    if staging.is_dir() {
        let (archive_bytes, sha256_hex) = build_tarball_with_sha256(&staging);
        let server = MockHttp::start(vec![
            ("/archive.tar.gz".into(), archive_bytes),
            (
                "/archive.tar.gz.sha256".into(),
                format!("{sha256_hex}  archive.tar.gz\n").into_bytes(),
            ),
        ]);
        substitute_in_session(tmp.path(), "{MOCK_HTTP}", &server.url());
        // MockHttp's listener-loop thread holds its own Arc clone of
        // the routes and runs detached — `server` falling out of scope
        // here is harmless (the type carries no Drop logic); the thread
        // and bound port persist until process exit.
        let _ = server;
    }

    // Primitives that read git history (`derive-boundary`, `check-stuck`)
    // need a real repo. Init one in the tempdir and commit the staged
    // state so every primitive that calls `Repository::discover` finds a
    // valid history. Fixtures that don't exercise those primitives pay a
    // tiny git-init overhead but otherwise are unaffected.
    init_git_repo(tmp.path());

    tmp
}

/// Tar+gzip every regular file under `root` into an in-memory archive,
/// preserving Unix mode bits. Returns the archive bytes plus the
/// lowercase-hex sha256 digest.
fn build_tarball_with_sha256(root: &Path) -> (Vec<u8>, String) {
    use sha2::{Digest, Sha256};
    let buffer: Vec<u8> = Vec::new();
    let encoder = flate2::write::GzEncoder::new(buffer, flate2::Compression::default());
    let mut builder = tar::Builder::new(encoder);
    builder.follow_symlinks(false);
    builder
        .append_dir_all(".", root)
        .expect("append staging tree");
    let archive_bytes = builder
        .into_inner()
        .expect("finish tar")
        .finish()
        .expect("finish gzip");
    let mut hasher = Sha256::new();
    hasher.update(&archive_bytes);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut hex, "{byte:02x}");
    }
    (archive_bytes, hex)
}

/// Read the staged `.govern.session.toml`, replace every occurrence of
/// `placeholder` with `replacement`, and write the result back. No-op
/// when the session file is absent.
fn substitute_in_session(root: &Path, placeholder: &str, replacement: &str) {
    let path = root.join(".govern.session.toml");
    if !path.is_file() {
        return;
    }
    let text = fs::read_to_string(&path).unwrap();
    let replaced = text.replace(placeholder, replacement);
    fs::write(&path, replaced).unwrap();
}

/// Minimal HTTP/1.1 server for parity fixtures that exercise
/// `fetch-archive`. Binds to 127.0.0.1:0 (kernel-assigned port),
/// services one route per request, ignores headers, and closes the
/// connection after responding.
struct MockHttp {
    addr: std::net::SocketAddr,
}

impl MockHttp {
    fn start(routes: Vec<(String, Vec<u8>)>) -> Self {
        use std::net::TcpListener;
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind mock http");
        let addr = listener.local_addr().expect("local addr");
        let routes = std::sync::Arc::new(routes);
        std::thread::spawn(move || {
            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };
                let routes = routes.clone();
                std::thread::spawn(move || {
                    let _ = MockHttp::handle(stream, &routes);
                });
            }
        });
        Self { addr }
    }

    fn url(&self) -> String {
        format!("http://{}", self.addr)
    }

    fn handle(
        mut stream: std::net::TcpStream,
        routes: &[(String, Vec<u8>)],
    ) -> std::io::Result<()> {
        use std::io::{BufRead, BufReader, Write};
        let peek = stream.try_clone()?;
        let mut reader = BufReader::new(peek);
        let mut request_line = String::new();
        reader.read_line(&mut request_line)?;
        let path = request_line
            .split_whitespace()
            .nth(1)
            .unwrap_or("/")
            .to_string();
        // Drain headers — read lines until the blank CRLF terminator.
        let mut hdr = String::new();
        while reader.read_line(&mut hdr)? > 0 {
            if hdr.trim().is_empty() {
                break;
            }
            hdr.clear();
        }
        if let Some((_, body)) = routes.iter().find(|(p, _)| *p == path) {
            let header = format!(
                "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: application/octet-stream\r\nConnection: close\r\n\r\n",
                body.len()
            );
            stream.write_all(header.as_bytes())?;
            stream.write_all(body)?;
        } else {
            let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            stream.write_all(resp.as_bytes())?;
        }
        stream.flush()?;
        let _ = stream.shutdown(std::net::Shutdown::Both);
        Ok(())
    }
}

fn init_git_repo(path: &Path) {
    use git2::{IndexAddOption, Repository, Signature, Time};
    let repo = Repository::init(path).expect("git init");
    let mut index = repo.index().unwrap();
    index
        .add_all(["*"], IndexAddOption::DEFAULT, None)
        .expect("git add");
    index.write().expect("index write");
    let tree_id = index.write_tree().expect("write tree");
    let tree = repo.find_tree(tree_id).unwrap();
    // A FIXED-time signature (not `Signature::now`) makes the commit sha
    // deterministic. Since the result-threading merge (spec 022 task 46)
    // surfaces git-derived values into extension payloads — e.g.
    // `derive-boundary`'s `first-commit`/`current-head` land in the
    // writeCode request — the sha must be byte-stable across runs and
    // machines or the golden would be flaky. A frozen `Time` plus the fixed
    // tree, message, and (absent) parents fully determine the commit OID.
    let when = Time::new(1_704_067_200, 0); // 2024-01-01T00:00:00Z
    let sig = Signature::new("Parity Test", "parity@example.com", &when).expect("signature");
    repo.commit(Some("HEAD"), &sig, &sig, "chore: fixture", &tree, &[])
        .expect("initial commit");
}

fn read_parity_spec(command: &str, fixture: &str) -> ParitySpec {
    // Prefer the repo-canonical command file; fall back to a fixture-
    // local procedure file for bootstrap-namespaced fixtures.
    let canonical = repo_root()
        .join("framework/commands")
        .join(format!("{command}.md"));
    let path = if canonical.is_file() {
        canonical
    } else {
        repo_root()
            .join("runtime/tests/fixtures")
            .join(fixture)
            .join("framework/bootstrap")
            .join(format!("{command}.md"))
    };
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
    let parsed: CommandFrontmatter = serde_norway::from_str(frontmatter).unwrap_or_else(|err| {
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

    // Optional `stdin.jsonl` under the fixture supplies pre-canned host
    // responses (llm-response, gate-response) for procedures that include
    // extension points or gates.
    let fixture_stdin = repo_root()
        .join("runtime/tests/fixtures")
        .join(fixture)
        .join("stdin.jsonl");
    if let Some(mut stdin) = child.stdin.take() {
        if fixture_stdin.is_file() {
            let payload = fs::read(&fixture_stdin).unwrap();
            stdin.write_all(&payload).expect("write stdin payload");
        }
        drop(stdin);
    }

    let output = child.wait_with_output().expect("wait for runtime");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "runtime exec {command} exited with {:?}\nstderr:\n{stderr}",
        output.status
    );

    let actual = String::from_utf8(output.stdout).expect("utf-8 stdout");

    // Optional `BLESS=1` env var: overwrite the golden file with the
    // captured actual stream (templating the current runtime version
    // back to `{{runtime-version}}`) instead of asserting. Used when a
    // protocol-level change requires bulk re-blessing of the corpus.
    if std::env::var("BLESS").as_deref() == Ok("1") {
        let golden_path = repo_root()
            .join("runtime/tests/golden")
            .join(format!("{fixture}.jsonl"));
        let templated = actual.replace(env!("CARGO_PKG_VERSION"), "{{runtime-version}}");
        fs::write(&golden_path, templated).expect("write blessed golden");
        return;
    }

    let golden = read_golden(fixture);
    let expanded = golden.replace("{{runtime-version}}", env!("CARGO_PKG_VERSION"));
    assert_eq!(
        actual, expanded,
        "stream mismatch for {fixture}.jsonl — re-bless by overwriting the golden with the captured stdout (or re-run with BLESS=1 to overwrite all goldens)",
    );

    let spec = read_parity_spec(command, fixture);
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
