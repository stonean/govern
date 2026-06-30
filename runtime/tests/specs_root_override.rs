//! Integration test (spec 040): every primitive that takes a bare *feature
//! name* resolves the spec-root directory from `.govern.toml`
//! `[paths] specs-root` instead of the hardcoded `specs/`.
//!
//! A repo configured with `specs-root = "governance"` is driven through the
//! filesystem primitives, and a stray default-named `specs/` tree is asserted
//! to be ignored. The git-backed primitives (`derive-boundary`, `check-stuck`)
//! and the cross-service `resolve-references` primitive are covered by
//! renamed-root unit tests inside their own modules, where the git/checkout
//! harness already lives.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::Path;

use gvrn::primitives;
use gvrn::schema::primitives::{
    DashboardArgs, MarkCriterionArgs, MarkTaskArgs, ReadSpecArgs, ReadTasksArgs, SetStatusArgs,
    TraverseDepsArgs,
};

const GOVERNANCE_TOML: &str = "[paths]\nspecs-root = \"governance\"\n";

fn write(path: &Path, body: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(path, body).unwrap();
}

fn spec_body(status: &str) -> String {
    format!(
        "---\nstatus: {status}\ndependencies: []\n---\n\n# Demo\n\n## Acceptance Criteria\n\n- [ ] First.\n- [ ] Second.\n"
    )
}

const TASKS_BODY: &str = "# Demo\n\n## 1. Bootstrap\n\n- [ ] Subtask one.\n- [ ] Subtask two.\n";

/// Seed a repo whose spec root is `governance`, with one feature plus a stray
/// default-named `specs/` tree that must never be consulted.
fn seed(repo: &Path) {
    write(&repo.join(".govern.toml"), GOVERNANCE_TOML);
    write(
        &repo.join("governance/001-demo/spec.md"),
        &spec_body("in-progress"),
    );
    write(&repo.join("governance/001-demo/tasks.md"), TASKS_BODY);
    // Decoy: same feature slug under the default root, and a stray feature.
    write(&repo.join("specs/001-demo/spec.md"), &spec_body("draft"));
    write(&repo.join("specs/999-stray/spec.md"), &spec_body("done"));
}

#[test]
fn read_spec_resolves_configured_root() {
    let tmp = tempfile::tempdir().unwrap();
    seed(tmp.path());
    let result = primitives::read_spec::run(
        &ReadSpecArgs {
            feature: "001-demo".into(),
            include_body: false,
        },
        tmp.path(),
    )
    .unwrap();
    // The resolved path is under governance/, proving the decoy specs/ copy
    // was not the one read.
    assert_eq!(result.path, "governance/001-demo/spec.md");
    assert_eq!(result.frontmatter.status, "in-progress");
}

#[test]
fn read_tasks_resolves_configured_root() {
    let tmp = tempfile::tempdir().unwrap();
    seed(tmp.path());
    let result = primitives::read_tasks::run(
        &ReadTasksArgs {
            feature: "001-demo".into(),
        },
        tmp.path(),
    )
    .unwrap();
    assert_eq!(result.path, "governance/001-demo/tasks.md");
}

#[test]
fn set_status_writes_under_configured_root() {
    let tmp = tempfile::tempdir().unwrap();
    seed(tmp.path());
    let result = primitives::set_status::run(
        &SetStatusArgs {
            feature: "001-demo".into(),
            from: "in-progress".into(),
            to: "done".into(),
        },
        tmp.path(),
    )
    .unwrap();
    assert_eq!(result.current, "done");
    assert_eq!(result.path, "governance/001-demo/spec.md");
    // Written under governance/, while the decoy specs/ copy is untouched.
    let governed = fs::read_to_string(tmp.path().join("governance/001-demo/spec.md")).unwrap();
    assert!(governed.contains("status: done"));
    let decoy = fs::read_to_string(tmp.path().join("specs/001-demo/spec.md")).unwrap();
    assert!(
        decoy.contains("status: draft"),
        "decoy specs/ copy untouched"
    );
}

#[test]
fn mark_task_resolves_configured_root() {
    let tmp = tempfile::tempdir().unwrap();
    seed(tmp.path());
    let result = primitives::mark_task::run(
        &MarkTaskArgs {
            feature: "001-demo".into(),
            task_number: "1".into(),
            subtask_index: 0,
            checked: true,
        },
        tmp.path(),
    )
    .unwrap();
    assert_eq!(result.path, "governance/001-demo/tasks.md");
    assert!(result.current);
}

#[test]
fn mark_criterion_resolves_configured_root() {
    let tmp = tempfile::tempdir().unwrap();
    seed(tmp.path());
    let result = primitives::mark_criterion::run(
        &MarkCriterionArgs {
            feature: "001-demo".into(),
            criterion_index: 0,
            checked: true,
        },
        tmp.path(),
    )
    .unwrap();
    assert_eq!(result.path, "governance/001-demo/spec.md");
    assert!(result.current);
}

#[test]
fn traverse_deps_resolves_configured_root() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    write(&repo.join(".govern.toml"), GOVERNANCE_TOML);
    write(
        &repo.join("governance/002-consumer/spec.md"),
        "---\nstatus: in-progress\ndependencies: [001-demo]\n---\n\n# Consumer\n",
    );
    write(
        &repo.join("governance/001-demo/spec.md"),
        &spec_body("done"),
    );
    let result = primitives::traverse_deps::run(
        &TraverseDepsArgs {
            feature: "002-consumer".into(),
        },
        repo,
    )
    .unwrap();
    assert_eq!(result.dependencies.len(), 1);
    assert!(result.dependencies[0].exists, "dep found under governance/");
    assert!(result.dependencies[0].compatible);
    assert!(result.compatible);
}

#[test]
fn dashboard_enumerates_configured_root_and_ignores_stray_specs() {
    let tmp = tempfile::tempdir().unwrap();
    seed(tmp.path());
    let result = primitives::dashboard::run(&DashboardArgs {}, tmp.path()).unwrap();
    let slugs: Vec<&str> = result.specs.iter().map(|s| s.slug.as_str()).collect();
    assert!(
        slugs.contains(&"001-demo"),
        "governance feature present: {slugs:?}"
    );
    assert!(
        !slugs.contains(&"999-stray"),
        "stray specs/ tree ignored: {slugs:?}"
    );
}

#[test]
fn unset_setting_keeps_default_specs_root() {
    // No `.govern.toml` → the default `specs` root, unchanged behavior.
    let tmp = tempfile::tempdir().unwrap();
    write(
        &tmp.path().join("specs/001-demo/spec.md"),
        &spec_body("done"),
    );
    let result = primitives::read_spec::run(
        &ReadSpecArgs {
            feature: "001-demo".into(),
            include_body: false,
        },
        tmp.path(),
    )
    .unwrap();
    assert_eq!(result.path, "specs/001-demo/spec.md");
}
