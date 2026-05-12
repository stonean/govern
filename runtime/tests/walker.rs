//! End-to-end walker integration test.
//!
//! Builds a synthetic procedure that exercises every step kind
//! (Primitive, Extension, Prose-with-gate-trigger, Prose-noop) and runs
//! the walker against mocked stdin/stdout buffers. Asserts the expected
//! JSON envelope sequence.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::io::Cursor;
use std::path::PathBuf;

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
