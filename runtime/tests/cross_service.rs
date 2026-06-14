//! Cross-service reference resolution: golden records + markdown-only parity.
//!
//! Spec 030's deterministic resolution work — resolve each `references:`
//! entry against the `.govern.toml` `[services]` registry, read the linked
//! spec's `status` from its local checkout, classify the outcome — runs
//! through the `resolve-references` runtime primitive when the runtime is
//! installed, and identically via host file tools on the markdown-only path.
//! Because the classification is fully deterministic (no prose read for
//! intent), both paths produce byte-identical resolution records.
//!
//! This test pins that contract two ways against the `cross-service-basic`
//! fixture, whose consumer spec exercises every outcome:
//!
//!   1. The runtime path: `resolve_references::run` serialized one
//!      `ResolutionRecord` per line equals `golden/cross-service-basic.jsonl`.
//!   2. Byte-identity: that golden equals the markdown-only capture under
//!      `parity/cross-service/expected.txt` — the records a host produces by
//!      following the command prose with file tools. A drift in either path
//!      (a re-blessed golden without an updated capture, or vice versa) fails
//!      the check.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::PathBuf;

use gvrn::primitives::resolve_references;
use gvrn::schema::primitives::ResolveReferencesArgs;

fn manifest_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn cross_service_basic_records_match_golden_and_markdown_only_capture() {
    let repo = manifest_dir().join("tests/fixtures/cross-service-basic");
    let args = ResolveReferencesArgs {
        feature: "001-consumer".into(),
    };
    let result = resolve_references::run(&args, &repo).expect("resolve-references runs clean");

    // One ResolutionRecord per line, in the consumer's `references:` order.
    let mut actual = String::new();
    for record in &result.references {
        actual.push_str(&serde_json::to_string(record).expect("serialize record"));
        actual.push('\n');
    }

    let golden = fs::read_to_string(manifest_dir().join("tests/golden/cross-service-basic.jsonl"))
        .expect("read golden");
    assert_eq!(
        actual, golden,
        "runtime resolution records must match golden/cross-service-basic.jsonl — \
         re-bless by overwriting the golden with the captured records"
    );

    // The markdown-only capture must be byte-identical to the runtime golden:
    // both paths share one classification contract and neither wraps the other.
    let markdown_only =
        fs::read_to_string(manifest_dir().join("tests/parity/cross-service/expected.txt"))
            .expect("read markdown-only capture");
    assert_eq!(
        golden, markdown_only,
        "markdown-only capture (parity/cross-service/expected.txt) must be byte-identical \
         to the runtime golden — the two paths produce identical resolution records"
    );
}
