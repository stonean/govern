//! `check-rule-ids` — verify cited rule IDs exist in rule files and aren't
//! deprecated.

#![allow(clippy::expect_used)]

use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

use crate::primitives::{Result, parse_atx_heading, read_text};
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
            let Some(whole) = cap.get(0) else { continue };
            let id = cap[1].to_string();
            let deprecated = section_is_deprecated(&content, whole.start());
            // A rule ID defined by more than one heading (unusual, but
            // representable) is deprecated when ANY defining section
            // carries the label.
            known
                .entry(id)
                .and_modify(|d| *d = *d || deprecated)
                .or_insert(deprecated);
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
/// inside the rule's own section: from the ID's defining heading (whose
/// match starts at `heading_start`) to the next heading of the same or
/// higher level (fewer or equal `#`s). Deeper headings are subsections of
/// the rule and stay in scope. The predecessor scanned a fixed 256-byte
/// window after ANY occurrence of the ID anywhere in the file, so a live
/// rule adjacent to a deprecated neighbor false-flagged. Matching the
/// bold-uppercase `**DEPRECATED` token avoids false positives from prose
/// mentions of "deprecated" (lowercase, unbolded) such as a rule's own
/// rationale.
///
/// Slicing only happens at regex match offsets and line boundaries — all
/// guaranteed char boundaries — preserving the earlier fix for windows
/// that landed mid-UTF-8-character in em-dash-dense rule files.
fn section_is_deprecated(content: &str, heading_start: usize) -> bool {
    let mut lines = content[heading_start..].lines();
    let Some(heading_line) = lines.next() else {
        return false;
    };
    let heading_level = parse_atx_heading(heading_line).map_or(4, |(level, _)| level);
    for line in lines {
        if let Some((level, _)) = parse_atx_heading(line)
            && level <= heading_level
        {
            // Same-or-higher-level heading: the rule's section ended.
            return false;
        }
        if line.contains("**DEPRECATED") {
            return true;
        }
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
    fn live_rule_adjacent_to_deprecated_neighbor_is_not_flagged() {
        // Scenario primitive-robustness-hardening: the old 256-byte
        // window after any ID occurrence bled into the NEXT rule's
        // section — a short live rule directly above a deprecated rule
        // false-flagged. The scan is now scoped to the rule's own
        // section (heading to next same-or-higher-level heading).
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("spec.md");
        std::fs::write(&spec_path, "Cites BE-LIVE-001 and BE-OLD-002.\n").unwrap();
        let rule_path = tmp.path().join("rules.md");
        std::fs::write(
            &rule_path,
            "### BE-LIVE-001\n\n> Short live rule.\n\n\
             ### BE-OLD-002\n\n**DEPRECATED in 0.5.0:** replaced by BE-NEW-003.\n",
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
        assert_eq!(
            result.deprecated,
            vec!["BE-OLD-002".to_string()],
            "only the genuinely deprecated rule may be flagged"
        );
        let cited: HashMap<&str, &RuleCitation> = result
            .citations
            .iter()
            .map(|c| (c.rule_id.as_str(), c))
            .collect();
        assert!(
            cited
                .get("BE-LIVE-001")
                .is_some_and(|c| c.found && !c.deprecated),
            "live rule adjacent to a deprecated neighbor must not be flagged"
        );
        assert!(
            cited
                .get("BE-OLD-002")
                .is_some_and(|c| c.found && c.deprecated)
        );
    }

    #[test]
    fn deprecation_label_inside_deeper_subsection_still_counts() {
        // A `####` subsection belongs to the `###` rule's section; a
        // label there still deprecates the rule. The section ends only
        // at the next same-or-higher-level heading.
        let tmp = tempfile::tempdir().unwrap();
        let spec_path = tmp.path().join("spec.md");
        std::fs::write(&spec_path, "Cites BE-SUB-001 and BE-NEXT-002.\n").unwrap();
        let rule_path = tmp.path().join("rules.md");
        std::fs::write(
            &rule_path,
            "### BE-SUB-001\n\n> Rule text.\n\n#### Notes\n\n**DEPRECATED in 0.6.0:** see BE-NEXT-002.\n\n\
             ### BE-NEXT-002\n\n> Live successor.\n",
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
        assert_eq!(result.deprecated, vec!["BE-SUB-001".to_string()]);
        let cited: HashMap<&str, &RuleCitation> = result
            .citations
            .iter()
            .map(|c| (c.rule_id.as_str(), c))
            .collect();
        assert!(
            cited
                .get("BE-NEXT-002")
                .is_some_and(|c| c.found && !c.deprecated)
        );
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
