//! End-to-end walker integration test.
//!
//! Builds a synthetic procedure that exercises every step kind
//! (Primitive, Extension, Prose-with-gate-trigger, Prose-noop) and runs
//! the walker against mocked stdin/stdout buffers. Asserts the expected
//! JSON envelope sequence.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::io::Cursor;
use std::path::{Path, PathBuf};

use git2::{IndexAddOption, Repository, Signature};
use serde_json::{Map, Value};

use gvrn::interpreter::{WalkOutcome, Walker};
use gvrn::schema::procedure::{Procedure, SourceRange, Step, StepNumber};

fn loc() -> SourceRange {
    SourceRange {
        start_line: 1,
        start_col: 1,
        end_line: 1,
        end_col: 1,
    }
}

fn fixture_repo() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/primitives/sample-repo")
}

/// Stage everything under `repo` and commit; returns the new commit sha.
fn commit_all(repo: &Repository, message: &str) -> String {
    let mut index = repo.index().unwrap();
    index.add_all(["*"], IndexAddOption::DEFAULT, None).unwrap();
    index.write().unwrap();
    let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
    let sig = Signature::now("Test", "test@example.com").unwrap();
    let parent = repo
        .head()
        .ok()
        .and_then(|h| h.target())
        .and_then(|oid| repo.find_commit(oid).ok());
    let parents: Vec<&git2::Commit> = parent.as_ref().into_iter().collect();
    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
        .unwrap()
        .to_string()
}

/// Write `body` to `path`, creating parent directories as needed.
fn write_file(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(path, body).unwrap();
}

#[test]
fn walks_a_procedure_exercising_every_step_kind() {
    let procedure = Procedure {
        command: "smoke".into(),
        steps: vec![
            // Step 1: prose noop.
            Step::Prose {
                number: StepNumber(vec![1]),
                text: "Preamble — do nothing observable.".into(),
                location: loc(),
            },
            // Step 2: real primitive against the sample-repo fixture.
            Step::Primitive {
                number: StepNumber(vec![2]),
                name: "read-spec".into(),
                prose: "Invoke `read-spec` for the targeted feature.".into(),
                location: loc(),
            },
            // Step 3: extension point — host echoes back a response.
            Step::Extension {
                number: StepNumber(vec![3]),
                identifier: "assessSpecQuality".into(),
                prose: "Ask the LLM to assess spec quality.".into(),
                location: loc(),
            },
            // Step 4: gate trigger via prose.
            Step::Prose {
                number: StepNumber(vec![4]),
                text: "Ask the user to approve the result.".into(),
                location: loc(),
            },
        ],
    };

    let mut context = Map::new();
    context.insert("feature".into(), Value::String("001-basic".into()));
    context.insert("include-body".into(), Value::Bool(false));

    // Mock the host: it must answer two prompts in order — one
    // llm-response (request-id req-1) and one gate-response
    // (request-id req-2). Counter starts at 1 per Walker::fresh_request_id.
    let host_responses = "\
        {\"type\":\"llm-response\",\"request-id\":\"req-1\",\"response\":{\"passed\":true}}\n\
        {\"type\":\"gate-response\",\"request-id\":\"req-2\",\"confirmed\":true}\n\
    ";

    let mut reader = Cursor::new(host_responses.to_string());
    let mut writer: Vec<u8> = Vec::new();
    let mut walker = Walker::new(
        &procedure,
        fixture_repo(),
        context,
        &mut reader,
        &mut writer,
    );
    let outcome = walker.run().unwrap();
    assert_eq!(outcome, WalkOutcome::Complete);

    let envelopes: Vec<Value> = std::str::from_utf8(&writer)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();

    // Expected sequence:
    //   progress(read-spec dispatch)           — step 2
    //   llm-request(assessSpecQuality)         — step 3
    //   progress(received llm-response)        — step 3
    //   gate-confirm(step-4)                   — step 4
    //   progress(gate confirmed)               — step 4
    //   complete                               — end
    let types: Vec<&str> = envelopes
        .iter()
        .map(|v| v["type"].as_str().unwrap())
        .collect();
    assert_eq!(
        types,
        vec![
            "progress",     // step 2 dispatch
            "llm-request",  // step 3
            "progress",     // step 3 received
            "gate-confirm", // step 4
            "progress",     // step 4 confirmed
            "complete",     // end
        ]
    );

    // Step-number annotations and primitive markers land on the right
    // envelopes.
    assert_eq!(envelopes[0]["primitive"], "read-spec");
    assert_eq!(envelopes[0]["step"], "2");
    assert_eq!(envelopes[1]["extension-point"], "assessSpecQuality");
    assert_eq!(envelopes[1]["request-id"], "req-1");
    assert_eq!(envelopes[3]["request-id"], "req-2");
    assert_eq!(envelopes[3]["gate"], "step-4");
    assert!(envelopes[5]["runtime-version"].is_string());
}

#[test]
fn walker_halts_on_primitive_failure_with_error_envelope() {
    let procedure = Procedure {
        command: "fail-fast".into(),
        steps: vec![
            Step::Primitive {
                number: StepNumber(vec![1]),
                name: "read-spec".into(),
                prose: String::new(),
                location: loc(),
            },
            Step::Primitive {
                number: StepNumber(vec![2]),
                name: "read-tasks".into(),
                prose: String::new(),
                location: loc(),
            },
        ],
    };
    let mut context = Map::new();
    context.insert("feature".into(), Value::String("999-missing".into()));

    let mut reader = Cursor::new(String::new());
    let mut writer: Vec<u8> = Vec::new();
    let mut walker = Walker::new(
        &procedure,
        fixture_repo(),
        context,
        &mut reader,
        &mut writer,
    );
    let outcome = walker.run().unwrap();
    match outcome {
        WalkOutcome::Errored { code, .. } => assert_eq!(code, "primitive-failure"),
        WalkOutcome::Complete => panic!("expected Errored, got Complete"),
    }

    let envelopes: Vec<Value> = std::str::from_utf8(&writer)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let types: Vec<&str> = envelopes
        .iter()
        .map(|v| v["type"].as_str().unwrap())
        .collect();
    // First primitive's dispatch progress, then error. The second
    // primitive never gets touched.
    assert_eq!(types, vec!["progress", "error"]);
    assert_eq!(envelopes[1]["code"], "primitive-failure");
    assert!(envelopes[1]["runtime-version"].is_string());
}

/// Task 46a ABI test: a primitive's structured result threads through the
/// walker context to a later extension's payload builder and a later
/// primitive. Walks the review pipeline shape
/// `compute-review-scope → discover-rule-files → performReview → write-review`
/// and asserts that `compute-review-scope`'s `scope`/`diff-base` and
/// `discover-rule-files`'s `selected`/`rules-dir` reach
/// `build_perform_review_request` (as `scope-files`/`rule-files`) and
/// `write-review` (which renders `diff-base` and the accumulated findings).
#[test]
fn review_primitive_results_thread_into_perform_review_and_write_review() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = Repository::init(tmp.path()).unwrap();
    let spec = |status: &str| format!("---\nstatus: {status}\ndependencies: []\n---\n\n# X\n");
    let spec_path = tmp.path().join("specs/001-x/spec.md");

    // History: 001-x goes planned → in-progress (the diff-base commit),
    // then a source file is added — the review scope. The rule file lands
    // in the first commit so it predates diff-base and stays out of scope.
    write_file(&spec_path, &spec("planned"));
    write_file(
        &tmp.path().join("framework/rules/security-backend.md"),
        "# Security\n\n- **SEC-BE-001**: no secrets in logs.\n",
    );
    commit_all(&repo, "feat: plan");
    write_file(&spec_path, &spec("in-progress"));
    let diff_base = commit_all(&repo, "chore: begin");
    write_file(&tmp.path().join("src/a.rs"), "fn a() {}\n");
    commit_all(&repo, "feat: implement");

    let procedure = Procedure {
        command: "review".into(),
        steps: vec![
            Step::Primitive {
                number: StepNumber(vec![1]),
                name: "compute-review-scope".into(),
                prose: String::new(),
                location: loc(),
            },
            Step::Primitive {
                number: StepNumber(vec![2]),
                name: "discover-rule-files".into(),
                prose: String::new(),
                location: loc(),
            },
            Step::Extension {
                number: StepNumber(vec![3]),
                identifier: "performReview".into(),
                prose: "Run the security pass.".into(),
                location: loc(),
            },
            Step::Primitive {
                number: StepNumber(vec![4]),
                name: "write-review".into(),
                prose: String::new(),
                location: loc(),
            },
        ],
    };

    // Seed only the values the review pipeline can't derive itself.
    // `diff-base` is deliberately NOT seeded — it must come from
    // `compute-review-scope` for this test to prove the threading.
    let mut context = Map::new();
    context.insert("feature".into(), Value::String("001-x".into()));
    context.insert(
        "reviewed-at".into(),
        Value::String("2026-07-06T00:00:00Z".into()),
    );
    context.insert("reviewed-against".into(), Value::String("headsha0".into()));
    context.insert("pass".into(), Value::String("security".into()));

    // One security pass returns a single MUST finding against the scoped file.
    let responses = "{\"type\":\"llm-response\",\"request-id\":\"req-1\",\"response\":{\"findings\":[{\"rule\":\"SEC-BE-001\",\"severity\":\"must\",\"file\":\"src/a.rs\",\"line-range\":\"1\",\"confidence\":\"high\"}]}}\n";

    let mut reader = Cursor::new(responses.to_string());
    let mut writer: Vec<u8> = Vec::new();
    let mut walker = Walker::new(
        &procedure,
        tmp.path().to_path_buf(),
        context,
        &mut reader,
        &mut writer,
    );
    assert_eq!(walker.run().unwrap(), WalkOutcome::Complete);

    let envelopes: Vec<Value> = std::str::from_utf8(&writer)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();

    // `compute-review-scope`'s `scope` and `discover-rule-files`'s
    // `selected`/`rules-dir` reached `build_perform_review_request`.
    let request = envelopes
        .iter()
        .find(|e| e["type"] == "llm-request" && e["extension-point"] == "performReview")
        .expect("a performReview llm-request was emitted");
    let scope_files = request["request"]["scope-files"].as_array().unwrap();
    assert!(
        scope_files.iter().any(|f| f["path"] == "src/a.rs"),
        "compute-review-scope `scope` threaded into the performReview payload: {scope_files:?}"
    );
    let rule_files = request["request"]["rule-files"].as_array().unwrap();
    assert!(
        rule_files
            .iter()
            .any(|f| f["name"] == "security-backend.md"),
        "discover-rule-files `selected`/`rules-dir` threaded into the payload: {rule_files:?}"
    );

    // `write-review` consumed `compute-review-scope`'s `diff-base` (rendered
    // to review.md frontmatter) and the accumulated performReview `findings`.
    let review = std::fs::read_to_string(tmp.path().join("specs/001-x/review.md")).unwrap();
    assert!(
        review.contains(&format!("diff-base: {diff_base}")),
        "diff-base threaded from compute-review-scope into write-review:\n{review}"
    );
    assert!(
        review.contains("SEC-BE-001"),
        "accumulated performReview findings reached write-review:\n{review}"
    );
    let spec_after = std::fs::read_to_string(&spec_path).unwrap();
    assert!(
        spec_after.contains("blocking: true"),
        "the MUST finding set blocking in the spec review block:\n{spec_after}"
    );
}

/// Protocol robustness (data-model §JSON-over-stdio ignore-and-continue):
/// while suspended awaiting an `llm-response`, a wrong-type envelope, a
/// malformed line, a blank keepalive line, and a response for a superseded
/// request-id are each logged to stderr and skipped — the walk still
/// completes once the valid response arrives, with no `error` envelope.
#[test]
fn stray_and_malformed_stdin_lines_are_ignored_while_awaiting_llm_response() {
    let procedure = Procedure {
        command: "noise".into(),
        steps: vec![Step::Extension {
            number: StepNumber(vec![1]),
            identifier: "assessSpecQuality".into(),
            prose: String::new(),
            location: loc(),
        }],
    };

    // Noise before the valid response: wrong-type envelope, malformed JSON,
    // blank keepalive, then an llm-response for a superseded request-id.
    let host_lines = "\
        {\"type\":\"gate-response\",\"request-id\":\"req-1\",\"confirmed\":true}\n\
        this is not json\n\
        \n\
        {\"type\":\"llm-response\",\"request-id\":\"req-0\",\"response\":{\"passed\":false}}\n\
        {\"type\":\"llm-response\",\"request-id\":\"req-1\",\"response\":{\"passed\":true}}\n\
    ";
    let mut reader = Cursor::new(host_lines.to_string());
    let mut writer: Vec<u8> = Vec::new();
    let mut walker = Walker::new(
        &procedure,
        fixture_repo(),
        Map::new(),
        &mut reader,
        &mut writer,
    );
    assert_eq!(walker.run().unwrap(), WalkOutcome::Complete);

    let types: Vec<String> = std::str::from_utf8(&writer)
        .unwrap()
        .lines()
        .map(|l| {
            serde_json::from_str::<Value>(l).unwrap()["type"]
                .as_str()
                .unwrap()
                .to_string()
        })
        .collect();
    // llm-request, progress(received), complete — no error envelope, and
    // the consumed response is the one matching req-1.
    assert_eq!(types, vec!["llm-request", "progress", "complete"]);
}

/// Same ignore-and-continue rule while awaiting a `gate-response`: stray
/// and malformed lines are skipped and the gate still resolves.
#[test]
fn stray_and_malformed_stdin_lines_are_ignored_while_awaiting_gate_response() {
    let procedure = Procedure {
        command: "noise-gate".into(),
        steps: vec![Step::Prose {
            number: StepNumber(vec![1]),
            text: "Ask the user to approve the change.".into(),
            location: loc(),
        }],
    };
    let host_lines = "\
        {\"type\":\"llm-response\",\"request-id\":\"req-1\",\"response\":{}}\n\
        {not json}\n\
        {\"type\":\"gate-response\",\"request-id\":\"req-9\",\"confirmed\":false}\n\
        {\"type\":\"gate-response\",\"request-id\":\"req-1\",\"confirmed\":true}\n\
    ";
    let mut reader = Cursor::new(host_lines.to_string());
    let mut writer: Vec<u8> = Vec::new();
    let mut walker = Walker::new(
        &procedure,
        fixture_repo(),
        Map::new(),
        &mut reader,
        &mut writer,
    );
    assert_eq!(walker.run().unwrap(), WalkOutcome::Complete);

    let envelopes: Vec<Value> = std::str::from_utf8(&writer)
        .unwrap()
        .lines()
        .map(|l| serde_json::from_str(l).unwrap())
        .collect();
    let types: Vec<&str> = envelopes
        .iter()
        .map(|v| v["type"].as_str().unwrap())
        .collect();
    // The superseded req-9 denial was ignored; the matching req-1
    // confirmation resolved the gate and the walk completed.
    assert_eq!(types, vec!["gate-confirm", "progress", "complete"]);
    assert!(
        envelopes[1]["message"]
            .as_str()
            .unwrap()
            .contains("confirmed"),
        "gate resolved from the matching response: {envelopes:?}"
    );
}

/// Stdin EOF while suspended awaiting a response stays an operational
/// error (the one non-ignorable inbound condition).
#[test]
fn stdin_eof_while_awaiting_response_is_an_operational_error() {
    let procedure = Procedure {
        command: "eof".into(),
        steps: vec![Step::Extension {
            number: StepNumber(vec![1]),
            identifier: "assessSpecQuality".into(),
            prose: String::new(),
            location: loc(),
        }],
    };
    // Only noise, then EOF — never a valid response.
    let mut reader = Cursor::new("not json\n".to_string());
    let mut writer: Vec<u8> = Vec::new();
    let mut walker = Walker::new(
        &procedure,
        fixture_repo(),
        Map::new(),
        &mut reader,
        &mut writer,
    );
    let err = walker.run().unwrap_err();
    assert_eq!(err.kind(), std::io::ErrorKind::UnexpectedEof);
}

/// Waiver-processing-order, exec-path binding: when `fired` is not seeded,
/// the walker binds the findings accumulated by the `performReview` passes
/// to `process-waivers`' `fired` argument, so a waiver whose rule still
/// fires classifies as applied — not mass-expired against an empty set.
#[test]
fn process_waivers_binds_fired_from_accumulated_pass_findings() {
    let tmp = tempfile::tempdir().unwrap();
    let spec_dir = tmp.path().join("specs/001-x");
    std::fs::create_dir_all(&spec_dir).unwrap();
    std::fs::write(
        spec_dir.join("spec.md"),
        "---\nstatus: in-progress\ndependencies: []\nreview:\n  waivers:\n    \
         - rule: SEC-BE-001\n      file: src/a.rs\n      \
         reason: \"Internal-only path behind mTLS; rule targets public endpoints\"\n      \
         waived-at: 2026-07-07T00:00:00Z\n      waived-by: test@example.com\n---\n\n# X\n",
    )
    .unwrap();
    std::fs::create_dir_all(tmp.path().join("src")).unwrap();
    std::fs::write(tmp.path().join("src/a.rs"), "fn a() {}\n").unwrap();

    // Corrected review order: the pass produces the findings, then
    // process-waivers classifies against them, then write-review renders.
    let procedure = Procedure {
        command: "review".into(),
        steps: vec![
            Step::Extension {
                number: StepNumber(vec![1]),
                identifier: "performReview".into(),
                prose: String::new(),
                location: loc(),
            },
            Step::Primitive {
                number: StepNumber(vec![2]),
                name: "process-waivers".into(),
                prose: String::new(),
                location: loc(),
            },
            Step::Primitive {
                number: StepNumber(vec![3]),
                name: "write-review".into(),
                prose: String::new(),
                location: loc(),
            },
        ],
    };

    // `fired` is deliberately NOT seeded — it must come from the pass's
    // accumulated `findings` for this test to prove the binding.
    let mut context = Map::new();
    context.insert("feature".into(), Value::String("001-x".into()));
    context.insert(
        "reviewed-at".into(),
        Value::String("2026-07-07T00:00:00Z".into()),
    );
    context.insert("reviewed-against".into(), Value::String("headsha0".into()));
    context.insert("diff-base".into(), Value::String("base0000".into()));

    let responses = "{\"type\":\"llm-response\",\"request-id\":\"req-1\",\"response\":{\"findings\":[{\"rule\":\"SEC-BE-001\",\"severity\":\"must\",\"file\":\"src/a.rs\",\"line-range\":\"1\",\"confidence\":\"high\"}]}}\n";
    let mut reader = Cursor::new(responses.to_string());
    let mut writer: Vec<u8> = Vec::new();
    let mut walker = Walker::new(
        &procedure,
        tmp.path().to_path_buf(),
        context,
        &mut reader,
        &mut writer,
    );
    assert_eq!(walker.run().unwrap(), WalkOutcome::Complete);

    let review = std::fs::read_to_string(spec_dir.join("review.md")).unwrap();
    // The waiver applied (rule still fires per the pass findings), so the
    // MUST is excluded from the blocking count ...
    assert!(
        review.contains("must-violations: 0"),
        "waiver classified as applied against the pass findings:\n{review}"
    );
    // ... and rendered with the waiver reason — proof it was applied, not
    // expired against an empty `fired` set.
    assert!(
        review.contains("Internal-only path behind mTLS"),
        "applied waiver reason reached write-review:\n{review}"
    );
    let spec_after = std::fs::read_to_string(spec_dir.join("spec.md")).unwrap();
    assert!(
        spec_after.contains("blocking: false"),
        "the waived finding does not block:\n{spec_after}"
    );
    assert!(
        spec_after.contains("SEC-BE-001"),
        "the still-valid waiver was not pruned from the spec:\n{spec_after}"
    );
}

/// Regression for the review-exec-wiring waiver gap: `process-waivers` emits
/// `applied`/`expired`, and `write-review` reads them via serde alias, so an
/// applied waiver threads through the walker context (which merges primitive
/// results by bare key) and excludes its finding from the blocking count.
/// Without the alias the waived MUST would re-block on the exec path.
#[test]
fn applied_waiver_threads_from_process_waivers_into_write_review() {
    let tmp = tempfile::tempdir().unwrap();
    let spec_dir = tmp.path().join("specs/001-x");
    std::fs::create_dir_all(&spec_dir).unwrap();
    // Spec carries a waiver anchored at (SEC-BE-001, src/a.rs).
    std::fs::write(
        spec_dir.join("spec.md"),
        "---\nstatus: in-progress\ndependencies: []\nreview:\n  waivers:\n    \
         - rule: SEC-BE-001\n      file: src/a.rs\n      \
         reason: \"Internal-only path behind mTLS; rule targets public endpoints\"\n      \
         waived-at: 2026-07-07T00:00:00Z\n      waived-by: test@example.com\n---\n\n# X\n",
    )
    .unwrap();
    std::fs::create_dir_all(tmp.path().join("src")).unwrap();
    std::fs::write(tmp.path().join("src/a.rs"), "fn a() {}\n").unwrap();

    let procedure = Procedure {
        command: "review".into(),
        steps: vec![
            Step::Primitive {
                number: StepNumber(vec![1]),
                name: "process-waivers".into(),
                prose: String::new(),
                location: loc(),
            },
            Step::Primitive {
                number: StepNumber(vec![2]),
                name: "write-review".into(),
                prose: String::new(),
                location: loc(),
            },
        ],
    };

    let mut context = Map::new();
    context.insert("feature".into(), Value::String("001-x".into()));
    context.insert(
        "reviewed-at".into(),
        Value::String("2026-07-07T00:00:00Z".into()),
    );
    context.insert("reviewed-against".into(), Value::String("headsha0".into()));
    context.insert("diff-base".into(), Value::String("base0000".into()));
    // The firing finding — in a live exec run the performReview passes supply
    // it; seeded here so process-waivers can classify the waiver as applied.
    context.insert(
        "fired".into(),
        serde_json::json!([{ "rule": "SEC-BE-001", "file": "src/a.rs" }]),
    );
    context.insert(
        "findings".into(),
        serde_json::json!([{
            "rule": "SEC-BE-001", "severity": "must", "file": "src/a.rs",
            "line-range": "1", "confidence": "high"
        }]),
    );

    let mut reader = Cursor::new(String::new());
    let mut writer: Vec<u8> = Vec::new();
    let mut walker = Walker::new(
        &procedure,
        tmp.path().to_path_buf(),
        context,
        &mut reader,
        &mut writer,
    );
    assert_eq!(walker.run().unwrap(), WalkOutcome::Complete);

    let review = std::fs::read_to_string(spec_dir.join("review.md")).unwrap();
    // The waived MUST is excluded from the blocking count ...
    assert!(
        review.contains("must-violations: 0"),
        "applied waiver excluded the MUST from the count:\n{review}"
    );
    // ... and rendered with the waiver reason, which only the applied waiver
    // carries — proof it threaded from process-waivers via the `applied` alias.
    assert!(
        review.contains("Internal-only path behind mTLS"),
        "applied waiver reason reached write-review:\n{review}"
    );
    let spec_after = std::fs::read_to_string(spec_dir.join("spec.md")).unwrap();
    assert!(
        spec_after.contains("blocking: false"),
        "a waived finding does not block:\n{spec_after}"
    );
}
