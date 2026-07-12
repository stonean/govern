//! `check-review-gate` — evaluate `/gov:implement`'s pre-done review gate.
//!
//! The deterministic surface behind the completion gate's step 13 (spec
//! 022, scenario coverage-expansion-primitives), which the host previously
//! walked by hand on every completion attempt: first the feature
//! directory's markdown lint (through the `lint-markdown` machinery,
//! replacing the raw `npx markdownlint-cli2` invocation), then the spec
//! frontmatter `review:` block. The first failing check wins and produces
//! the canonical `blocked: …` message — with the adopter's `[host]
//! project` command namespace substituted into the `/{project}:review`
//! references — plus, on `must-violations`, the resolve-or-waive
//! guidance. A blocked gate is a domain outcome the host acts on (halt,
//! do not propose the transition), never an operational error.

use std::path::Path;

use crate::host::Host;
use crate::primitives::{
    PrimitiveError, Result, lint_markdown, read_text, split_frontmatter, validate_no_traversal,
};
use crate::schema::paths;
use crate::schema::primitives::{
    CheckReviewGateArgs, CheckReviewGateResult, Frontmatter, LintMarkdownArgs, LintMarkdownResult,
    ReviewGateBlock,
};

/// Execute the `check-review-gate` primitive against the given repo root.
///
/// # Errors
///
/// Returns [`PrimitiveError::InvalidPath`] when `feature` is empty,
/// absolute, or carries a parent-directory component,
/// [`PrimitiveError::FeatureNotFound`] when the feature directory does
/// not exist, [`PrimitiveError::Io`] when `spec.md` is unreadable or
/// `npx` cannot be spawned, or [`PrimitiveError::Yaml`] for a malformed
/// frontmatter block. Every gate verdict — including all three block
/// reasons — is a domain outcome in the result.
pub fn run(args: &CheckReviewGateArgs, repo: &Path) -> Result<CheckReviewGateResult> {
    run_with_lint(args, repo, lint_markdown::run)
}

/// Implementation seam that lets unit tests inject a canned lint outcome
/// instead of spawning `npx markdownlint-cli2`. The MCP and CLI surfaces
/// both call [`run`], which forwards the real `lint-markdown` primitive.
pub(crate) fn run_with_lint(
    args: &CheckReviewGateArgs,
    repo: &Path,
    lint: impl FnOnce(&LintMarkdownArgs, &Path) -> Result<LintMarkdownResult>,
) -> Result<CheckReviewGateResult> {
    validate_no_traversal(&args.feature)?;
    let root = paths::Paths::load(repo).specs_root;
    let feature_dir = repo.join(&root).join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            root,
            feature: args.feature.clone(),
        });
    }
    let rel_dir = format!("{root}/{}", args.feature);

    // Gate check 1: every markdown file in the feature directory passes
    // markdownlint (recursive — scenarios/ included; `**` matches zero or
    // more directories, so the feature dir's own files are covered).
    let lint_result = lint(
        &LintMarkdownArgs {
            paths: vec![format!("{rel_dir}/**/*.md")],
            fix: false,
        },
        repo,
    )?;
    if !lint_result.clean {
        let message = if lint_result.violations.is_empty() {
            // Non-zero exit with nothing parseable: a config or runtime
            // error, or a violation shape the parser does not recognize.
            format!(
                "blocked: markdownlint-cli2 exited {} for {rel_dir} — resolve the lint failure before completing",
                lint_result.exit_code
            )
        } else {
            format!(
                "blocked: {} markdownlint violation(s) in {rel_dir} — resolve them before completing",
                lint_result.violations.len()
            )
        };
        return Ok(CheckReviewGateResult {
            passed: false,
            blocked_by: Some(ReviewGateBlock::MarkdownLint),
            message: Some(message),
            guidance: None,
            violations: lint_result.violations,
        });
    }

    // Gate checks 2 and 3: the spec frontmatter `review:` block.
    let spec_path = feature_dir.join("spec.md");
    let content = read_text(&spec_path)?;
    let (fm_text, _body) = split_frontmatter(&content, &spec_path)?;
    let frontmatter: Frontmatter =
        serde_norway::from_str(fm_text).map_err(|source| PrimitiveError::Yaml {
            path: spec_path.clone(),
            source,
        })?;
    let project = Host::load(repo).project;

    let review = match frontmatter.review {
        Some(review) if review.last_run.is_some() => review,
        // Absent block or null `last-run`: the spec has never completed a
        // review.
        _ => {
            return Ok(CheckReviewGateResult {
                passed: false,
                blocked_by: Some(ReviewGateBlock::NotReviewed),
                message: Some(format!(
                    "blocked: spec has not been reviewed — run /{project}:review before completing"
                )),
                guidance: None,
                violations: vec![],
            });
        }
    };

    if review.blocking {
        return Ok(CheckReviewGateResult {
            passed: false,
            blocked_by: Some(ReviewGateBlock::MustViolations),
            message: Some(format!(
                "blocked: spec has {} MUST violation(s) — see {rel_dir}/review.md",
                review.must_violations
            )),
            guidance: Some(format!(
                "Resolve the violations and re-run /{project}:review, or run \
                 /{project}:review --waive <rule-id> --reason \"...\" for each waivable finding."
            )),
            violations: vec![],
        });
    }

    Ok(CheckReviewGateResult {
        passed: true,
        blocked_by: None,
        message: None,
        guidance: None,
        violations: vec![],
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::schema::primitives::MarkdownViolation;
    use std::fs;
    use tempfile::tempdir;

    const REVIEWED_CLEAN: &str = "---\nstatus: in-progress\ndependencies: []\nreview:\n  last-run: 2026-07-10T00:00:00Z\n  reviewed-against: abc123\n  must-violations: 0\n  should-violations: 1\n  low-confidence: 0\n  blocking: false\n---\n\n# 007 — Gate\n";
    const REVIEWED_BLOCKING: &str = "---\nstatus: in-progress\ndependencies: []\nreview:\n  last-run: 2026-07-10T00:00:00Z\n  reviewed-against: abc123\n  must-violations: 3\n  should-violations: 0\n  low-confidence: 0\n  blocking: true\n---\n\n# 007 — Gate\n";
    const NEVER_REVIEWED: &str = "---\nstatus: in-progress\ndependencies: []\nreview:\n  last-run: null\n  reviewed-against: null\n  must-violations: 0\n  should-violations: 0\n  low-confidence: 0\n  blocking: false\n---\n\n# 007 — Gate\n";
    const NO_REVIEW_BLOCK: &str =
        "---\nstatus: in-progress\ndependencies: []\n---\n\n# 007 — Gate\n";

    fn seed(repo: &Path, spec: &str) {
        fs::create_dir_all(repo.join("specs/007-gate")).unwrap();
        fs::write(repo.join("specs/007-gate/spec.md"), spec).unwrap();
        // Pin the slash-command namespace so canonical messages are
        // deterministic (the default is the tempdir's random basename).
        fs::write(repo.join(".govern.toml"), "[host]\nproject = \"gov\"\n").unwrap();
    }

    fn args() -> CheckReviewGateArgs {
        CheckReviewGateArgs {
            feature: "007-gate".into(),
        }
    }

    // The wrapper matches the `run_with_lint` seam signature.
    #[allow(clippy::unnecessary_wraps)]
    fn clean_lint(_: &LintMarkdownArgs, _: &Path) -> Result<LintMarkdownResult> {
        Ok(LintMarkdownResult {
            violations: vec![],
            clean: true,
            exit_code: 0,
        })
    }

    #[test]
    fn passes_when_lint_clean_and_review_current() {
        let tmp = tempdir().unwrap();
        seed(tmp.path(), REVIEWED_CLEAN);
        let result = run_with_lint(&args(), tmp.path(), clean_lint).unwrap();
        assert!(result.passed);
        assert!(result.blocked_by.is_none());
        assert!(result.message.is_none());
        assert!(result.guidance.is_none());
        assert!(result.violations.is_empty());
    }

    #[test]
    fn lint_violations_block_before_review_state_is_consulted() {
        let tmp = tempdir().unwrap();
        // Even a never-reviewed spec reports the lint block first — the
        // gate's documented order.
        seed(tmp.path(), NEVER_REVIEWED);
        let violation = MarkdownViolation {
            path: "specs/007-gate/plan.md".into(),
            line: 12,
            rule: "MD012".into(),
            message: "Multiple consecutive blank lines".into(),
        };
        let canned = violation.clone();
        let result = run_with_lint(&args(), tmp.path(), move |_, _| {
            Ok(LintMarkdownResult {
                violations: vec![canned],
                clean: false,
                exit_code: 1,
            })
        })
        .unwrap();
        assert!(!result.passed);
        assert_eq!(result.blocked_by, Some(ReviewGateBlock::MarkdownLint));
        assert_eq!(
            result.message.as_deref(),
            Some(
                "blocked: 1 markdownlint violation(s) in specs/007-gate — resolve them before completing"
            )
        );
        assert_eq!(result.violations, vec![violation]);
    }

    #[test]
    fn unparseable_lint_failure_blocks_with_exit_code() {
        let tmp = tempdir().unwrap();
        seed(tmp.path(), REVIEWED_CLEAN);
        let result = run_with_lint(&args(), tmp.path(), |_, _| {
            Ok(LintMarkdownResult {
                violations: vec![],
                clean: false,
                exit_code: 2,
            })
        })
        .unwrap();
        assert!(!result.passed);
        assert_eq!(result.blocked_by, Some(ReviewGateBlock::MarkdownLint));
        assert_eq!(
            result.message.as_deref(),
            Some(
                "blocked: markdownlint-cli2 exited 2 for specs/007-gate — resolve the lint failure before completing"
            )
        );
        assert!(result.violations.is_empty());
    }

    #[test]
    fn lint_receives_the_recursive_feature_dir_glob() {
        let tmp = tempdir().unwrap();
        seed(tmp.path(), REVIEWED_CLEAN);
        let mut seen: Option<LintMarkdownArgs> = None;
        run_with_lint(&args(), tmp.path(), |lint_args, _| {
            seen = Some(lint_args.clone());
            clean_lint(lint_args, Path::new(""))
        })
        .unwrap();
        let seen = seen.unwrap();
        assert_eq!(seen.paths, vec!["specs/007-gate/**/*.md".to_string()]);
        assert!(!seen.fix, "the gate never lints in fix mode");
    }

    #[test]
    fn null_last_run_blocks_not_reviewed() {
        let tmp = tempdir().unwrap();
        seed(tmp.path(), NEVER_REVIEWED);
        let result = run_with_lint(&args(), tmp.path(), clean_lint).unwrap();
        assert!(!result.passed);
        assert_eq!(result.blocked_by, Some(ReviewGateBlock::NotReviewed));
        assert_eq!(
            result.message.as_deref(),
            Some("blocked: spec has not been reviewed — run /gov:review before completing")
        );
        assert!(result.guidance.is_none());
    }

    #[test]
    fn absent_review_block_blocks_not_reviewed() {
        let tmp = tempdir().unwrap();
        seed(tmp.path(), NO_REVIEW_BLOCK);
        let result = run_with_lint(&args(), tmp.path(), clean_lint).unwrap();
        assert!(!result.passed);
        assert_eq!(result.blocked_by, Some(ReviewGateBlock::NotReviewed));
    }

    #[test]
    fn blocking_review_blocks_with_must_count_and_waive_guidance() {
        let tmp = tempdir().unwrap();
        seed(tmp.path(), REVIEWED_BLOCKING);
        let result = run_with_lint(&args(), tmp.path(), clean_lint).unwrap();
        assert!(!result.passed);
        assert_eq!(result.blocked_by, Some(ReviewGateBlock::MustViolations));
        assert_eq!(
            result.message.as_deref(),
            Some("blocked: spec has 3 MUST violation(s) — see specs/007-gate/review.md")
        );
        let guidance = result.guidance.unwrap();
        assert!(guidance.contains("re-run /gov:review"), "{guidance}");
        assert!(guidance.contains("--waive <rule-id>"), "{guidance}");
    }

    #[test]
    fn honors_host_project_in_messages() {
        let tmp = tempdir().unwrap();
        seed(tmp.path(), NEVER_REVIEWED);
        fs::write(
            tmp.path().join(".govern.toml"),
            "[host]\nproject = \"anvil\"\n",
        )
        .unwrap();
        let result = run_with_lint(&args(), tmp.path(), clean_lint).unwrap();
        assert_eq!(
            result.message.as_deref(),
            Some("blocked: spec has not been reviewed — run /anvil:review before completing")
        );
    }

    #[test]
    fn honors_configured_specs_root() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("governance/007-gate")).unwrap();
        fs::write(
            tmp.path().join("governance/007-gate/spec.md"),
            REVIEWED_CLEAN,
        )
        .unwrap();
        fs::write(
            tmp.path().join(".govern.toml"),
            "[host]\nproject = \"gov\"\n\n[paths]\nspecs-root = \"governance\"\n",
        )
        .unwrap();
        let mut seen_glob = String::new();
        let result = run_with_lint(&args(), tmp.path(), |lint_args, _| {
            seen_glob = lint_args.paths[0].clone();
            Ok(LintMarkdownResult {
                violations: vec![],
                clean: true,
                exit_code: 0,
            })
        })
        .unwrap();
        assert!(result.passed);
        assert_eq!(seen_glob, "governance/007-gate/**/*.md");
    }

    #[test]
    fn missing_feature_directory_errors() {
        let tmp = tempdir().unwrap();
        seed(tmp.path(), REVIEWED_CLEAN);
        let err = run_with_lint(
            &CheckReviewGateArgs {
                feature: "099-absent".into(),
            },
            tmp.path(),
            clean_lint,
        )
        .unwrap_err();
        assert!(matches!(err, PrimitiveError::FeatureNotFound { .. }));
    }

    #[test]
    fn rejects_traversal_and_absolute_feature() {
        let tmp = tempdir().unwrap();
        seed(tmp.path(), REVIEWED_CLEAN);
        for bad in ["../007-gate", "/etc", ""] {
            let err = run_with_lint(
                &CheckReviewGateArgs {
                    feature: bad.into(),
                },
                tmp.path(),
                clean_lint,
            )
            .unwrap_err();
            assert!(
                matches!(err, PrimitiveError::InvalidPath { .. }),
                "expected InvalidPath for {bad:?}"
            );
        }
    }
}
