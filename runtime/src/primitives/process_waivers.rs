//! `process-waivers` — deterministic per-run waiver processing for `/gov:review`.
//!
//! Reads `review.waivers` from a spec's frontmatter and classifies each entry
//! against the currently-firing `(rule, file)` findings:
//!
//! - **apply** — the anchored file exists AND the rule still fires there.
//! - **expire** — the file is gone OR the rule no longer fires there. The
//!   entry drops on the next frontmatter write (write-review's job); this
//!   primitive emits the `waiver expired: …` notice and reports the anchor.
//! - **malformed** — a field is missing/empty; warn and skip, never prune.
//! - **duplicate** — a repeated `(rule, file)` pair; only the first applies.
//!
//! The anchor is the `(rule, file)` pair only — line numbers are not part of
//! it, so code moving within a file does not expire a waiver. Read-only:
//! frontmatter mutation belongs to `write-review`.
//!
//! Defined by
//! `specs/022-deterministic-runtime/scenarios/review-runtime-acceleration.md`.

use std::path::Path;

use serde::Deserialize;

use crate::primitives::{PrimitiveError, Result, read_text, split_frontmatter};
use crate::schema::paths;
use crate::schema::primitives::{ProcessWaiversArgs, ProcessWaiversResult, WaiverRef};

/// Required waiver fields, checked in this order; the first missing/empty one
/// names the `malformed …` diagnostic.
const REQUIRED_FIELDS: &[&str] = &["rule", "file", "reason", "waived-at", "waived-by"];

/// Execute the `process-waivers` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeatureNotFound`] when the feature has no
/// `spec.md`, [`PrimitiveError::MissingFrontmatter`] when that file has no
/// frontmatter block, [`PrimitiveError::Yaml`] when the frontmatter fails to
/// parse, or [`PrimitiveError::Io`] on read failure.
pub fn run(args: &ProcessWaiversArgs, repo: &Path) -> Result<ProcessWaiversResult> {
    super::validate_no_traversal(&args.feature)?;
    let layout = paths::Paths::load(repo);
    let spec_path = repo
        .join(&layout.specs_root)
        .join(&args.feature)
        .join("spec.md");
    if !spec_path.is_file() {
        return Err(PrimitiveError::FeatureNotFound {
            root: layout.specs_root,
            feature: args.feature.clone(),
        });
    }
    let content = read_text(&spec_path)?;
    let (fm_text, _body) = split_frontmatter(&content, &spec_path)?;
    let frontmatter: SpecFrontmatter =
        serde_norway::from_str(fm_text).map_err(|source| PrimitiveError::Yaml {
            path: spec_path.clone(),
            source,
        })?;
    let waivers = frontmatter
        .review
        .map(|review| review.waivers)
        .unwrap_or_default();

    let mut applied: Vec<WaiverRef> = Vec::new();
    let mut expired: Vec<WaiverRef> = Vec::new();
    let mut notices: Vec<String> = Vec::new();
    let mut seen: Vec<(String, String)> = Vec::new();

    for (index, waiver) in waivers.iter().enumerate() {
        if let Some(field) = first_missing_field(waiver) {
            notices.push(format!(
                "malformed waiver at review.waivers[{index}]: missing '{field}'"
            ));
            continue;
        }
        // Safe: `first_missing_field` returned `None`, so each is present and
        // non-empty.
        let rule = waiver.rule.clone().unwrap_or_default();
        let file = waiver.file.clone().unwrap_or_default();
        let reason = waiver.reason.clone().unwrap_or_default();

        if seen.iter().any(|(r, f)| r == &rule && f == &file) {
            notices.push(format!(
                "duplicate waiver: rule {rule} at {file} — entry [{index}] ignored"
            ));
            continue;
        }
        seen.push((rule.clone(), file.clone()));

        let file_exists = repo.join(&file).exists();
        let rule_fires = args
            .fired
            .iter()
            .any(|finding| finding.rule == rule && finding.file == file);

        if file_exists && rule_fires {
            applied.push(WaiverRef { rule, file, reason });
        } else {
            notices.push(format!("waiver expired: rule {rule} at {file} ({reason})"));
            expired.push(WaiverRef { rule, file, reason });
        }
    }

    Ok(ProcessWaiversResult {
        applied,
        expired,
        notices,
    })
}

/// Return the first required field that is absent or empty, or `None` when the
/// waiver is well-formed.
fn first_missing_field(waiver: &RawWaiver) -> Option<&'static str> {
    let values = [
        waiver.rule.as_deref(),
        waiver.file.as_deref(),
        waiver.reason.as_deref(),
        waiver.waived_at.as_deref(),
        waiver.waived_by.as_deref(),
    ];
    REQUIRED_FIELDS
        .iter()
        .zip(values)
        .find(|(_, value)| value.unwrap_or("").trim().is_empty())
        .map(|(field, _)| *field)
}

/// Minimal spec frontmatter shape: just the `review.waivers` list.
#[derive(Deserialize)]
struct SpecFrontmatter {
    #[serde(default)]
    review: Option<ReviewFrontmatter>,
}

/// `review:` block — only `waivers` is consulted here.
#[derive(Deserialize, Default)]
struct ReviewFrontmatter {
    #[serde(default)]
    waivers: Vec<RawWaiver>,
}

/// One waiver entry, parsed loosely so a malformed entry (missing field) is a
/// reportable warning rather than a whole-frontmatter parse failure.
#[derive(Deserialize)]
struct RawWaiver {
    #[serde(default)]
    rule: Option<String>,
    #[serde(default)]
    file: Option<String>,
    #[serde(default)]
    reason: Option<String>,
    #[serde(default, rename = "waived-at")]
    waived_at: Option<String>,
    #[serde(default, rename = "waived-by")]
    waived_by: Option<String>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::schema::primitives::FiredFinding;
    use tempfile::TempDir;

    /// Write `specs/{feature}/spec.md` with the given `review.waivers` YAML
    /// block (already indented under `waivers:`), and touch each path in
    /// `existing_files` relative to the repo root.
    fn setup(feature: &str, waivers_yaml: &str, existing_files: &[&str]) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("specs").join(feature);
        std::fs::create_dir_all(&dir).unwrap();
        let content = format!(
            "---\nstatus: in-progress\ndependencies: []\nreview:\n  waivers:\n{waivers_yaml}---\n\n# Spec\n"
        );
        std::fs::write(dir.join("spec.md"), content).unwrap();
        for rel in existing_files {
            let path = tmp.path().join(rel);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, "code\n").unwrap();
        }
        tmp
    }

    fn fired(pairs: &[(&str, &str)]) -> Vec<FiredFinding> {
        pairs
            .iter()
            .map(|(rule, file)| FiredFinding {
                rule: (*rule).to_string(),
                file: (*file).to_string(),
            })
            .collect()
    }

    fn args(feature: &str, fired_pairs: &[(&str, &str)]) -> ProcessWaiversArgs {
        ProcessWaiversArgs {
            feature: feature.to_string(),
            fired: fired(fired_pairs),
        }
    }

    const ONE_WAIVER: &str = "    - rule: SEC-BE-014\n      file: src/api/internal.ts\n      reason: Endpoint is internal-only behind mTLS.\n      waived-at: 2026-05-10T14:40:00Z\n      waived-by: dev@example.com\n";

    #[test]
    fn applies_when_file_exists_and_rule_fires() {
        let tmp = setup("001-x", ONE_WAIVER, &["src/api/internal.ts"]);
        let result = run(
            &args("001-x", &[("SEC-BE-014", "src/api/internal.ts")]),
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.applied.len(), 1);
        assert_eq!(result.applied[0].rule, "SEC-BE-014");
        assert_eq!(result.applied[0].file, "src/api/internal.ts");
        assert!(result.expired.is_empty());
        assert!(result.notices.is_empty());
    }

    #[test]
    fn expires_when_file_is_gone() {
        // File not created → anchor no longer exists.
        let tmp = setup("001-x", ONE_WAIVER, &[]);
        let result = run(
            &args("001-x", &[("SEC-BE-014", "src/api/internal.ts")]),
            tmp.path(),
        )
        .unwrap();
        assert!(result.applied.is_empty());
        assert_eq!(result.expired.len(), 1);
        assert_eq!(
            result.notices,
            vec![
                "waiver expired: rule SEC-BE-014 at src/api/internal.ts (Endpoint is internal-only behind mTLS.)"
            ]
        );
    }

    #[test]
    fn expires_when_rule_no_longer_fires() {
        // File exists but the rule is not in the fired set.
        let tmp = setup("001-x", ONE_WAIVER, &["src/api/internal.ts"]);
        let result = run(&args("001-x", &[]), tmp.path()).unwrap();
        assert!(result.applied.is_empty());
        assert_eq!(result.expired.len(), 1);
        assert!(result.notices[0].starts_with("waiver expired: rule SEC-BE-014"));
    }

    #[test]
    fn does_not_extend_to_a_different_file() {
        // The rule fires, but at a different file than the waiver's anchor.
        let tmp = setup(
            "001-x",
            ONE_WAIVER,
            &["src/api/internal.ts", "src/api/other.ts"],
        );
        let result = run(
            &args("001-x", &[("SEC-BE-014", "src/api/other.ts")]),
            tmp.path(),
        )
        .unwrap();
        // The waiver anchors (SEC-BE-014, internal.ts), which does not fire →
        // it expires; other.ts is a separate finding, not covered here.
        assert!(result.applied.is_empty());
        assert_eq!(result.expired.len(), 1);
    }

    #[test]
    fn code_moving_within_file_does_not_expire() {
        // The anchor is (rule, file) only — no line number. The rule still
        // fires in the same file, so the waiver applies regardless of line.
        let tmp = setup("001-x", ONE_WAIVER, &["src/api/internal.ts"]);
        let result = run(
            &args("001-x", &[("SEC-BE-014", "src/api/internal.ts")]),
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.applied.len(), 1);
    }

    #[test]
    fn malformed_missing_reason_is_skipped_with_warning() {
        let waiver = "    - rule: SEC-BE-014\n      file: src/api/internal.ts\n      waived-at: 2026-05-10T14:40:00Z\n      waived-by: dev@example.com\n";
        let tmp = setup("001-x", waiver, &["src/api/internal.ts"]);
        let result = run(
            &args("001-x", &[("SEC-BE-014", "src/api/internal.ts")]),
            tmp.path(),
        )
        .unwrap();
        assert!(result.applied.is_empty());
        assert!(result.expired.is_empty());
        assert_eq!(
            result.notices,
            vec!["malformed waiver at review.waivers[0]: missing 'reason'"]
        );
    }

    #[test]
    fn malformed_missing_waived_by_names_that_field() {
        let waiver = "    - rule: SEC-BE-014\n      file: src/api/internal.ts\n      reason: Internal-only endpoint behind mTLS.\n      waived-at: 2026-05-10T14:40:00Z\n";
        let tmp = setup("001-x", waiver, &["src/api/internal.ts"]);
        let result = run(&args("001-x", &[]), tmp.path()).unwrap();
        assert_eq!(
            result.notices,
            vec!["malformed waiver at review.waivers[0]: missing 'waived-by'"]
        );
    }

    #[test]
    fn duplicate_first_applies_rest_warn() {
        let waivers = format!(
            "{ONE_WAIVER}    - rule: SEC-BE-014\n      file: src/api/internal.ts\n      reason: Duplicate entry that should be ignored.\n      waived-at: 2026-05-11T00:00:00Z\n      waived-by: dev@example.com\n"
        );
        let tmp = setup("001-x", &waivers, &["src/api/internal.ts"]);
        let result = run(
            &args("001-x", &[("SEC-BE-014", "src/api/internal.ts")]),
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.applied.len(), 1);
        assert_eq!(
            result.notices,
            vec!["duplicate waiver: rule SEC-BE-014 at src/api/internal.ts — entry [1] ignored"]
        );
    }

    #[test]
    fn no_waivers_yields_empty_result() {
        let tmp = TempDir::new().unwrap();
        let dir = tmp.path().join("specs/001-x");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("spec.md"),
            "---\nstatus: in-progress\ndependencies: []\n---\n\n# Spec\n",
        )
        .unwrap();
        let result = run(&args("001-x", &[]), tmp.path()).unwrap();
        assert!(result.applied.is_empty());
        assert!(result.expired.is_empty());
        assert!(result.notices.is_empty());
    }

    #[test]
    fn missing_feature_is_operational_error() {
        let tmp = TempDir::new().unwrap();
        let err = run(&args("999-nope", &[]), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::FeatureNotFound { .. }));
    }
}
