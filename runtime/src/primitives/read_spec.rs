//! `read-spec` — parse spec frontmatter and body sections.

use std::path::Path;

use crate::primitives::{
    PrimitiveError, Result, parse_atx_heading, read_text, rel_path, section_lines,
    split_frontmatter,
};
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
    let feature_dir = repo.join("specs").join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            feature: args.feature.clone(),
        });
    }
    let spec_path = feature_dir.join("spec.md");
    let content = read_text(&spec_path)?;
    let (fm_text, body) = split_frontmatter(&content, &spec_path)?;
    let frontmatter: Frontmatter =
        serde_yaml::from_str(fm_text).map_err(|source| PrimitiveError::Yaml {
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

fn parse_checkboxes(body: &str, section_heading: &str) -> Vec<AcceptanceCriterion> {
    let mut out = Vec::new();
    for line in section_lines(body, section_heading) {
        if let Some((checked, text)) = parse_checkbox_item(line) {
            out.push(AcceptanceCriterion { checked, text });
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

fn parse_checkbox_item(line: &str) -> Option<(bool, String)> {
    let trimmed = line.trim_start();
    let rest = trimmed.strip_prefix("- [")?;
    let bytes = rest.as_bytes();
    if bytes.len() < 3 || bytes[1] != b']' {
        return None;
    }
    let checked = matches!(bytes[0], b'x' | b'X');
    let text = rest[2..].trim().to_string();
    Some((checked, text))
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
