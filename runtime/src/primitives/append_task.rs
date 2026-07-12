//! `append-task` — append a numbered task block to a feature's `tasks.md`.
//!
//! Computes the next task number as `max(existing) + 1` so a tasks file with
//! `## 1.`, `## 3.` headings produces `## 4.` rather than overwriting `## 3.`.
//! Creates `tasks.md` with a derived heading when absent. Atomic write via
//! tempfile + rename.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::primitives::{
    PhaseRange, PrimitiveError, Result, TasksStructure, detect_tasks_structure, iter_phase_ranges,
    iter_task_numbers_at_levels, parse_atx_heading, read_text, rel_path, split_frontmatter,
    validate_no_traversal, validate_slug, write_atomic,
};
use crate::schema::primitives::{AppendTaskArgs, AppendTaskResult};

/// Execute the `append-task` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::InvalidArgument`] when `title`, `done-when`,
/// or a `body` item carries an embedded newline (structure injection into
/// `tasks.md`), [`PrimitiveError::FeaturePathNotFound`] when the resolved
/// feature directory does not exist, or [`PrimitiveError::Io`] for
/// filesystem failures.
pub fn run(args: &AppendTaskArgs, repo: &Path) -> Result<AppendTaskResult> {
    validate_no_traversal(&args.feature_path)?;
    // Text arguments are interpolated verbatim into tasks.md line
    // templates; an embedded newline would smuggle extra markdown
    // structure (phantom headings, checkboxes, Done-when lines) past the
    // renderer. Reject rather than flatten — the caller sees exactly
    // which argument to fix (scenario primitive-robustness-hardening).
    validate_single_line("title", &args.title)?;
    validate_single_line("done-when", &args.done_when)?;
    if let Some(items) = &args.body {
        for (idx, item) in items.iter().enumerate() {
            validate_single_line(&format!("body[{idx}]"), item)?;
        }
    }
    // BE-INPUT-001: the `slug` argument is interpolated verbatim into the
    // default-body `scenarios/{slug}.md` line, but the other text arguments
    // above were the only ones screened. Validate it against the slug-grammar
    // allowlist (BE-INPUT-002) — mirroring `create-scenario` — so a slug like
    // `x\n## 99. Phantom` cannot smuggle a path segment or a phantom task
    // heading into tasks.md. Runs before the feature-dir check so the refusal
    // never touches disk.
    if let Some(slug) = &args.slug {
        validate_slug(slug)?;
    }
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
    let new_content = match detect_tasks_structure(&existing) {
        TasksStructure::Flat => {
            let block = render_flat_task_block(next_number, args);
            stitch(&existing, &block)
        }
        TasksStructure::Phased => insert_phased_task(&existing, next_number, args)?,
    };
    write_atomic(&tasks_path, &new_content)?;

    Ok(AppendTaskResult {
        task_number: next_number,
        path: rel_path(&tasks_path, repo),
        created: created_now,
    })
}

/// Reject a single-line text argument that carries an embedded newline
/// (`\n` or `\r`). The renderer interpolates these values into one-line
/// templates; a newline injects task structure.
fn validate_single_line(argument: &str, value: &str) -> Result<()> {
    super::validate_single_line("append-task", argument, value)
}

/// Return `max(existing-task-number) + 1` across both flat (`## N.`) and
/// phased (`### N.`) task headings in `tasks.md`. Walking both levels makes
/// the numbering robust to mixed-structure files: an old `## 1.` flat task
/// alongside a new `### 2.` phased task produces `next = 3`, not `next = 2`.
fn next_task_number(content: &str) -> u32 {
    iter_task_numbers_at_levels(content, &[2, 3])
        .max()
        .unwrap_or(0)
        + 1
}

/// Render the flat task block at ATX-2 (`## N. Title`). Always preceded
/// by a blank-line separator when stitched onto the existing file.
fn render_flat_task_block(number: u32, args: &AppendTaskArgs) -> String {
    render_task_block(number, args, /* heading_level= */ 2)
}

/// Render a task block at the requested heading level. Phased tasks live
/// at level 3 (`### N. Title`) under `## …` phase containers; flat tasks
/// live at level 2 (`## N. Title`) at the file's top scope.
fn render_task_block(number: u32, args: &AppendTaskArgs, heading_level: u8) -> String {
    let mut out = String::new();
    let hashes = "#".repeat(heading_level as usize);
    let _ = writeln!(out, "{hashes} {number}. {}\n", args.title);
    if let Some(items) = &args.body {
        for item in items {
            let _ = writeln!(out, "- [ ] {}", item.trim());
        }
    } else if let Some(slug) = &args.slug {
        // Default single sub-item. The "scenarios/{slug}.md" pointer mirrors
        // the convention `/gov:amend`'s scenario branch uses; `slug` comes from
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

/// Insert a `### N.` task into the phased `tasks.md` body. Behavior:
///
/// - When `parent_heading` is supplied, locate the named `## …` phase and
///   insert the task at the end of its content range. Refuses with
///   `ParentHeadingNotFound` when the heading does not match.
/// - When `parent_heading` is omitted, locate (or create) the default
///   follow-on phase: `## Phase {next-letter} — Follow-on scenarios`,
///   where `{next-letter}` is the next alphabetical letter after existing
///   `Phase X` labels (defaulting to `A` when none exist).
fn insert_phased_task(existing: &str, number: u32, args: &AppendTaskArgs) -> Result<String> {
    let phases = iter_phase_ranges(existing);
    let block = render_task_block(number, args, /* heading_level= */ 3);

    if let Some(target) = &args.parent_heading {
        let phase = phases
            .iter()
            .find(|p| p.heading == *target)
            .ok_or_else(|| PrimitiveError::ParentHeadingNotFound {
                heading: target.clone(),
                available: phases
                    .iter()
                    .map(|p| p.heading.as_str())
                    .collect::<Vec<_>>()
                    .join(", "),
            })?;
        return Ok(insert_inside_phase(existing, phase, &block));
    }

    // No parent_heading supplied — find or create the default follow-on phase.
    // First check for any existing `Phase X — Follow-on scenarios` phase and
    // extend it; only create a new phase when no follow-on phase exists yet.
    // This keeps the Phase-letter sequence bounded over multiple follow-on
    // appends (otherwise every `/gov:amend` scenario branch would bump the
    // letter, exhausting A–Z over time).
    if let Some(phase) = phases.iter().find(|p| is_follow_on_phase(&p.heading)) {
        return Ok(insert_inside_phase(existing, phase, &block));
    }
    let default_heading = default_follow_on_phase_heading(&phases);
    if let Some(phase) = phases.iter().find(|p| p.heading == default_heading) {
        Ok(insert_inside_phase(existing, phase, &block))
    } else {
        // Phase does not yet exist; append phase header + task at file
        // bottom. Use the same trailing-newline normalization as the flat
        // stitch path so re-runs are idempotent.
        let trimmed = existing.trim_end_matches(['\n', '\r']);
        let mut out =
            String::with_capacity(trimmed.len() + default_heading.len() + block.len() + 8);
        out.push_str(trimmed);
        out.push_str("\n\n## ");
        out.push_str(&default_heading);
        out.push_str("\n\n");
        out.push_str(&block);
        if !out.ends_with('\n') {
            out.push('\n');
        }
        Ok(out)
    }
}

/// Insert `block` at the end of the named phase's content range. Preserves
/// content before and after the phase exactly; adds blank-line separation
/// before the inserted block and trims any trailing blank lines inside the
/// phase so re-runs are idempotent.
fn insert_inside_phase(existing: &str, phase: &PhaseRange, block: &str) -> String {
    let lines: Vec<&str> = existing.lines().collect();
    // `end_line` is 1-based, inclusive; convert to a slice end-exclusive.
    let mut before_end = phase.end_line.min(lines.len());
    // Trim trailing blank lines inside the phase range so we don't grow
    // unbounded vertical whitespace on repeated appends.
    while before_end > phase.start_line && lines[before_end - 1].trim().is_empty() {
        before_end -= 1;
    }
    let mut out = String::with_capacity(existing.len() + block.len() + 4);
    for (idx, line) in lines.iter().enumerate().take(before_end) {
        out.push_str(line);
        if idx + 1 < lines.len() || existing.ends_with('\n') {
            out.push('\n');
        }
    }
    // Ensure a blank line of separation before the inserted block.
    if !out.ends_with("\n\n") {
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push('\n');
    }
    out.push_str(block);
    if !out.ends_with('\n') {
        out.push('\n');
    }
    // Re-attach the tail (lines after the phase's range).
    if before_end < lines.len() {
        out.push('\n');
        for (i, line) in lines.iter().enumerate().skip(before_end) {
            out.push_str(line);
            if i + 1 < lines.len() || existing.ends_with('\n') {
                out.push('\n');
            }
        }
    }
    out
}

/// `true` when `heading` matches the shape `Phase X — Follow-on scenarios`
/// (any single uppercase letter for X). Used to detect an existing follow-on
/// phase to extend rather than spawning a new Phase letter on every append.
fn is_follow_on_phase(heading: &str) -> bool {
    let rest = match heading.strip_prefix("Phase ") {
        Some(r) => r.trim_start(),
        None => return false,
    };
    let mut chars = rest.chars();
    let Some(letter) = chars.next() else {
        return false;
    };
    if !letter.is_ascii_uppercase() {
        return false;
    }
    let suffix: String = chars.collect();
    // Accept `Phase X — Follow-on scenarios` exactly, ignoring whether the
    // dash is an em-dash, en-dash, or ASCII hyphen (the spec uses em-dash;
    // adopters who type a hyphen by hand should still hit this branch).
    let normalized = suffix
        .trim()
        .trim_start_matches(['—', '–', '-'])
        .trim_start();
    normalized.eq_ignore_ascii_case("Follow-on scenarios")
}

/// Compute the default follow-on phase heading: `Phase {next-letter} —
/// Follow-on scenarios`. `{next-letter}` is the next ASCII uppercase letter
/// after the highest existing `Phase X` label (where X is a single uppercase
/// letter). Defaults to `A` when no such labels exist.
fn default_follow_on_phase_heading(phases: &[PhaseRange]) -> String {
    let mut max_letter: Option<char> = None;
    for phase in phases {
        // Match headings shaped like "Phase X" (any text after the X is
        // ignored — `Phase A — Refactor`, `Phase A`, `Phase A:`, etc. all
        // contribute the letter A to the max).
        let rest = phase.heading.strip_prefix("Phase ").map(str::trim_start);
        let Some(rest) = rest else { continue };
        let Some(letter) = rest.chars().next() else {
            continue;
        };
        if !letter.is_ascii_uppercase() {
            continue;
        }
        // The character after the letter (if any) should not itself be a
        // letter — otherwise "Phaser" would parse as letter 'r'. Allow
        // word-boundary chars: end-of-string, space, hyphen, em/en-dash,
        // colon, period.
        let next_char = rest.chars().nth(1);
        let is_boundary = match next_char {
            None => true,
            Some(c) => !c.is_alphanumeric(),
        };
        if !is_boundary {
            continue;
        }
        max_letter = match max_letter {
            None => Some(letter),
            Some(prev) if letter > prev => Some(letter),
            other => other,
        };
    }
    let next_letter = match max_letter {
        None => 'A',
        Some(c) if c < 'Z' => char::from(c as u8 + 1),
        Some(_) => 'Z', // saturate at Z; the unlikely "Phase Z" already exists case
    };
    format!("Phase {next_letter} — Follow-on scenarios")
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
    if let Ok(spec) = read_text(&feature_dir.join("spec.md"))
        && let Ok((_fm, body)) = split_frontmatter(&spec, &feature_dir.join("spec.md"))
    {
        for line in body.lines() {
            if let Some((level, text)) = parse_atx_heading(line)
                && level == 1
            {
                return format!("# {text} Tasks");
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
            // Phased-structure routing is tested separately; flat tests
            // leave this unset.
            parent_heading: None,
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
    fn rejects_structure_injection_via_title_newline() {
        // A title smuggling `\n## 99. Phantom task` would append a second
        // task heading through one call. Must refuse before any write.
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let tasks_path = tmp.path().join("specs/042-foo/tasks.md");
        fs::write(&tasks_path, "# Tasks\n\n## 1. First\n").unwrap();
        let original = fs::read_to_string(&tasks_path).unwrap();

        let a = args(
            "specs/042-foo",
            "Innocent\n\n## 99. Phantom task\n\n- [ ] injected",
            "done.",
        );
        let err = run(&a, tmp.path()).unwrap_err();
        assert!(
            matches!(&err, PrimitiveError::InvalidArgument { primitive, argument, .. }
                if primitive == "append-task" && argument == "title"),
            "expected InvalidArgument for title, got {err:?}"
        );
        assert_eq!(
            fs::read_to_string(&tasks_path).unwrap(),
            original,
            "tasks.md must be untouched"
        );
    }

    #[test]
    fn rejects_slug_injection_before_write() {
        // BE-INPUT-001: a slug smuggling `\n## 99. …` would append a
        // phantom task heading through the default-body line. The slug
        // allowlist must refuse it before any write.
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let tasks_path = tmp.path().join("specs/042-foo/tasks.md");
        fs::write(&tasks_path, "# Tasks\n\n## 1. First\n").unwrap();
        let original = fs::read_to_string(&tasks_path).unwrap();

        let mut a = args("specs/042-foo", "Innocent", "done.");
        a.body = None;
        a.slug = Some("x\n## 99. Phantom\n- [ ] injected".into());
        let err = run(&a, tmp.path()).unwrap_err();
        assert!(
            matches!(&err, PrimitiveError::InvalidSlug { .. }),
            "expected InvalidSlug, got {err:?}"
        );
        assert_eq!(
            fs::read_to_string(&tasks_path).unwrap(),
            original,
            "tasks.md must be untouched"
        );
    }

    #[test]
    fn rejects_newlines_in_done_when_and_body_items() {
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");

        let mut a = args("specs/042-foo", "Fine title", "done.\n- [ ] extra");
        let err = run(&a, tmp.path()).unwrap_err();
        assert!(
            matches!(&err, PrimitiveError::InvalidArgument { argument, .. }
                if argument == "done-when"),
            "expected InvalidArgument for done-when, got {err:?}"
        );

        a = args("specs/042-foo", "Fine title", "done.");
        a.body = Some(vec!["ok item".into(), "bad\r\nitem".into()]);
        a.slug = None;
        let err = run(&a, tmp.path()).unwrap_err();
        assert!(
            matches!(&err, PrimitiveError::InvalidArgument { argument, .. }
                if argument == "body[1]"),
            "expected InvalidArgument for body[1], got {err:?}"
        );
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

    // --- phased-structure tests -----------------------------------------------

    fn phased_tasks_md() -> &'static str {
        "# 042 — Foo Tasks\n\n\
         ## Phase A — Bootstrap\n\n\
         ### 1. Wire up the crate\n\n\
         - [x] do thing\n\n\
         - **Done when**: it is done.\n\n\
         ## Phase B — Implementation\n\n\
         ### 2. Build the thing\n\n\
         - [x] do other thing\n\n\
         - **Done when**: it is done.\n"
    }

    #[test]
    fn phased_append_under_explicit_parent_heading() {
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let tasks_path = tmp.path().join("specs/042-foo/tasks.md");
        fs::write(&tasks_path, phased_tasks_md()).unwrap();
        let mut a = args("specs/042-foo", "Ship the thing", "shipped.");
        a.slug = Some("ship-it".into());
        a.parent_heading = Some("Phase B — Implementation".into());
        let result = run(&a, tmp.path()).unwrap();
        assert_eq!(result.task_number, 3);
        let body = fs::read_to_string(&tasks_path).unwrap();
        // New task uses ### N. (phased convention), not ## N. Match line
        // starts to avoid the `### 3. Ship` heading being a substring hit
        // for `## 3. Ship`.
        assert!(
            body.contains("### 3. Ship the thing"),
            "expected ### 3. heading under phased structure, got:\n{body}"
        );
        assert!(
            !body.lines().any(|l| l.starts_with("## 3. Ship the thing")),
            "found a flat-style ## 3. task in a phased file"
        );
        // The new task lands under Phase B, not Phase A or a new phase.
        let phase_b_idx = body.find("## Phase B").unwrap();
        let new_task_idx = body.find("### 3. Ship the thing").unwrap();
        assert!(
            new_task_idx > phase_b_idx,
            "new task should appear after the Phase B heading"
        );
        // No new Phase C — Follow-on scenarios phase was created.
        assert!(!body.contains("Follow-on scenarios"));
        // Existing tasks preserved verbatim.
        assert!(body.contains("### 1. Wire up the crate"));
        assert!(body.contains("### 2. Build the thing"));
    }

    #[test]
    fn phased_append_creates_default_follow_on_phase_when_no_parent_heading() {
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let tasks_path = tmp.path().join("specs/042-foo/tasks.md");
        fs::write(&tasks_path, phased_tasks_md()).unwrap();
        let mut a = args("specs/042-foo", "Follow-on", "follow-on done.");
        a.slug = Some("follow-on".into());
        a.parent_heading = None;
        let result = run(&a, tmp.path()).unwrap();
        assert_eq!(result.task_number, 3);
        let body = fs::read_to_string(&tasks_path).unwrap();
        // Default phase letter is C (next after A and B).
        assert!(
            body.contains("## Phase C — Follow-on scenarios"),
            "expected default Phase C follow-on header, got:\n{body}"
        );
        // New task is ### 3. under the new phase.
        let phase_c_idx = body.find("## Phase C — Follow-on scenarios").unwrap();
        let new_task_idx = body.find("### 3. Follow-on").unwrap();
        assert!(new_task_idx > phase_c_idx);
    }

    #[test]
    fn phased_append_extends_existing_follow_on_phase() {
        // Second invocation should land under the same Phase C — Follow-on
        // scenarios rather than creating Phase D.
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let tasks_path = tmp.path().join("specs/042-foo/tasks.md");
        let seed = format!(
            "{}\n## Phase C — Follow-on scenarios\n\n\
             ### 3. First follow-on\n\n\
             - [x] done.\n\n\
             - **Done when**: done.\n",
            phased_tasks_md()
        );
        fs::write(&tasks_path, &seed).unwrap();
        let mut a = args("specs/042-foo", "Second follow-on", "done.");
        a.slug = Some("second-follow-on".into());
        a.parent_heading = None;
        let result = run(&a, tmp.path()).unwrap();
        assert_eq!(result.task_number, 4);
        let body = fs::read_to_string(&tasks_path).unwrap();
        // Single Phase C header, two tasks under it.
        assert_eq!(
            body.matches("## Phase C — Follow-on scenarios").count(),
            1,
            "should not create a duplicate Phase C header"
        );
        assert!(body.contains("### 3. First follow-on"));
        assert!(body.contains("### 4. Second follow-on"));
        // No Phase D was created.
        assert!(!body.contains("## Phase D"));
    }

    #[test]
    fn phased_append_creates_phase_a_when_no_phase_letters_exist() {
        // A phased file whose `## …` headings are non-letter (e.g., a stage
        // label like `## Stage 1 — Foo`) should not collide with the
        // default Phase A — Follow-on scenarios on first follow-on.
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let tasks_path = tmp.path().join("specs/042-foo/tasks.md");
        let seed = "# 042 — Foo Tasks\n\n\
                    ## Stage 1 — Bootstrap\n\n\
                    ### 1. Wire up\n\n\
                    - [x] done.\n\n\
                    - **Done when**: done.\n";
        fs::write(&tasks_path, seed).unwrap();
        let mut a = args("specs/042-foo", "First follow-on", "done.");
        a.slug = Some("first".into());
        a.parent_heading = None;
        run(&a, tmp.path()).unwrap();
        let body = fs::read_to_string(&tasks_path).unwrap();
        assert!(
            body.contains("## Phase A — Follow-on scenarios"),
            "expected default Phase A when no Phase letters exist, got:\n{body}"
        );
    }

    #[test]
    fn phased_append_refuses_unknown_parent_heading() {
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let tasks_path = tmp.path().join("specs/042-foo/tasks.md");
        fs::write(&tasks_path, phased_tasks_md()).unwrap();
        let mut a = args("specs/042-foo", "Misrouted", "done.");
        a.slug = Some("misrouted".into());
        a.parent_heading = Some("Phase Z — does not exist".into());
        let err = run(&a, tmp.path()).unwrap_err();
        let PrimitiveError::ParentHeadingNotFound {
            heading, available, ..
        } = err
        else {
            panic!("expected ParentHeadingNotFound, got: {err:?}");
        };
        assert_eq!(heading, "Phase Z — does not exist");
        assert!(available.contains("Phase A — Bootstrap"));
        assert!(available.contains("Phase B — Implementation"));
        // File unchanged (the primitive refused before write).
        assert_eq!(fs::read_to_string(&tasks_path).unwrap(), phased_tasks_md());
    }

    #[test]
    fn mixed_structure_treated_as_phased() {
        // A file with both ## N. (flat) and ### N. (phased) is phased per
        // the scenario's edge case. The next task should land under the
        // appropriate phase, not at file bottom as ## N.
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let tasks_path = tmp.path().join("specs/042-foo/tasks.md");
        let seed = "# 042 — Foo Tasks\n\n\
                    ## 1. Legacy flat task\n\n\
                    - [x] done.\n\n\
                    - **Done when**: done.\n\n\
                    ## Phase A — New work\n\n\
                    ### 2. First phased task\n\n\
                    - [x] done.\n\n\
                    - **Done when**: done.\n";
        fs::write(&tasks_path, seed).unwrap();
        let mut a = args("specs/042-foo", "Third task", "done.");
        a.slug = Some("third".into());
        a.parent_heading = Some("Phase A — New work".into());
        let result = run(&a, tmp.path()).unwrap();
        // Max across ## 1. and ### 2. is 2, so next is 3.
        assert_eq!(result.task_number, 3);
        let body = fs::read_to_string(&tasks_path).unwrap();
        assert!(body.contains("### 3. Third task"));
        assert!(body.contains("## 1. Legacy flat task"));
    }

    #[test]
    fn flat_file_ignores_parent_heading_arg() {
        // In a flat file, parent_heading is informational only — the task
        // still lands at file bottom as ## N.
        let tmp = tempdir().unwrap();
        make_feature_with_spec(tmp.path(), "specs/042-foo", "042 — Foo");
        let tasks_path = tmp.path().join("specs/042-foo/tasks.md");
        fs::write(&tasks_path, "# 042 — Foo Tasks\n\n## 1. First\n").unwrap();
        let mut a = args("specs/042-foo", "Second", "done.");
        a.parent_heading = Some("Phase A — does not matter".into());
        let result = run(&a, tmp.path()).unwrap();
        assert_eq!(result.task_number, 2);
        let body = fs::read_to_string(&tasks_path).unwrap();
        assert!(body.contains("## 2. Second"));
        assert!(!body.contains("### 2. Second"));
    }
}
