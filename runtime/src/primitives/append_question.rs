//! `append-question` — append one question bullet to `## Open Questions`.
//!
//! The deterministic surface behind `/gov:amend`'s question-route write
//! (spec 022, scenario coverage-expansion-primitives), which previously
//! had no primitive — asymmetric with the scenario route's
//! `create-scenario` + `append-task`. Appends `- {question}` to the
//! target artifact's `## Open Questions` section: the feature's `spec.md`
//! by default, or `scenarios/{slug}.md` when `scenario` names a slug.
//!
//! Three contract points carried over from the command prose:
//!
//! - **Dedup** uses the normalized-whitespace comparison amend already
//!   specifies (collapse whitespace runs, trim, case-insensitive) against
//!   the entries `read-spec`'s question parser reports, so the runtime
//!   and markdown-only paths agree on question identity. A match is the
//!   `appended: false` domain outcome (nothing written) reporting the
//!   existing entry.
//! - **Same-write back-edge**: on a spec target whose status is
//!   `clarified`, `planned`, `in-progress`, or `done`, the frontmatter
//!   status reverts to `draft` in the same atomic write as the append —
//!   never a window where the body holds an unresolved question but the
//!   status still claims otherwise. Scenario targets have no status field
//!   and never back-edge.
//! - **Section creation**: a missing `## Open Questions` section is
//!   created per the template order — immediately before
//!   `## Resolved Questions` when that section exists (the scenario
//!   scaffold), else at the end of the file (the spec template puts it
//!   last). A `*None …*` placeholder line left by the scaffolds is
//!   replaced by the first real entry.

use std::path::Path;

use crate::primitives::set_status::locate_status_field;
use crate::primitives::{
    PrimitiveError, Result, read_spec, read_text, rel_path, split_frontmatter_with_offset,
    validate_no_traversal, validate_single_line, validate_slug, write_atomic,
};
use crate::schema::paths;
use crate::schema::primitives::{AppendQuestionArgs, AppendQuestionResult};

/// The heading line of the questions section, exact ATX form.
const SECTION_HEADING: &str = "## Open Questions";

/// The heading whose presence anchors section creation for scenario-shaped
/// artifacts (`## Open Questions` inserts immediately before it).
const RESOLVED_HEADING: &str = "## Resolved Questions";

/// Statuses the same-write back-edge reverts to `draft`. `draft` itself
/// is a no-op; a value outside the lifecycle set is left alone
/// (`validate-frontmatter` owns flagging corrupt statuses).
const BACK_EDGE_STATUSES: [&str; 4] = ["clarified", "planned", "in-progress", "done"];

/// Execute the `append-question` primitive against the given repo root.
///
/// # Errors
///
/// Returns [`PrimitiveError::InvalidPath`] for a malformed `feature`,
/// [`PrimitiveError::InvalidSlug`] for a malformed `scenario`,
/// [`PrimitiveError::InvalidArgument`] when `question` is empty or
/// multi-line, [`PrimitiveError::FeatureNotFound`] when the feature
/// directory is missing, [`PrimitiveError::Io`] when the target artifact
/// is unreadable, or [`PrimitiveError::MissingFrontmatter`] when it lacks
/// `---` fences. A duplicate question is the `appended: false` domain
/// outcome, not an error.
pub fn run(args: &AppendQuestionArgs, repo: &Path) -> Result<AppendQuestionResult> {
    validate_no_traversal(&args.feature)?;
    validate_single_line("append-question", "question", &args.question)?;
    let question = args.question.trim();
    if question.is_empty() {
        return Err(PrimitiveError::InvalidArgument {
            primitive: "append-question".into(),
            argument: "question".into(),
            reason: "question is empty".into(),
        });
    }

    let root = paths::Paths::load(repo).specs_root;
    let feature_dir = repo.join(&root).join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            root,
            feature: args.feature.clone(),
        });
    }
    let (target_path, is_spec) = match &args.scenario {
        Some(slug) => {
            validate_slug(slug)?;
            (
                feature_dir.join("scenarios").join(format!("{slug}.md")),
                false,
            )
        }
        None => (feature_dir.join("spec.md"), true),
    };

    let content = read_text(&target_path)?;
    let (fm_text, body, fm_start) = split_frontmatter_with_offset(&content, &target_path)?;

    // Dedup against exactly the entries read-spec's parser reports, with
    // amend's normalized-whitespace comparison.
    let target = normalize(question);
    if let Some(existing) = read_spec::parse_open_questions(body, "Open Questions")
        .into_iter()
        .find(|q| normalize(&q.text) == target)
    {
        return Ok(AppendQuestionResult {
            path: rel_path(&target_path, repo),
            appended: false,
            duplicate_of: Some(existing.text),
            section_created: false,
            status_reverted: false,
            previous_status: None,
        });
    }

    // Same-write back-edge first (a frontmatter splice never moves the
    // body insertion point computed afterwards on the spliced content).
    let mut new_content = content.clone();
    let mut status_reverted = false;
    let mut previous_status = None;
    if is_spec
        && let Ok((current, range)) = locate_status_field(fm_text, "", "")
        && BACK_EDGE_STATUSES.contains(&current.as_str())
    {
        let absolute = (fm_start + range.start)..(fm_start + range.end);
        new_content.replace_range(absolute, "draft");
        status_reverted = true;
        previous_status = Some(current);
    }

    let (new_content, section_created) = insert_question(&new_content, question);
    write_atomic(&target_path, &new_content)?;

    Ok(AppendQuestionResult {
        path: rel_path(&target_path, repo),
        appended: true,
        duplicate_of: None,
        section_created,
        status_reverted,
        previous_status,
    })
}

/// Amend's normalized-whitespace comparison key: whitespace runs collapse
/// to a single space, leading/trailing whitespace trims away, and the
/// comparison is case-insensitive.
fn normalize(text: &str) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

/// Insert `- {question}` into the `## Open Questions` section of
/// `content`, creating the section when absent. Returns the rewritten
/// content and whether the section was created.
fn insert_question(content: &str, question: &str) -> (String, bool) {
    let lines: Vec<&str> = content.lines().collect();
    let bullet = format!("- {question}");

    if let Some(heading_idx) = find_heading(&lines, SECTION_HEADING) {
        // Section boundary: the next ATX heading of level 1 or 2, or EOF.
        let end = lines[heading_idx + 1..]
            .iter()
            .position(|l| is_section_boundary(l))
            .map_or(lines.len(), |offset| heading_idx + 1 + offset);

        // Keep the section's existing content minus scaffold placeholders
        // and surrounding blanks; the bullet appends after the last kept
        // line — directly when that line is already a bullet (one list),
        // after a blank otherwise (markdownlint MD032: lists surrounded
        // by blank lines).
        let mut kept: Vec<String> = lines[heading_idx + 1..end]
            .iter()
            .filter(|l| !is_placeholder(l))
            .map(|l| (*l).to_string())
            .collect();
        while kept.first().is_some_and(|l| l.trim().is_empty()) {
            kept.remove(0);
        }
        while kept.last().is_some_and(|l| l.trim().is_empty()) {
            kept.pop();
        }
        if kept
            .last()
            .is_some_and(|l| !l.trim_start().starts_with("- "))
        {
            kept.push(String::new());
        }
        kept.push(bullet);

        let mut out: Vec<String> = lines[..=heading_idx]
            .iter()
            .map(|l| (*l).to_string())
            .collect();
        out.push(String::new());
        out.extend(kept);
        if end < lines.len() {
            out.push(String::new());
            out.extend(lines[end..].iter().map(|l| (*l).to_string()));
        }
        return (format!("{}\n", out.join("\n").trim_end()), false);
    }

    // Section absent: create it before `## Resolved Questions` when that
    // section exists (scenario scaffold order), else at end of file (the
    // spec template puts Open Questions last).
    let block = [SECTION_HEADING.to_string(), String::new(), bullet];
    let mut out: Vec<String>;
    if let Some(resolved_idx) = find_heading(&lines, RESOLVED_HEADING) {
        out = lines[..resolved_idx]
            .iter()
            .map(|l| (*l).to_string())
            .collect();
        while out.last().is_some_and(|l| l.trim().is_empty()) {
            out.pop();
        }
        out.push(String::new());
        out.extend(block);
        out.push(String::new());
        out.extend(lines[resolved_idx..].iter().map(|l| (*l).to_string()));
    } else {
        out = lines.iter().map(|l| (*l).to_string()).collect();
        while out.last().is_some_and(|l| l.trim().is_empty()) {
            out.pop();
        }
        out.push(String::new());
        out.extend(block);
    }
    (format!("{}\n", out.join("\n").trim_end()), true)
}

/// Index of the line whose trimmed text equals the given ATX heading.
fn find_heading(lines: &[&str], heading: &str) -> Option<usize> {
    lines.iter().position(|l| l.trim_end() == heading)
}

/// Whether a line closes the Open Questions section: an ATX heading of
/// level 1 or 2 (deeper headings belong to the section, mirroring
/// `read-spec`'s section parser).
fn is_section_boundary(line: &str) -> bool {
    let t = line.trim_start();
    (t.starts_with("# ") || t.starts_with("## ")) && !t.starts_with("###")
}

/// Whether a line is a scaffold placeholder the first real entry
/// replaces: the emphasized `*None …*` forms `create-scenario` and the
/// fixtures leave behind (`*None — all resolved.*`, `*None yet.*`,
/// `*None — captured during scenario authoring.*`).
fn is_placeholder(line: &str) -> bool {
    let t = line.trim();
    t.starts_with("*None") && t.ends_with('*')
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn seed_spec(repo: &Path, spec: &str) {
        fs::create_dir_all(repo.join("specs/009-quest")).unwrap();
        fs::write(repo.join("specs/009-quest/spec.md"), spec).unwrap();
    }

    fn read_spec_file(repo: &Path) -> String {
        fs::read_to_string(repo.join("specs/009-quest/spec.md")).unwrap()
    }

    fn args(question: &str) -> AppendQuestionArgs {
        AppendQuestionArgs {
            feature: "009-quest".into(),
            question: question.into(),
            scenario: None,
        }
    }

    #[test]
    fn appends_bullet_to_existing_section() {
        let tmp = tempdir().unwrap();
        seed_spec(
            tmp.path(),
            "---\nstatus: draft\ndependencies: []\n---\n\n# 009\n\n## Open Questions\n\n- Existing question?\n",
        );
        let result = run(&args("What about retries?"), tmp.path()).unwrap();
        assert!(result.appended);
        assert!(!result.section_created);
        assert!(!result.status_reverted, "draft spec never back-edges");
        assert_eq!(result.path, "specs/009-quest/spec.md");
        assert_eq!(
            read_spec_file(tmp.path()),
            "---\nstatus: draft\ndependencies: []\n---\n\n# 009\n\n## Open Questions\n\n- Existing question?\n- What about retries?\n"
        );
    }

    #[test]
    fn back_edges_non_draft_spec_in_the_same_write() {
        let tmp = tempdir().unwrap();
        for prior in ["clarified", "planned", "in-progress", "done"] {
            seed_spec(
                tmp.path(),
                &format!(
                    "---\nstatus: {prior}\ndependencies: []\n---\n\n# 009\n\n## Open Questions\n"
                ),
            );
            let result = run(&args("New wrinkle?"), tmp.path()).unwrap();
            assert!(result.appended, "{prior}");
            assert!(result.status_reverted, "{prior}");
            assert_eq!(result.previous_status.as_deref(), Some(prior));
            let content = read_spec_file(tmp.path());
            assert!(content.contains("status: draft"), "{content}");
            assert!(content.contains("- New wrinkle?"), "{content}");
        }
    }

    #[test]
    fn dedup_is_whitespace_and_case_insensitive() {
        let tmp = tempdir().unwrap();
        let before = "---\nstatus: planned\ndependencies: []\n---\n\n# 009\n\n## Open Questions\n\n- Should   rate limits be\n  configurable per tenant?\n";
        seed_spec(tmp.path(), before);
        let result = run(
            &args("should rate limits be configurable per tenant?"),
            tmp.path(),
        )
        .unwrap();
        assert!(!result.appended);
        assert_eq!(
            result.duplicate_of.as_deref(),
            Some("Should   rate limits be configurable per tenant?"),
            "the continuation-folded existing entry is reported verbatim \
             (only the comparison normalizes whitespace)"
        );
        assert!(!result.status_reverted, "a dedup never mutates status");
        assert_eq!(read_spec_file(tmp.path()), before, "no write on dedup");
    }

    #[test]
    fn replaces_scaffold_placeholder_with_first_entry() {
        let tmp = tempdir().unwrap();
        seed_spec(
            tmp.path(),
            "---\nstatus: draft\ndependencies: []\n---\n\n# 009\n\n## Open Questions\n\n*None — all resolved.*\n\n## Resolved Questions\n\n*None yet.*\n",
        );
        let result = run(&args("First real question?"), tmp.path()).unwrap();
        assert!(result.appended);
        assert_eq!(
            read_spec_file(tmp.path()),
            "---\nstatus: draft\ndependencies: []\n---\n\n# 009\n\n## Open Questions\n\n- First real question?\n\n## Resolved Questions\n\n*None yet.*\n"
        );
    }

    #[test]
    fn keeps_template_comment_and_separates_list_with_blank() {
        let tmp = tempdir().unwrap();
        seed_spec(
            tmp.path(),
            "---\nstatus: draft\ndependencies: []\n---\n\n# 009\n\n## Open Questions\n\n<!-- guidance comment -->\n",
        );
        run(&args("Q one?"), tmp.path()).unwrap();
        assert_eq!(
            read_spec_file(tmp.path()),
            "---\nstatus: draft\ndependencies: []\n---\n\n# 009\n\n## Open Questions\n\n<!-- guidance comment -->\n\n- Q one?\n",
            "comment kept; blank line separates the list (MD032)"
        );
    }

    #[test]
    fn creates_missing_section_at_end_of_spec() {
        let tmp = tempdir().unwrap();
        seed_spec(
            tmp.path(),
            "---\nstatus: clarified\ndependencies: []\n---\n\n# 009\n\n## Acceptance Criteria\n\n- [ ] Works.\n",
        );
        let result = run(&args("Where does it go?"), tmp.path()).unwrap();
        assert!(result.section_created);
        assert!(result.status_reverted);
        assert_eq!(
            read_spec_file(tmp.path()),
            "---\nstatus: draft\ndependencies: []\n---\n\n# 009\n\n## Acceptance Criteria\n\n- [ ] Works.\n\n## Open Questions\n\n- Where does it go?\n"
        );
    }

    #[test]
    fn creates_missing_section_before_resolved_questions() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("specs/009-quest/scenarios")).unwrap();
        fs::write(
            tmp.path().join("specs/009-quest/scenarios/edge.md"),
            "---\nsection: \"Core\"\n---\n\n# Edge\n\n## Behavior\n\nDoes things.\n\n## Resolved Questions\n\n- Old one? Answered.\n",
        )
        .unwrap();
        seed_spec(tmp.path(), "---\nstatus: done\n---\n\n# 009\n");
        let mut a = args("Scenario question?");
        a.scenario = Some("edge".into());
        let result = run(&a, tmp.path()).unwrap();
        assert!(result.section_created);
        assert!(!result.status_reverted, "scenario targets never back-edge");
        assert_eq!(result.path, "specs/009-quest/scenarios/edge.md");
        let content =
            fs::read_to_string(tmp.path().join("specs/009-quest/scenarios/edge.md")).unwrap();
        assert_eq!(
            content,
            "---\nsection: \"Core\"\n---\n\n# Edge\n\n## Behavior\n\nDoes things.\n\n## Open Questions\n\n- Scenario question?\n\n## Resolved Questions\n\n- Old one? Answered.\n"
        );
        // The spec's status is untouched on a scenario target.
        assert!(read_spec_file(tmp.path()).contains("status: done"));
    }

    #[test]
    fn scenario_target_appends_without_touching_spec_status() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("specs/009-quest/scenarios")).unwrap();
        fs::write(
            tmp.path().join("specs/009-quest/scenarios/edge.md"),
            "---\nsection: \"Core\"\n---\n\n# Edge\n\n## Open Questions\n\n*None — captured during scenario authoring.*\n\n## Resolved Questions\n\n*None yet.*\n",
        )
        .unwrap();
        seed_spec(tmp.path(), "---\nstatus: in-progress\n---\n\n# 009\n");
        let mut a = args("How does the edge behave?");
        a.scenario = Some("edge".into());
        let result = run(&a, tmp.path()).unwrap();
        assert!(result.appended);
        assert!(!result.section_created);
        let content =
            fs::read_to_string(tmp.path().join("specs/009-quest/scenarios/edge.md")).unwrap();
        assert!(
            content.contains("## Open Questions\n\n- How does the edge behave?\n"),
            "{content}"
        );
        assert!(read_spec_file(tmp.path()).contains("status: in-progress"));
    }

    #[test]
    fn spec_without_status_field_appends_without_back_edge() {
        let tmp = tempdir().unwrap();
        seed_spec(
            tmp.path(),
            "---\ndependencies: []\n---\n\n# 009\n\n## Open Questions\n",
        );
        let result = run(&args("Still works?"), tmp.path()).unwrap();
        assert!(result.appended);
        assert!(!result.status_reverted);
        assert!(result.previous_status.is_none());
    }

    #[test]
    fn rejects_empty_and_multiline_question() {
        let tmp = tempdir().unwrap();
        seed_spec(tmp.path(), "---\nstatus: draft\n---\n\n# 009\n");
        for bad in ["", "   ", "a\nb", "c\rd"] {
            let err = run(&args(bad), tmp.path()).unwrap_err();
            assert!(
                matches!(err, PrimitiveError::InvalidArgument { .. }),
                "expected InvalidArgument for {bad:?}"
            );
        }
    }

    #[test]
    fn rejects_malformed_scenario_slug() {
        let tmp = tempdir().unwrap();
        seed_spec(tmp.path(), "---\nstatus: draft\n---\n\n# 009\n");
        for bad in ["../escape", "UPPER", "a b"] {
            let mut a = args("A question?");
            a.scenario = Some(bad.into());
            let err = run(&a, tmp.path()).unwrap_err();
            assert!(
                matches!(err, PrimitiveError::InvalidSlug { .. }),
                "expected InvalidSlug for {bad:?}"
            );
        }
    }

    #[test]
    fn missing_feature_and_missing_artifact_error() {
        let tmp = tempdir().unwrap();
        seed_spec(tmp.path(), "---\nstatus: draft\n---\n\n# 009\n");
        let err = run(
            &AppendQuestionArgs {
                feature: "099-absent".into(),
                question: "Anyone home?".into(),
                scenario: None,
            },
            tmp.path(),
        )
        .unwrap_err();
        assert!(matches!(err, PrimitiveError::FeatureNotFound { .. }));

        let mut a = args("Anyone home?");
        a.scenario = Some("no-such-scenario".into());
        let err = run(&a, tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::Io { .. }));
    }

    #[test]
    fn honors_configured_specs_root() {
        let tmp = tempdir().unwrap();
        fs::write(
            tmp.path().join(".govern.toml"),
            "[paths]\nspecs-root = \"governance\"\n",
        )
        .unwrap();
        fs::create_dir_all(tmp.path().join("governance/009-quest")).unwrap();
        fs::write(
            tmp.path().join("governance/009-quest/spec.md"),
            "---\nstatus: planned\n---\n\n# 009\n\n## Open Questions\n",
        )
        .unwrap();
        let result = run(&args("Rooted right?"), tmp.path()).unwrap();
        assert_eq!(result.path, "governance/009-quest/spec.md");
        assert!(result.status_reverted);
    }
}
