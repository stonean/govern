//! `append-task` — append a numbered task block to a feature's `tasks.md`.
//!
//! Computes the next task number as `max(existing) + 1` so a tasks file with
//! `## 1.`, `## 3.` headings produces `## 4.` rather than overwriting `## 3.`.
//! Creates `tasks.md` with a derived heading when absent. Atomic write via
//! tempfile + rename.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::primitives::{
    PrimitiveError, Result, iter_numbered_headings, parse_atx_heading, read_text, rel_path,
    split_frontmatter, validate_no_traversal, write_atomic,
};
use crate::schema::primitives::{AppendTaskArgs, AppendTaskResult};

/// Execute the `append-task` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeaturePathNotFound`] when the resolved feature
/// directory does not exist, or [`PrimitiveError::Io`] for filesystem
/// failures.
pub fn run(args: &AppendTaskArgs, repo: &Path) -> Result<AppendTaskResult> {
    validate_no_traversal(&args.feature_path)?;
    // Q1 resolution: when body is omitted, slug is required. Refuse cleanly
    // rather than silently doubling the slug from the title (the bug the
    // 022/runtime-primitive-structural-bugs scenario closed).
    if args.body.is_none() && args.slug.is_none() {
        return Err(PrimitiveError::MissingArgument {
            primitive: "append-task".into(),
            argument: "slug".into(),
            reason:
                "the default body needs a slug to fill scenarios/{slug}.md; pass either 'slug' or an explicit 'body'"
                    .into(),
        });
    }
    let feature_dir = repo.join(&args.feature_path);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeaturePathNotFound {
            path: PathBuf::from(&args.feature_path),
        });
    }

    let tasks_path = feature_dir.join("tasks.md");
    let (existing, created_now) = match read_text(&tasks_path) {
        Ok(text) => (text, false),
        Err(PrimitiveError::Io { source, .. }) if source.kind() == std::io::ErrorKind::NotFound => {
            let heading = derive_tasks_heading(&feature_dir);
            let intro = if feature_dir.join("plan.md").exists() {
                "Tasks derived from the [plan](plan.md). Complete in order."
            } else {
                "Tasks. Complete in order."
            };
            (format!("{heading}\n\n{intro}\n"), true)
        }
        Err(err) => return Err(err),
    };

    let next_number = next_task_number(&existing);
    let block = render_task_block(next_number, args);
    let new_content = stitch(&existing, &block);
    write_atomic(&tasks_path, &new_content)?;

    Ok(AppendTaskResult {
        task_number: next_number,
        path: rel_path(&tasks_path, repo),
        created: created_now,
    })
}

/// Return `max(existing-task-number) + 1` from the ATX-2 numbered headings
/// in `tasks.md`. Delegates to the shared `iter_numbered_headings` helper so
/// `read-tasks` and `append-task` agree on how to recognize task headings
/// (including the fenced-block skip).
fn next_task_number(content: &str) -> u32 {
    iter_numbered_headings(content).max().unwrap_or(0) + 1
}

/// Render the appended task block. Always preceded by a blank-line separator
/// when stitched onto the existing file.
fn render_task_block(number: u32, args: &AppendTaskArgs) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "## {number}. {}\n", args.title);
    if let Some(items) = &args.body {
        for item in items {
            let _ = writeln!(out, "- [ ] {}", item.trim());
        }
    } else if let Some(slug) = &args.slug {
        // Default single sub-item. The "scenarios/{slug}.md" pointer mirrors
        // the convention `/gov:ask`'s scenario branch uses; `slug` comes from
        // the explicit argument (required when body is omitted; see Q1).
        let _ = writeln!(
            out,
            "- [ ] Implement the behavior described in `scenarios/{slug}.md`"
        );
    }
    // The `(None, None)` branch is unreachable — run() refuses that
    // combination before calling this function. No `else` arm here so the
    // invariant is enforced by the caller, not by a panic in render code.
    out.push('\n');
    let _ = writeln!(out, "- **Done when**: {}", args.done_when.trim());
    out
}

/// Append `block` to `existing`, ensuring exactly one blank line of
/// separation and that the final file ends with a single trailing newline.
fn stitch(existing: &str, block: &str) -> String {
    let trimmed = existing.trim_end_matches(['\n', '\r']);
    let mut out = String::with_capacity(trimmed.len() + block.len() + 4);
    out.push_str(trimmed);
    out.push_str("\n\n");
    out.push_str(block);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

/// Read the feature's spec to compose the new tasks.md H1 ("# NNN — Feature
/// Tasks"). Falls back to a minimal heading when the spec cannot be read.
fn derive_tasks_heading(feature_dir: &Path) -> String {
    if let Ok(spec) = read_text(&feature_dir.join("spec.md")) {
        if let Ok((_fm, body)) = split_frontmatter(&spec, &feature_dir.join("spec.md")) {
            for line in body.lines() {
                if let Some((level, text)) = parse_atx_heading(line) {
                    if level == 1 {
                        return format!("# {text} Tasks");
                    }
                }
            }
        }
    }
    "# Tasks".to_string()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn args(feature_path: &str, title: &str, done_when: &str) -> AppendTaskArgs {
        AppendTaskArgs {
            feature_path: feature_path.into(),
            title: title.into(),
            done_when: done_when.into(),
            body: None,
            // Explicit slug is required when body is None; tests that
            // exercise default-body behavior pass a clean slug here.
            slug: Some(slug_default_for(title)),
        }
    }

    /// Test helper: produce a sensible default slug for the test's title so
    /// the default-body assertions remain readable. Production callers pass
    /// `slug` explicitly; this helper is only for compactness in tests.
    fn slug_default_for(title: &str) -> String {
        title
            .split([':', ' '])
            .rfind(|part| !part.is_empty())
            .unwrap_or("scenario")
            .to_lowercase()
    }

    fn make_feature_with_spec(tmp: &Path, feature_path: &str, h1: &str) {
        fs::create_dir_all(tmp.join(feature_path)).unwrap();
        let body = format!("---\nstatus: in-progress\ndependencies: []\n---\n\n# {h1}\n\nIntro.\n");
        fs::write(tmp.join(feature_path).join("spec.md"), body).unwrap();
    }

    #[test]
    fn appends_to_existing_tasks() {
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let tasks_path = tmp.path().join("specs/042-foo/tasks.md");
        fs::write(
            &tasks_path,
            "# 042 — Foo Tasks\n\n## 1. First\n\n- [x] do thing\n\n- **Done when**: it is done.\n",
        )
        .unwrap();
        let result = run(
            &args(
                "specs/042-foo",
                "Implement scenario: retry",
                "the scenario is implemented.",
            ),
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.task_number, 2);
        assert!(!result.created);
        let body = fs::read_to_string(&tasks_path).unwrap();
        assert!(body.contains("## 1. First"));
        assert!(body.contains("## 2. Implement scenario: retry"));
        assert!(body.contains("- [ ] Implement the behavior described in `scenarios/retry.md`"));
        assert!(body.contains("- **Done when**: the scenario is implemented."));
    }

    #[test]
    fn next_number_uses_max_not_count() {
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let tasks_path = tmp.path().join("specs/042-foo/tasks.md");
        fs::write(
            &tasks_path,
            "# Tasks\n\n## 1. First\n\n## 3. Third (with a gap)\n",
        )
        .unwrap();
        let result = run(&args("specs/042-foo", "Fourth", "done."), tmp.path()).unwrap();
        assert_eq!(result.task_number, 4);
    }

    #[test]
    fn creates_tasks_md_when_absent_using_spec_heading() {
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let result = run(
            &args("specs/042-foo", "Bootstrap", "the crate builds."),
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.task_number, 1);
        assert!(result.created);
        let body = fs::read_to_string(tmp.path().join("specs/042-foo/tasks.md")).unwrap();
        assert!(body.starts_with("# 042 — Foo Tasks"));
        assert!(body.contains("## 1. Bootstrap"));
    }

    #[test]
    fn creates_tasks_md_with_fallback_heading_when_spec_unreadable() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("specs/042-foo")).unwrap();
        let result = run(&args("specs/042-foo", "First", "done."), tmp.path()).unwrap();
        assert!(result.created);
        let body = fs::read_to_string(tmp.path().join("specs/042-foo/tasks.md")).unwrap();
        assert!(body.starts_with("# Tasks"));
        assert!(body.contains("## 1. First"));
    }

    #[test]
    fn uses_explicit_body_when_supplied() {
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let mut a = args("specs/042-foo", "Manual", "done.");
        a.body = Some(vec!["Sub-item one".into(), "Sub-item two".into()]);
        // When body is supplied, slug is ignored.
        a.slug = None;
        run(&a, tmp.path()).unwrap();
        let body = fs::read_to_string(tmp.path().join("specs/042-foo/tasks.md")).unwrap();
        assert!(body.contains("- [ ] Sub-item one"));
        assert!(body.contains("- [ ] Sub-item two"));
        assert!(!body.contains("- [ ] Implement the behavior"));
    }

    #[test]
    fn refuses_when_body_and_slug_both_omitted() {
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let mut a = args("specs/042-foo", "Implement scenario: retry", "done.");
        a.slug = None;
        a.body = None;
        let err = run(&a, tmp.path()).unwrap_err();
        assert!(
            matches!(&err, PrimitiveError::MissingArgument { primitive, argument, .. }
                if primitive == "append-task" && argument == "slug"),
            "expected MissingArgument for slug, got: {err:?}"
        );
    }

    #[test]
    fn explicit_slug_drives_default_body_not_title() {
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let mut a = args(
            "specs/042-foo",
            "Implement scenarios/living-specs.md",
            "done.",
        );
        // The title carries scenarios/...md text; under the old bug the
        // primitive would derive a broken slug from it ("scenarios/living-specs.md"
        // → doubled prefix/extension). With the explicit slug arg, the body
        // points at the canonical scenarios/{slug}.md path.
        a.slug = Some("living-specs".into());
        a.body = None;
        run(&a, tmp.path()).unwrap();
        let body = fs::read_to_string(tmp.path().join("specs/042-foo/tasks.md")).unwrap();
        assert!(
            body.contains("- [ ] Implement the behavior described in `scenarios/living-specs.md`"),
            "expected clean scenarios/living-specs.md pointer, got:\n{body}"
        );
        // No doubled prefix or extension.
        assert!(
            !body.contains("scenarios/scenarios/"),
            "doubled prefix slipped in"
        );
        assert!(
            !body.contains(".md.md"),
            "doubled extension slipped in: {body}"
        );
    }

    #[test]
    fn refuses_when_feature_path_is_missing() {
        let tmp = tempdir().unwrap();
        let err = run(&args("specs/999-nope", "x", "done."), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::FeaturePathNotFound { .. }));
    }

    #[test]
    fn ignores_task_numbers_inside_fenced_code() {
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let tasks_path = tmp.path().join("specs/042-foo/tasks.md");
        fs::write(
            &tasks_path,
            "# Tasks\n\n## 1. First\n\n```text\n## 99. fake\n```\n",
        )
        .unwrap();
        let result = run(&args("specs/042-foo", "Second", "done."), tmp.path()).unwrap();
        assert_eq!(result.task_number, 2);
    }

    #[test]
    fn dropping_named_tempfile_leaves_target_unchanged() {
        use std::io::Write;
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let tasks_path = tmp.path().join("specs/042-foo/tasks.md");
        fs::write(&tasks_path, "# Tasks\n\n## 1. First\n").unwrap();
        let original = fs::read_to_string(&tasks_path).unwrap();
        {
            let parent = tasks_path.parent().unwrap();
            let mut tf = tempfile::NamedTempFile::new_in(parent).unwrap();
            tf.write_all(b"INTERRUPTED").unwrap();
        }
        assert_eq!(original, fs::read_to_string(&tasks_path).unwrap());
    }

    #[test]
    fn refuses_when_feature_path_has_parent_component() {
        let tmp = tempdir().unwrap();
        let err = run(&args("specs/../target", "x", "done."), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidPath { .. }));
    }

    #[test]
    fn refuses_when_feature_path_is_absolute() {
        let tmp = tempdir().unwrap();
        let err = run(&args("/tmp/x", "x", "done."), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidPath { .. }));
    }

    #[test]
    fn newly_created_tasks_omits_plan_link_when_plan_missing() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("specs/042-foo")).unwrap();
        // No spec.md, no plan.md — only the feature dir.
        run(&args("specs/042-foo", "First", "done."), tmp.path()).unwrap();
        let body = fs::read_to_string(tmp.path().join("specs/042-foo/tasks.md")).unwrap();
        assert!(!body.contains("[plan](plan.md)"));
        assert!(body.contains("Tasks. Complete in order."));
    }

    #[test]
    fn newly_created_tasks_includes_plan_link_when_plan_present() {
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        fs::write(tmp.path().join("specs/042-foo/plan.md"), "# Plan\n").unwrap();
        run(&args("specs/042-foo", "First", "done."), tmp.path()).unwrap();
        let body = fs::read_to_string(tmp.path().join("specs/042-foo/tasks.md")).unwrap();
        assert!(body.contains("[plan](plan.md)"));
    }
}
