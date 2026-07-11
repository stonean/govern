//! Integration coverage for the `/gov:review` command procedure.
//!
//! Parses the shipped `framework/commands/review.md` and asserts it walks the
//! review-runtime-acceleration procedure end-to-end: `compute-review-scope` →
//! `discover-rule-files` → five `performReview` passes → `process-waivers` →
//! `write-review`. `process-waivers` runs after the passes so waivers are
//! classified against real findings (spec 022 scenario
//! waiver-processing-order) — the pre-pass ordering mass-expired every waiver
//! against an empty `fired` set. This is the parity check for the command
//! rewrite (spec 022 scenario review-runtime-acceleration, tasks 45g/45i) —
//! it fails if a future edit drops a primitive, reorders the passes, or lets
//! the file regress to legacy prose.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::path::PathBuf;

use gvrn::parser::parse;
use gvrn::schema::procedure::Step;

fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is the runtime crate; the workspace root is its parent.
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_path_buf()
}

/// Reduce a step to `(number, kind:name)` for order-preserving comparison.
fn label(step: &Step) -> (String, String) {
    let num = |n: &gvrn::schema::procedure::StepNumber| {
        n.0.iter().map(u32::to_string).collect::<Vec<_>>().join(".")
    };
    match step {
        Step::Primitive { number, name, .. } => (num(number), format!("primitive:{name}")),
        Step::Extension {
            number, identifier, ..
        } => (num(number), format!("extension:{identifier}")),
        Step::Prose { number, .. } => (num(number), "prose".to_string()),
    }
}

#[test]
fn review_command_parses_to_the_expected_procedure() {
    let source =
        std::fs::read_to_string(workspace_root().join("framework/commands/review.md")).unwrap();
    let procedure = parse(&source, "review").expect("review.md must parse as a Procedure");

    let steps: Vec<(String, String)> = procedure.steps.iter().map(label).collect();
    let expected: Vec<(String, String)> = [
        ("1", "primitive:compute-review-scope"),
        ("2", "primitive:discover-rule-files"),
        ("3", "extension:performReview"),
        ("4", "extension:performReview"),
        ("5", "extension:performReview"),
        ("6", "extension:performReview"),
        ("7", "extension:performReview"),
        ("8", "primitive:process-waivers"),
        ("9", "primitive:write-review"),
    ]
    .into_iter()
    .map(|(n, k)| (n.to_string(), k.to_string()))
    .collect();

    assert_eq!(steps, expected);
}
