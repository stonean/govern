//! `read-spec` — parse spec frontmatter and body sections.

use std::path::Path;

use crate::primitives::{
    PrimitiveError, Result, checkbox, parse_atx_heading, read_text, rel_path, section_line_indices,
    section_lines, split_frontmatter,
};
use crate::schema::paths;
use crate::schema::primitives::{
    AcceptanceCriterion, Frontmatter, OpenQuestion, ReadSpecArgs, ReadSpecResult, SpecSection,
};

/// Execute the `read-spec` primitive against the given repo root.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeatureNotFound`] when `specs/<feature>/` does
/// not exist, [`PrimitiveError::Io`] on filesystem failures,
/// [`PrimitiveError::MissingFrontmatter`] when the spec lacks `---` fences,
/// or [`PrimitiveError::Yaml`] when the frontmatter is not valid YAML.
pub fn run(args: &ReadSpecArgs, repo: &Path) -> Result<ReadSpecResult> {
    let root = paths::Paths::load(repo).specs_root;
    let feature_dir = repo.join(&root).join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            root,
            feature: args.feature.clone(),
        });
    }
    let spec_path = feature_dir.join("spec.md");
    let content = read_text(&spec_path)?;
    let (fm_text, body) = split_frontmatter(&content, &spec_path)?;
    let frontmatter: Frontmatter =
        serde_norway::from_str(fm_text).map_err(|source| PrimitiveError::Yaml {
            path: spec_path.clone(),
            source,
        })?;

    let sections = parse_sections(body, args.include_body);
    let acceptance_criteria = parse_checkboxes(body, "Acceptance Criteria");
    let open_questions = parse_open_questions(body, "Open Questions");

    Ok(ReadSpecResult {
        frontmatter,
        sections,
        acceptance_criteria,
        open_questions,
        path: rel_path(&spec_path, repo),
    })
}

fn parse_sections(body: &str, include_body: bool) -> Vec<SpecSection> {
    let mut sections: Vec<SpecSection> = Vec::new();
    let mut pending_body: Vec<&str> = Vec::new();
    let mut current: Option<(String, u8)> = None;

    for line in body.lines() {
        if let Some((level, heading)) = parse_atx_heading(line)
            && level >= 2
        {
            if let Some((h, l)) = current.take() {
                sections.push(SpecSection {
                    heading: h,
                    level: l,
                    body: if include_body {
                        pending_body.join("\n").trim().to_string()
                    } else {
                        String::new()
                    },
                });
            }
            pending_body.clear();
            current = Some((heading, level));
            continue;
        }
        if current.is_some() {
            pending_body.push(line);
        }
    }
    if let Some((heading, level)) = current {
        sections.push(SpecSection {
            heading,
            level,
            body: if include_body {
                pending_body.join("\n").trim().to_string()
            } else {
                String::new()
            },
        });
    }
    sections
}

/// Walk the named section's checkboxes with comment/fence awareness
/// ([`section_line_indices`]): example checkboxes inside a template
/// guidance comment or a fenced code block are not criteria. The indexes
/// of the returned criteria form a contract with `mark-criterion`'s
/// addressing — both consume the same shared walker AND the same checkbox
/// grammar ([`checkbox::parse_checkbox_line`]), so index N here is the
/// checkbox index N flips.
///
/// A wrapped acceptance criterion spans multiple source lines: an indented
/// non-checkbox continuation line folds into the preceding criterion's
/// text (mirroring [`parse_open_questions`]) rather than being dropped
/// mid-sentence. The index derivation stays keyed to checkbox lines only —
/// a continuation line never pushes a new entry — so the read/mark index
/// contract is preserved.
fn parse_checkboxes(body: &str, section_heading: &str) -> Vec<AcceptanceCriterion> {
    let lines: Vec<&str> = body.lines().collect();
    let mut out: Vec<AcceptanceCriterion> = Vec::new();
    for idx in section_line_indices(&lines, section_heading) {
        let line = lines[idx];
        if let Some((checked, text)) = checkbox::parse_checkbox_line(line) {
            out.push(AcceptanceCriterion { checked, text });
            continue;
        }
        // Fold an indented, non-checkbox continuation line into the last
        // criterion. Indentation is the wrap signal (markdown continuation
        // lines are indented under their list item); a non-indented,
        // non-checkbox line is not a continuation and is ignored.
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if line.starts_with([' ', '\t'])
            && let Some(current) = out.last_mut()
        {
            current.text.push(' ');
            current.text.push_str(trimmed);
        }
    }
    out
}

fn parse_open_questions(body: &str, section_heading: &str) -> Vec<OpenQuestion> {
    let mut out = Vec::new();
    let mut current: Option<String> = None;
    for line in section_lines(body, section_heading) {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("- ") {
            if let Some(prev) = current.take() {
                push_question(&mut out, &prev);
            }
            current = Some(rest.trim().to_string());
        } else if !trimmed.is_empty() && current.is_some() {
            let continuation = trimmed.to_string();
            if let Some(buf) = current.as_mut() {
                buf.push(' ');
                buf.push_str(&continuation);
            }
        } else if trimmed.is_empty()
            && let Some(prev) = current.take()
        {
            push_question(&mut out, &prev);
        }
    }
    if let Some(prev) = current {
        push_question(&mut out, &prev);
    }
    out
}

fn push_question(out: &mut Vec<OpenQuestion>, text: &str) {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed == "*None — all resolved.*" {
        return;
    }
    out.push(OpenQuestion {
        text: trimmed.to_string(),
    });
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::path::PathBuf;

    fn fixture_repo() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/primitives/sample-repo")
    }

    #[test]
    fn parses_basic_spec() {
        let repo = fixture_repo();
        let result = run(
            &ReadSpecArgs {
                feature: "001-basic".into(),
                include_body: false,
            },
            &repo,
        )
        .unwrap();

        assert_eq!(result.frontmatter.status, "clarified");
        assert!(result.frontmatter.dependencies.is_empty());
        assert_eq!(result.path, "specs/001-basic/spec.md");

        let section_headings: Vec<&str> =
            result.sections.iter().map(|s| s.heading.as_str()).collect();
        assert_eq!(
            section_headings,
            vec![
                "Motivation",
                "Acceptance Criteria",
                "Open Questions",
                "Resolved Questions",
            ]
        );
        for section in &result.sections {
            assert!(
                section.body.is_empty(),
                "body skipped when include_body=false"
            );
        }

        assert_eq!(result.acceptance_criteria.len(), 3);
        assert!(!result.acceptance_criteria[0].checked);
        assert!(result.acceptance_criteria[1].checked);
        assert!(!result.acceptance_criteria[2].checked);

        assert_eq!(result.open_questions.len(), 1);
        assert!(
            result.open_questions[0]
                .text
                .starts_with("Should fixtures embed binary assets")
        );
    }

    #[test]
    fn include_body_populates_section_text() {
        let repo = fixture_repo();
        let result = run(
            &ReadSpecArgs {
                feature: "001-basic".into(),
                include_body: true,
            },
            &repo,
        )
        .unwrap();
        let motivation = result
            .sections
            .iter()
            .find(|s| s.heading == "Motivation")
            .unwrap();
        assert!(motivation.body.contains("deterministic input"));
    }

    #[test]
    fn dependent_spec_lists_dependencies() {
        let repo = fixture_repo();
        let result = run(
            &ReadSpecArgs {
                feature: "002-dependent".into(),
                include_body: false,
            },
            &repo,
        )
        .unwrap();
        assert_eq!(result.frontmatter.status, "planned");
        assert_eq!(result.frontmatter.dependencies, vec!["001-basic"]);
    }

    #[test]
    fn template_state_spec_reports_zero_criteria() {
        // The shipped spec template embeds example `- [ ]` checkboxes inside
        // the Acceptance Criteria guidance comment; a template-state spec
        // must report zero criteria (scenario spec-side-parser-hardening).
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();
        let template =
            std::fs::read_to_string(repo_root.join("framework/templates/spec/spec.md")).unwrap();
        let tmp = tempfile::tempdir().unwrap();
        let feature_dir = tmp.path().join("specs/042-fresh");
        std::fs::create_dir_all(&feature_dir).unwrap();
        std::fs::write(feature_dir.join("spec.md"), template).unwrap();

        let result = run(
            &ReadSpecArgs {
                feature: "042-fresh".into(),
                include_body: false,
            },
            tmp.path(),
        )
        .unwrap();
        assert!(
            result.acceptance_criteria.is_empty(),
            "template guidance-comment checkboxes counted as criteria: {:?}",
            result.acceptance_criteria
        );
    }

    #[test]
    fn criteria_inside_comments_and_fences_are_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        let feature_dir = tmp.path().join("specs/042-fresh");
        std::fs::create_dir_all(&feature_dir).unwrap();
        let spec = "---\nstatus: draft\ndependencies: []\n---\n\n# T\n\n\
                    ## Acceptance Criteria\n\n\
                    <!--\n- [ ] Example inside comment\n-->\n\
                    - [ ] Real criterion.\n\
                    ```text\n- [ ] Example inside fence\n```\n\
                    - [x] Second real criterion.\n";
        std::fs::write(feature_dir.join("spec.md"), spec).unwrap();

        let result = run(
            &ReadSpecArgs {
                feature: "042-fresh".into(),
                include_body: false,
            },
            tmp.path(),
        )
        .unwrap();
        let texts: Vec<&str> = result
            .acceptance_criteria
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        assert_eq!(texts, vec!["Real criterion.", "Second real criterion."]);
        assert!(!result.acceptance_criteria[0].checked);
        assert!(result.acceptance_criteria[1].checked);
    }

    #[test]
    fn folds_wrapped_continuation_into_criterion_text() {
        // A multi-line acceptance criterion must reach verifyCriteria whole,
        // not truncated mid-sentence. The index contract is preserved: the
        // wrapped criterion is still one entry at its checkbox index.
        let tmp = tempfile::tempdir().unwrap();
        let feature_dir = tmp.path().join("specs/042-fresh");
        std::fs::create_dir_all(&feature_dir).unwrap();
        let spec = "---\nstatus: draft\ndependencies: []\n---\n\n# T\n\n\
                    ## Acceptance Criteria\n\n\
                    - [ ] A criterion that wraps across\n  two source lines.\n\
                    - [x] A single-line criterion.\n";
        std::fs::write(feature_dir.join("spec.md"), spec).unwrap();

        let result = run(
            &ReadSpecArgs {
                feature: "042-fresh".into(),
                include_body: false,
            },
            tmp.path(),
        )
        .unwrap();
        let texts: Vec<&str> = result
            .acceptance_criteria
            .iter()
            .map(|c| c.text.as_str())
            .collect();
        assert_eq!(
            texts,
            vec![
                "A criterion that wraps across two source lines.",
                "A single-line criterion.",
            ]
        );
        assert_eq!(
            result.acceptance_criteria.len(),
            2,
            "wrap must not add an entry"
        );
        assert!(!result.acceptance_criteria[0].checked);
        assert!(result.acceptance_criteria[1].checked);
    }

    #[test]
    fn missing_feature_errors() {
        let repo = fixture_repo();
        let err = run(
            &ReadSpecArgs {
                feature: "999-nonexistent".into(),
                include_body: false,
            },
            &repo,
        )
        .unwrap_err();
        matches!(err, PrimitiveError::FeatureNotFound { .. });
    }
}
