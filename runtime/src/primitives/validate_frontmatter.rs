//! `validate-frontmatter` — full frontmatter schema check.
//!
//! Ports the semantics of `scripts/lint-frontmatter.sh` with real YAML
//! parsing rather than the shell-side shape check: every issue is reported
//! as a `FrontmatterFinding` rather than printed to stdout.

use std::path::{Path, PathBuf};

use serde_norway::Value as YamlValue;

use crate::primitives::{Result, read_text, split_frontmatter};
use crate::schema::primitives::{
    FrontmatterFinding, ValidateFrontmatterArgs, ValidateFrontmatterResult,
};

const ALLOWED_STATUSES: &[&str] = &["draft", "clarified", "planned", "in-progress", "done"];

/// Execute the `validate-frontmatter` primitive.
///
/// # Errors
///
/// Returns [`crate::primitives::PrimitiveError::Io`] when the file cannot
/// be read or [`crate::primitives::PrimitiveError::MissingFrontmatter`]
/// when no `---` fence pair is present. YAML parse failures surface as
/// findings, not operational errors.
pub fn run(args: &ValidateFrontmatterArgs, repo: &Path) -> Result<ValidateFrontmatterResult> {
    let path = resolve_path(repo, &args.path);
    let content = read_text(&path)?;
    let (fm_text, _body) = split_frontmatter(&content, &path)?;

    let mut findings: Vec<FrontmatterFinding> = Vec::new();
    let parsed: YamlValue = match serde_norway::from_str(fm_text) {
        Ok(v) => v,
        Err(e) => {
            findings.push(FrontmatterFinding {
                severity: "blocking".into(),
                field: String::new(),
                message: format!("frontmatter is not valid YAML: {e}"),
            });
            return Ok(ValidateFrontmatterResult {
                findings,
                clean: false,
            });
        }
    };

    let YamlValue::Mapping(map) = &parsed else {
        findings.push(FrontmatterFinding {
            severity: "blocking".into(),
            field: String::new(),
            message: "frontmatter must be a mapping".into(),
        });
        return Ok(ValidateFrontmatterResult {
            findings,
            clean: false,
        });
    };

    if let Some(status) = map.get("status") {
        match status {
            YamlValue::String(s) => {
                if !ALLOWED_STATUSES.contains(&s.as_str()) {
                    findings.push(FrontmatterFinding {
                        severity: "blocking".into(),
                        field: "status".into(),
                        message: format!(
                            "status '{s}' is not one of {}",
                            ALLOWED_STATUSES.join("|")
                        ),
                    });
                }
            }
            _ => findings.push(FrontmatterFinding {
                severity: "blocking".into(),
                field: "status".into(),
                message: "status must be a string".into(),
            }),
        }
    }

    if let Some(deps) = map.get("dependencies") {
        if !matches!(deps, YamlValue::Sequence(_)) {
            findings.push(FrontmatterFinding {
                severity: "blocking".into(),
                field: "dependencies".into(),
                message: "dependencies must be a list".into(),
            });
        } else if let YamlValue::Sequence(items) = deps {
            for (i, item) in items.iter().enumerate() {
                if !matches!(item, YamlValue::String(_)) {
                    findings.push(FrontmatterFinding {
                        severity: "blocking".into(),
                        field: format!("dependencies[{i}]"),
                        message: "dependency entry must be a string feature name".into(),
                    });
                }
            }
        }
    }

    if let Some(review) = map.get("review") {
        validate_review_block(review, &mut findings);
    }

    let clean = findings.is_empty();
    Ok(ValidateFrontmatterResult { findings, clean })
}

fn validate_review_block(review: &YamlValue, findings: &mut Vec<FrontmatterFinding>) {
    let YamlValue::Mapping(map) = review else {
        findings.push(FrontmatterFinding {
            severity: "blocking".into(),
            field: "review".into(),
            message: "review must be a mapping".into(),
        });
        return;
    };
    for key in ["must-violations", "should-violations", "low-confidence"] {
        if let Some(value) = map.get(key)
            && !matches!(value, YamlValue::Number(_))
        {
            findings.push(FrontmatterFinding {
                severity: "blocking".into(),
                field: format!("review.{key}"),
                message: "must be a number".into(),
            });
        }
    }
    if let Some(value) = map.get("blocking")
        && !matches!(value, YamlValue::Bool(_))
    {
        findings.push(FrontmatterFinding {
            severity: "blocking".into(),
            field: "review.blocking".into(),
            message: "must be a boolean".into(),
        });
    }
}

fn resolve_path(repo: &Path, path_arg: &str) -> PathBuf {
    let candidate = Path::new(path_arg);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo.join(candidate)
    }
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
    fn fixture_spec_is_clean() {
        let repo = fixture_repo();
        let result = run(
            &ValidateFrontmatterArgs {
                path: "specs/001-basic/spec.md".into(),
            },
            &repo,
        )
        .unwrap();
        assert!(result.clean, "expected clean, got {:?}", result.findings);
        assert!(result.findings.is_empty());
    }

    #[test]
    fn unknown_status_is_blocking() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("spec.md");
        std::fs::write(&path, "---\nstatus: wibble\ndependencies: []\n---\n\n# X\n").unwrap();
        let result = run(
            &ValidateFrontmatterArgs {
                path: path.to_string_lossy().into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.clean);
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].field, "status");
    }

    #[test]
    fn dependencies_must_be_a_list() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("spec.md");
        std::fs::write(
            &path,
            "---\nstatus: draft\ndependencies: not-a-list\n---\n\n# X\n",
        )
        .unwrap();
        let result = run(
            &ValidateFrontmatterArgs {
                path: path.to_string_lossy().into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.clean);
        assert!(result.findings.iter().any(|f| f.field == "dependencies"));
    }
}
