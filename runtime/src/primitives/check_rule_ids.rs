//! `check-rule-ids` — verify cited rule IDs exist in rule files and aren't
//! deprecated.

#![allow(clippy::expect_used)]

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

use crate::primitives::{Result, read_text};
use crate::schema::primitives::{CheckRuleIdsArgs, CheckRuleIdsResult, RuleCitation};

/// Execute the `check-rule-ids` primitive.
///
/// # Errors
///
/// Returns [`crate::primitives::PrimitiveError::Io`] when the target file
/// or any rule file cannot be read.
pub fn run(args: &CheckRuleIdsArgs, repo: &Path) -> Result<CheckRuleIdsResult> {
    let mut known: HashMap<String, bool> = HashMap::new();
    for rule_file in &args.rule_files {
        let path = resolve(repo, rule_file);
        let content = read_text(&path)?;
        for cap in heading_id_regex().captures_iter(&content) {
            let id = cap[1].to_string();
            let deprecated = is_deprecated(&content, &id);
            known.insert(id, deprecated);
        }
    }

    let path = resolve(repo, &args.path);
    let content = read_text(&path)?;
    let mut citations: Vec<RuleCitation> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();
    let mut missing: HashSet<String> = HashSet::new();
    let mut deprecated_hits: HashSet<String> = HashSet::new();
    for cap in citation_regex().captures_iter(&content) {
        let id = cap[0].to_string();
        if !seen.insert(id.clone()) {
            continue;
        }
        match known.get(&id).copied() {
            Some(true) => {
                deprecated_hits.insert(id.clone());
                citations.push(RuleCitation {
                    rule_id: id,
                    found: true,
                    deprecated: true,
                });
            }
            Some(false) => citations.push(RuleCitation {
                rule_id: id,
                found: true,
                deprecated: false,
            }),
            None => {
                missing.insert(id.clone());
                citations.push(RuleCitation {
                    rule_id: id,
                    found: false,
                    deprecated: false,
                });
            }
        }
    }

    let mut missing: Vec<String> = missing.into_iter().collect();
    missing.sort();
    let mut deprecated: Vec<String> = deprecated_hits.into_iter().collect();
    deprecated.sort();
    Ok(CheckRuleIdsResult {
        citations,
        missing,
        deprecated,
    })
}

fn resolve(repo: &Path, path_arg: &str) -> std::path::PathBuf {
    let candidate = Path::new(path_arg);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo.join(candidate)
    }
}

/// `### BE-AUTHN-001` — matches rule-ID headings inside a rule file. The
/// category segment follows the schema grammar `[A-Z][A-Z0-9]*` (digits are
/// permitted after the first letter, e.g. `FE-A11YFORM-001`), per
/// `specs/008-security-rules/data-model.md`.
fn heading_id_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"(?m)^#{2,4}\s+([A-Z]{2,5}-[A-Z][A-Z0-9]+-\d{3,4})\b")
            .expect("hard-coded regex compiles")
    })
}

/// Plain text citations: `BE-AUTHN-001` anywhere in a body (not the heading
/// line itself for the rule file's self-references). The category segment
/// mirrors `heading_id_regex` and allows digit-bearing categories.
fn citation_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"\b[A-Z]{2,5}-[A-Z][A-Z0-9]+-\d{3,4}\b").expect("hard-coded regex compiles")
    })
}

/// Detect the canonical deprecation label `**DEPRECATED in {version}:**`
/// (per `specs/008-security-rules/data-model.md` and constitution §rules)
/// within a window following the rule ID. Matching the bold-uppercase
/// `**DEPRECATED` token avoids false positives from prose mentions of
/// "deprecated" (lowercase, unbolded) such as a rule's own rationale.
fn is_deprecated(content: &str, id: &str) -> bool {
    let mut idx = 0usize;
    while let Some(pos) = content[idx..].find(id) {
        let abs = idx + pos;
        // The 256-byte window can land mid-UTF-8-character (rule files are
        // em-dash-dense); walk back to a char boundary so the slice cannot
        // panic. `abs` is a match start, so it is always a boundary.
        let mut window_end = (abs + 256).min(content.len());
        while !content.is_char_boundary(window_end) {
            window_end -= 1;
        }
        let window = &content[abs..window_end];
        if window.contains("**DEPRECATED") {
            return true;
        }
        idx = abs + id.len();
    }
    false
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
    fn fixture_spec_cites_known_rule() {
        let repo = fixture_repo();
        let result = run(
            &CheckRuleIdsArgs {
                path: "specs/001-basic/spec.md".into(),
                rule_files: vec!["framework/rules/security-backend.md".into()],
            },
            &repo,
        )
        .unwrap();

        let ids: Vec<&str> = result
            .citations
            .iter()
            .map(|c| c.rule_id.as_str())
            .collect();
        assert!(ids.contains(&"BE-AUTHN-001"));
        assert!(
            result.missing.is_empty(),
            "unexpected missing: {:?}",
            result.missing
        );
        assert!(result.deprecated.is_empty());
    }

    #[test]
    fn missing_rule_is_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("spec.md");
        std::fs::write(
            &spec_path,
            "# Demo\n\nReferences BE-AUTHN-001 and BE-MISSING-999.\n",
        )
        .unwrap();
        let rule_path = tmp.path().join("rules.md");
        std::fs::write(&rule_path, "# Rules\n\n### BE-AUTHN-001\n\n> Hashed.\n").unwrap();
        let result = run(
            &CheckRuleIdsArgs {
                path: spec_path.to_string_lossy().into(),
                rule_files: vec![rule_path.to_string_lossy().into()],
            },
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.missing, vec!["BE-MISSING-999".to_string()]);
        let cited: HashMap<&str, &RuleCitation> = result
            .citations
            .iter()
            .map(|c| (c.rule_id.as_str(), c))
            .collect();
        assert!(
            cited
                .get("BE-AUTHN-001")
                .is_some_and(|c| c.found && !c.deprecated)
        );
        assert!(cited.get("BE-MISSING-999").is_some_and(|c| !c.found));
    }

    #[test]
    fn deprecated_rule_is_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("spec.md");
        std::fs::write(&spec_path, "Cites BE-OLD-001.\n").unwrap();
        let rule_path = tmp.path().join("rules.md");
        std::fs::write(
            &rule_path,
            "### BE-OLD-001\n\n**DEPRECATED in 0.5.0:** replaced by BE-NEW-002.\n",
        )
        .unwrap();
        let result = run(
            &CheckRuleIdsArgs {
                path: spec_path.to_string_lossy().into(),
                rule_files: vec![rule_path.to_string_lossy().into()],
            },
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.deprecated, vec!["BE-OLD-001".to_string()]);
        assert!(result.missing.is_empty());
    }

    #[test]
    fn multibyte_content_at_window_edge_does_not_panic() {
        // Byte `abs + 256` of the deprecation window must be walked back to
        // a char boundary when it lands inside a multibyte character
        // (scenario spec-side-parser-hardening). Padding is sized so the
        // window edge lands mid-em-dash.
        let rule_content = format!(
            "### BE-EM-001\n\n{}{}\n",
            "a".repeat(243),
            "\u{2014}".repeat(20)
        );
        let abs = rule_content.find("BE-EM-001").unwrap();
        assert!(
            !rule_content.is_char_boundary(abs + 256),
            "fixture must place the window edge inside a multibyte char"
        );

        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("spec.md");
        std::fs::write(&spec_path, "Cites BE-EM-001.\n").unwrap();
        let rule_path = tmp.path().join("rules.md");
        std::fs::write(&rule_path, &rule_content).unwrap();
        let result = run(
            &CheckRuleIdsArgs {
                path: spec_path.to_string_lossy().into(),
                rule_files: vec![rule_path.to_string_lossy().into()],
            },
            tmp.path(),
        )
        .unwrap();
        assert!(result.missing.is_empty());
        assert!(result.deprecated.is_empty());
        assert!(
            result
                .citations
                .iter()
                .any(|c| c.rule_id == "BE-EM-001" && c.found && !c.deprecated)
        );
    }

    #[test]
    fn digit_bearing_category_is_recognized() {
        // Schema permits `[A-Z][A-Z0-9]*` categories; `FE-A11YFORM-*` and
        // `FE-A11YMEDIA-*` (accessibility-frontend.md) carry digits. The ID
        // regex must harvest those headings and match their citations.
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("spec.md");
        std::fs::write(&spec_path, "Cites FE-A11YFORM-001 and FE-A11YMEDIA-002.\n").unwrap();
        let rule_path = tmp.path().join("rules.md");
        std::fs::write(
            &rule_path,
            "### FE-A11YFORM-001\n\n> Labels.\n\n### FE-A11YMEDIA-002\n\n> Alt text.\n",
        )
        .unwrap();
        let result = run(
            &CheckRuleIdsArgs {
                path: spec_path.to_string_lossy().into(),
                rule_files: vec![rule_path.to_string_lossy().into()],
            },
            tmp.path(),
        )
        .unwrap();
        assert!(
            result.missing.is_empty(),
            "digit-bearing categories misreported as missing: {:?}",
            result.missing
        );
        let found: Vec<&str> = result
            .citations
            .iter()
            .filter(|c| c.found)
            .map(|c| c.rule_id.as_str())
            .collect();
        assert!(found.contains(&"FE-A11YFORM-001"));
        assert!(found.contains(&"FE-A11YMEDIA-002"));
    }
}
