//! `validate-frontmatter` — full frontmatter schema check.
//!
//! Ports the semantics of `scripts/lint-frontmatter.sh` with real YAML
//! parsing rather than the shell-side shape check: every issue is reported
//! as a `FrontmatterFinding` rather than printed to stdout.

use std::path::Path;

use serde_norway::Value as YamlValue;

use crate::primitives::{Result, read_text, resolve_path, split_frontmatter};
use crate::schema::primitives::{
    FrontmatterFinding, ValidateFrontmatterArgs, ValidateFrontmatterResult,
};
use crate::schema::status::ALLOWED_STATUSES;

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

    // An empty frontmatter block (`---\n---\n`) parses as YAML null; treat
    // it as an empty mapping so the required-field checks below report
    // per-field findings rather than a misleading "must be a mapping".
    let empty_map = serde_norway::Mapping::new();
    let map = match &parsed {
        YamlValue::Mapping(map) => map,
        YamlValue::Null => &empty_map,
        _ => {
            findings.push(FrontmatterFinding {
                severity: "blocking".into(),
                field: String::new(),
                message: "frontmatter must be a mapping".into(),
            });
            return Ok(ValidateFrontmatterResult {
                findings,
                clean: false,
            });
        }
    };

    // `status` and `dependencies` are required on spec frontmatter —
    // absence is hard-fail per constitution §text-first-artifacts
    // (Validation Severity), same tier as an invalid value.
    match map.get("status") {
        Some(YamlValue::String(s)) => {
            if !ALLOWED_STATUSES.contains(&s.as_str()) {
                findings.push(FrontmatterFinding {
                    severity: "blocking".into(),
                    field: "status".into(),
                    message: format!("status '{s}' is not one of {}", ALLOWED_STATUSES.join("|")),
                });
            }
        }
        Some(_) => findings.push(FrontmatterFinding {
            severity: "blocking".into(),
            field: "status".into(),
            message: "status must be a string".into(),
        }),
        None => findings.push(FrontmatterFinding {
            severity: "blocking".into(),
            field: "status".into(),
            message: "status is missing".into(),
        }),
    }

    match map.get("dependencies") {
        Some(YamlValue::Sequence(items)) => {
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
        Some(_) => findings.push(FrontmatterFinding {
            severity: "blocking".into(),
            field: "dependencies".into(),
            message: "dependencies must be a list".into(),
        }),
        None => findings.push(FrontmatterFinding {
            severity: "blocking".into(),
            field: "dependencies".into(),
            message: "dependencies is missing".into(),
        }),
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
    fn missing_status_is_blocking() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("spec.md");
        std::fs::write(&path, "---\ndependencies: []\n---\n\n# X\n").unwrap();
        let result = run(
            &ValidateFrontmatterArgs {
                path: path.to_string_lossy().into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.clean);
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].severity, "blocking");
        assert_eq!(result.findings[0].field, "status");
        assert_eq!(result.findings[0].message, "status is missing");
    }

    #[test]
    fn missing_dependencies_is_blocking() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("spec.md");
        std::fs::write(&path, "---\nstatus: draft\n---\n\n# X\n").unwrap();
        let result = run(
            &ValidateFrontmatterArgs {
                path: path.to_string_lossy().into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.clean);
        assert_eq!(result.findings.len(), 1);
        assert_eq!(result.findings[0].severity, "blocking");
        assert_eq!(result.findings[0].field, "dependencies");
        assert_eq!(result.findings[0].message, "dependencies is missing");
    }

    #[test]
    fn empty_frontmatter_reports_both_missing_fields() {
        // Present-but-empty frontmatter is a validation finding, not a
        // MissingFrontmatter halt (scenario spec-side-parser-hardening).
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("spec.md");
        std::fs::write(&path, "---\n---\n\n# X\n").unwrap();
        let result = run(
            &ValidateFrontmatterArgs {
                path: path.to_string_lossy().into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.clean);
        let fields: Vec<&str> = result.findings.iter().map(|f| f.field.as_str()).collect();
        assert_eq!(fields, vec!["status", "dependencies"]);
        assert!(result.findings.iter().all(|f| f.severity == "blocking"));
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
