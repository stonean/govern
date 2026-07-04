//! `write-review` — render `specs/NNN/review.md` and update the spec's
//! `review:` frontmatter block for `/gov:review`.
//!
//! Consumes the pass findings as a single `findings` array (the
//! content-ingestion convention), plus the waiver results from
//! `process-waivers` and the scope scalars from `compute-review-scope`, and:
//!
//! - applies the deterministic **cross-pass dedup** — same `(rule, file)` with
//!   overlapping line ranges collapses to one finding, highest-severity-wins
//!   (tie broken by higher confidence);
//! - **buckets** the survivors: findings matched by an applied waiver drop out
//!   of the counts into Waived findings; a `low`-confidence finding lands in
//!   Low-confidence regardless of severity; the rest split MUST / SHOULD;
//! - renders the fixed report skeleton and updates the spec `review:` block
//!   (`last-run`, `reviewed-against`, `must-violations`, `should-violations`,
//!   `low-confidence`, `blocking`), pruning any **expired** waiver entries from
//!   `review.waivers` on the write (per `process-waivers`' contract);
//! - the empty-scope case is a branch of this primitive, not a prose
//!   special-case: it emits the 0-findings, `blocking: false` report.
//!
//! Both writes are atomic (tempfile + rename). `blocking` is true exactly when
//! `must-violations` exceeds zero, and the exit code (0 / 1) is derivable from
//! it. Defined by
//! `specs/022-deterministic-runtime/scenarios/review-runtime-acceleration.md`.

use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize;

use crate::primitives::{
    PrimitiveError, Result, read_text, rel_path, split_frontmatter, write_atomic,
};
use crate::schema::paths;
use crate::schema::primitives::{ReviewFinding, WriteReviewArgs, WriteReviewResult};

/// Execute the `write-review` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeatureNotFound`] when the feature has no
/// `spec.md`, [`PrimitiveError::MissingFrontmatter`] when that file has no
/// frontmatter block, [`PrimitiveError::Yaml`] when the frontmatter fails to
/// parse, or [`PrimitiveError::Io`] on read/write failure.
pub fn run(args: &WriteReviewArgs, repo: &Path) -> Result<WriteReviewResult> {
    let root = paths::Paths::load(repo).specs_root;
    let feature_dir = repo.join(&root).join(&args.feature);
    let spec_path = feature_dir.join("spec.md");
    if !spec_path.is_file() {
        return Err(PrimitiveError::FeatureNotFound {
            root,
            feature: args.feature.clone(),
        });
    }

    // Dedup, then bucket the survivors.
    let deduped = dedup_findings(&args.findings);
    let mut must: Vec<&ReviewFinding> = Vec::new();
    let mut should: Vec<&ReviewFinding> = Vec::new();
    let mut low: Vec<&ReviewFinding> = Vec::new();
    let mut waived: Vec<&ReviewFinding> = Vec::new();
    for finding in &deduped {
        if waiver_reason(finding, &args.applied_waivers).is_some() {
            waived.push(finding);
        } else if finding.confidence.eq_ignore_ascii_case("low") {
            low.push(finding);
        } else if finding.severity.eq_ignore_ascii_case("must") {
            must.push(finding);
        } else {
            should.push(finding);
        }
    }

    let must_n = u32::try_from(must.len()).unwrap_or(u32::MAX);
    let should_n = u32::try_from(should.len()).unwrap_or(u32::MAX);
    let low_n = u32::try_from(low.len()).unwrap_or(u32::MAX);
    let waived_n = u32::try_from(waived.len()).unwrap_or(u32::MAX);
    let blocking = must_n > 0;

    // Render + write the report.
    let report = render_report(args, &must, &should, &low, &waived, blocking);
    let review_path = feature_dir.join("review.md");
    write_atomic(&review_path, &report)?;

    // Update the spec's `review:` frontmatter block.
    let spec_content = read_text(&spec_path)?;
    let updated =
        update_spec_review_block(&spec_content, &spec_path, args, must_n, should_n, low_n)?;
    if updated != spec_content {
        write_atomic(&spec_path, &updated)?;
    }

    Ok(WriteReviewResult {
        path: rel_path(&review_path, repo),
        spec_path: rel_path(&spec_path, repo),
        must_violations: must_n,
        should_violations: should_n,
        low_confidence: low_n,
        waived: waived_n,
        blocking,
        exit_code: i32::from(blocking),
    })
}

// -- dedup -------------------------------------------------------------------

/// Cross-pass dedup: collapse findings that share a `(rule, file)` anchor and
/// have overlapping line ranges into one, keeping the highest-severity member
/// (tie broken by higher confidence). First-seen order is preserved for a
/// stable render.
fn dedup_findings(findings: &[ReviewFinding]) -> Vec<ReviewFinding> {
    let mut kept: Vec<ReviewFinding> = Vec::new();
    for finding in findings {
        let range = parse_range(&finding.line_range);
        let overlap = kept.iter().position(|existing| {
            existing.rule == finding.rule
                && existing.file == finding.file
                && ranges_overlap(range, parse_range(&existing.line_range))
        });
        if let Some(idx) = overlap {
            if finding_rank(finding) > finding_rank(&kept[idx]) {
                kept[idx] = finding.clone();
            }
        } else {
            kept.push(finding.clone());
        }
    }
    kept
}

/// Rank a finding for dedup: severity dominates (`must` > `should`), ties break
/// on confidence (`high` > `low`).
fn finding_rank(finding: &ReviewFinding) -> u8 {
    let severity = u8::from(finding.severity.eq_ignore_ascii_case("must")) * 2;
    let confidence = u8::from(!finding.confidence.eq_ignore_ascii_case("low"));
    severity + confidence
}

/// Parse a line-range string into an inclusive `(start, end)`. An empty or
/// unparseable range covers the whole file, so it overlaps any range sharing
/// the same `(rule, file)`.
fn parse_range(text: &str) -> (u32, u32) {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return (0, u32::MAX);
    }
    if let Some((start, end)) = trimmed.split_once('-') {
        let start = start.trim().parse::<u32>().unwrap_or(0);
        let end = end.trim().parse::<u32>().unwrap_or(u32::MAX);
        (start.min(end), start.max(end))
    } else if let Ok(single) = trimmed.parse::<u32>() {
        (single, single)
    } else {
        (0, u32::MAX)
    }
}

/// Inclusive interval overlap.
fn ranges_overlap(a: (u32, u32), b: (u32, u32)) -> bool {
    a.0 <= b.1 && b.0 <= a.1
}

/// The reason of the first applied waiver anchored to this finding's
/// `(rule, file)`, or `None` when unwaived.
fn waiver_reason<'a>(
    finding: &ReviewFinding,
    applied: &'a [crate::schema::primitives::WaiverRef],
) -> Option<&'a str> {
    applied
        .iter()
        .find(|waiver| waiver.rule == finding.rule && waiver.file == finding.file)
        .map(|waiver| waiver.reason.as_str())
}

// -- report rendering --------------------------------------------------------

/// Render the full `review.md` document (frontmatter + fixed skeleton).
fn render_report(
    args: &WriteReviewArgs,
    must: &[&ReviewFinding],
    should: &[&ReviewFinding],
    low: &[&ReviewFinding],
    waived: &[&ReviewFinding],
    blocking: bool,
) -> String {
    let feature = &args.feature;

    let mut fm = String::from("---\n");
    let _ = writeln!(fm, "spec: {feature}");
    if let Some(scenario) = args.scenario.as_deref().filter(|s| !s.trim().is_empty()) {
        let _ = writeln!(fm, "scenario: {scenario}");
    }
    let _ = writeln!(fm, "reviewed-at: {}", args.reviewed_at);
    let _ = writeln!(fm, "reviewed-against: {}", args.reviewed_against);
    let _ = writeln!(fm, "diff-base: {}", args.diff_base);
    let _ = writeln!(fm, "must-violations: {}", must.len());
    let _ = writeln!(fm, "should-violations: {}", should.len());
    let _ = writeln!(fm, "low-confidence: {}", low.len());
    let _ = writeln!(fm, "captured-issues: {}", args.captured_issues.len());
    let _ = writeln!(fm, "skipped-passes: [{}]", args.skipped_passes.join(", "));
    fm.push_str("---");

    let summary = args
        .summary
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .map_or_else(
            || {
                generate_summary(
                    args,
                    must.len(),
                    should.len(),
                    low.len(),
                    waived.len(),
                    blocking,
                )
            },
            str::to_string,
        );

    let sections = [
        format!("# Review — {feature}"),
        format!("## Summary\n\n{summary}"),
        format!(
            "## MUST violations (blocking)\n\n{}",
            render_findings(must, "MUST", &args.applied_waivers)
        ),
        format!(
            "## SHOULD violations (advisory)\n\n{}",
            render_findings(should, "SHOULD", &args.applied_waivers)
        ),
        format!(
            "## Low-confidence findings\n\n{}",
            render_findings(low, "LOW-CONFIDENCE", &args.applied_waivers)
        ),
        format!(
            "## Waived findings\n\n{}",
            render_findings(waived, "WAIVED", &args.applied_waivers)
        ),
        format!(
            "## Captured issues\n\n{}",
            render_captured(&args.captured_issues)
        ),
        format!(
            "## Skipped passes\n\n{}",
            render_skipped(&args.skipped_passes)
        ),
    ];

    format!("{fm}\n\n{}\n", sections.join("\n\n"))
}

/// A deterministic one-line Summary derived from the counts.
fn generate_summary(
    args: &WriteReviewArgs,
    must: usize,
    should: usize,
    low: usize,
    waived: usize,
    blocking: bool,
) -> String {
    if args.empty_scope {
        return "Review scope is empty — no implementation files in scope. \
             Zero findings across all passes; blocking: no."
            .to_string();
    }
    let mut summary = format!(
        "{must} MUST violation(s), {should} SHOULD violation(s), {low} low-confidence finding(s)"
    );
    if waived > 0 {
        let _ = write!(summary, ", {waived} waived");
    }
    let _ = write!(
        summary,
        ". blocking: {}.",
        if blocking { "yes" } else { "no" }
    );
    summary
}

/// Render a bucket of findings, or `*None.*` when empty.
fn render_findings(
    findings: &[&ReviewFinding],
    label: &str,
    applied: &[crate::schema::primitives::WaiverRef],
) -> String {
    if findings.is_empty() {
        return "*None.*".to_string();
    }
    findings
        .iter()
        .map(|finding| finding_block(finding, label, waiver_reason(finding, applied)))
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Render one finding block per the fixed per-finding shape. Absent optional
/// fields (rule text, finding prose, suggested fix) drop their bullet so the
/// output stays markdownlint-clean.
fn finding_block(finding: &ReviewFinding, label: &str, waived_reason: Option<&str>) -> String {
    let mut out = String::new();
    let summary = finding.summary.trim();
    if summary.is_empty() {
        let _ = writeln!(out, "### {label}: {}\n", finding.rule);
    } else {
        let _ = writeln!(out, "### {label}: {} — {summary}\n", finding.rule);
    }
    let range = finding.line_range.trim();
    if range.is_empty() {
        let _ = writeln!(out, "- **File**: `{}`", finding.file);
    } else {
        let _ = writeln!(out, "- **File**: `{}:{range}`", finding.file);
    }
    if !finding.rule_text.trim().is_empty() {
        let _ = writeln!(out, "- **Rule**: {}", finding.rule_text.trim());
    }
    if !finding.finding.trim().is_empty() {
        let _ = writeln!(out, "- **Finding**: {}", finding.finding.trim());
    }
    let _ = writeln!(
        out,
        "- **Auto-fixable**: {}",
        if finding.auto_fixable { "yes" } else { "no" }
    );
    if !finding.suggested_fix.trim().is_empty() {
        let _ = writeln!(out, "- **Suggested fix**: {}", finding.suggested_fix.trim());
    }
    if let Some(reason) = waived_reason {
        let _ = writeln!(out, "- **Waived**: {reason}");
    }
    out.trim_end().to_string()
}

/// Render captured inbox issues as a list, or `*None.*` when empty. Lines that
/// already carry a `- ` bullet render verbatim; bare lines get one.
fn render_captured(issues: &[String]) -> String {
    if issues.is_empty() {
        return "*None.*".to_string();
    }
    issues
        .iter()
        .map(|line| {
            let trimmed = line.trim_end();
            if trimmed.trim_start().starts_with("- ") {
                trimmed.to_string()
            } else {
                format!("- {}", trimmed.trim_start())
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Render skipped passes as a list, or `*None.*` when empty.
fn render_skipped(skipped: &[String]) -> String {
    if skipped.is_empty() {
        return "*None.*".to_string();
    }
    skipped
        .iter()
        .map(|pass| format!("- {pass}"))
        .collect::<Vec<_>>()
        .join("\n")
}

// -- spec frontmatter update -------------------------------------------------

/// Rewrite the spec's `review:` frontmatter block with the fresh scalar fields,
/// preserving every other top-level key verbatim and pruning expired waivers
/// from `review.waivers`. Inserts the block when absent.
fn update_spec_review_block(
    content: &str,
    spec_path: &Path,
    args: &WriteReviewArgs,
    must: u32,
    should: u32,
    low: u32,
) -> Result<String> {
    let (fm_text, body) = split_frontmatter(content, spec_path)?;
    let existing: SpecReviewFm =
        serde_norway::from_str(fm_text).map_err(|source| PrimitiveError::Yaml {
            path: spec_path.into(),
            source,
        })?;
    let existing_waivers = existing.review.map(|r| r.waivers).unwrap_or_default();
    let surviving: Vec<RawWaiverFull> = existing_waivers
        .into_iter()
        .filter(|waiver| !is_expired(waiver, &args.expired_waivers))
        .collect();

    let block = render_review_yaml(args, must, should, low, &surviving);
    let new_fm = splice_review_block(fm_text, &block);
    Ok(format!("---\n{new_fm}\n---\n{body}"))
}

/// Whether a waiver's `(rule, file)` anchor is in the expired set.
fn is_expired(waiver: &RawWaiverFull, expired: &[crate::schema::primitives::WaiverRef]) -> bool {
    let (Some(rule), Some(file)) = (waiver.rule.as_deref(), waiver.file.as_deref()) else {
        return false; // malformed waivers are never pruned
    };
    expired
        .iter()
        .any(|entry| entry.rule == rule && entry.file == file)
}

/// Render the `review:` YAML block (no trailing newline).
fn render_review_yaml(
    args: &WriteReviewArgs,
    must: u32,
    should: u32,
    low: u32,
    waivers: &[RawWaiverFull],
) -> String {
    let mut block = String::from("review:\n");
    let _ = writeln!(block, "  last-run: {}", args.reviewed_at);
    let _ = writeln!(block, "  reviewed-against: {}", args.reviewed_against);
    let _ = writeln!(block, "  must-violations: {must}");
    let _ = writeln!(block, "  should-violations: {should}");
    let _ = writeln!(block, "  low-confidence: {low}");
    let _ = writeln!(block, "  blocking: {}", must > 0);
    if !waivers.is_empty() {
        block.push_str("  waivers:\n");
        for waiver in waivers {
            let fields = [
                ("rule", waiver.rule.as_deref()),
                ("file", waiver.file.as_deref()),
                ("reason", waiver.reason.as_deref()),
                ("waived-at", waiver.waived_at.as_deref()),
                ("waived-by", waiver.waived_by.as_deref()),
            ];
            let mut first = true;
            for (key, value) in fields {
                if let Some(value) = value {
                    let indent = if first { "    - " } else { "      " };
                    let _ = writeln!(block, "{indent}{key}: {}", yaml_scalar(value));
                    first = false;
                }
            }
        }
    }
    block.trim_end_matches('\n').to_string()
}

/// Replace the `review:` block region of a frontmatter body with `block`,
/// preserving surrounding top-level keys. Appends the block when no `review:`
/// key is present.
fn splice_review_block(fm_text: &str, block: &str) -> String {
    let lines: Vec<&str> = fm_text.lines().collect();
    let start = lines
        .iter()
        .position(|line| top_level_key_is(line, "review"));
    let mut out: Vec<&str> = Vec::new();
    if let Some(i) = start {
        let mut end = i + 1;
        while end < lines.len() && !is_new_top_level(lines[end]) {
            end += 1;
        }
        out.extend_from_slice(&lines[..i]);
        out.extend(block.lines());
        out.extend_from_slice(&lines[end..]);
    } else {
        out.extend_from_slice(&lines);
        out.extend(block.lines());
    }
    out.join("\n")
}

/// Whether `line` is the top-level (unindented) `{key}:` frontmatter key.
fn top_level_key_is(line: &str, key: &str) -> bool {
    !line.starts_with([' ', '\t'])
        && line
            .strip_prefix(key)
            .is_some_and(|rest| rest.starts_with(':'))
}

/// Whether `line` opens a new top-level key (non-empty, unindented).
fn is_new_top_level(line: &str) -> bool {
    !line.is_empty() && !line.starts_with([' ', '\t'])
}

/// Emit a YAML scalar, double-quoting only when a plain scalar would be
/// ambiguous (empty, surrounding whitespace, an indicator lead, a `: ` / ` #`
/// sequence, a trailing colon, or an embedded quote/newline). Simple
/// timestamps, shas, rule IDs, and paths stay unquoted.
fn yaml_scalar(value: &str) -> String {
    if needs_quote(value) {
        serde_json::to_string(value).unwrap_or_else(|_| format!("\"{value}\""))
    } else {
        value.to_string()
    }
}

/// YAML plain-scalar indicator characters: a value leading with one of these
/// must be quoted.
const YAML_INDICATORS: &[u8] = b"!&*?|>@%#{}[],\"'`:-";

fn needs_quote(value: &str) -> bool {
    if value.is_empty() || value != value.trim() {
        return true;
    }
    if value.contains(": ")
        || value.contains(" #")
        || value.contains('\n')
        || value.contains('"')
        || value.ends_with(':')
    {
        return true;
    }
    YAML_INDICATORS.contains(&value.as_bytes()[0])
}

// -- existing-frontmatter parse shapes ---------------------------------------

/// Minimal spec frontmatter shape: just the `review.waivers` list, parsed
/// loosely so a malformed waiver entry survives as reportable state.
#[derive(Deserialize)]
struct SpecReviewFm {
    #[serde(default)]
    review: Option<RawReviewBlock>,
}

#[derive(Deserialize, Default)]
struct RawReviewBlock {
    #[serde(default)]
    waivers: Vec<RawWaiverFull>,
}

/// One waiver entry with every field optional, so pruning preserves the full
/// record (`waived-at` / `waived-by` included) that `WaiverRef` drops.
#[derive(Deserialize)]
struct RawWaiverFull {
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
    use crate::schema::primitives::WaiverRef;
    use std::fs;
    use tempfile::{TempDir, tempdir};

    fn finding(
        rule: &str,
        severity: &str,
        file: &str,
        range: &str,
        confidence: &str,
    ) -> ReviewFinding {
        ReviewFinding {
            rule: rule.into(),
            severity: severity.into(),
            file: file.into(),
            line_range: range.into(),
            confidence: confidence.into(),
            summary: format!("{rule} summary"),
            finding: "Explanation of the finding.".into(),
            rule_text: "Verbatim rule text.".into(),
            auto_fixable: false,
            suggested_fix: String::new(),
        }
    }

    fn waiver(rule: &str, file: &str) -> WaiverRef {
        WaiverRef {
            rule: rule.into(),
            file: file.into(),
            reason: "Justified for now.".into(),
        }
    }

    fn base_args(feature: &str) -> WriteReviewArgs {
        WriteReviewArgs {
            feature: feature.into(),
            reviewed_at: "2026-07-02T12:00:00Z".into(),
            reviewed_against: "abc1234".into(),
            diff_base: "def5678".into(),
            scenario: None,
            empty_scope: false,
            summary: None,
            skipped_passes: Vec::new(),
            findings: Vec::new(),
            applied_waivers: Vec::new(),
            expired_waivers: Vec::new(),
            captured_issues: Vec::new(),
        }
    }

    /// Write `specs/{feature}/spec.md` with the given frontmatter body (the
    /// text between the `---` fences) and return the tempdir.
    fn spec_repo(feature: &str, frontmatter: &str) -> TempDir {
        let tmp = tempdir().unwrap();
        let dir = tmp.path().join("specs").join(feature);
        fs::create_dir_all(&dir).unwrap();
        fs::write(
            dir.join("spec.md"),
            format!("---\n{frontmatter}\n---\n\n# {feature}\n"),
        )
        .unwrap();
        tmp
    }

    fn review_md(tmp: &TempDir, feature: &str) -> String {
        fs::read_to_string(tmp.path().join("specs").join(feature).join("review.md")).unwrap()
    }

    fn spec_md(tmp: &TempDir, feature: &str) -> String {
        fs::read_to_string(tmp.path().join("specs").join(feature).join("spec.md")).unwrap()
    }

    #[test]
    fn empty_scope_report_has_zero_findings_and_is_not_blocking() {
        let tmp = spec_repo("001-x", "status: in-progress\ndependencies: []");
        let mut args = base_args("001-x");
        args.empty_scope = true;
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.must_violations, 0);
        assert_eq!(result.should_violations, 0);
        assert_eq!(result.low_confidence, 0);
        assert!(!result.blocking);
        assert_eq!(result.exit_code, 0);
        let report = review_md(&tmp, "001-x");
        assert!(report.contains("must-violations: 0"));
        assert!(report.contains("Review scope is empty"));
        assert!(report.contains("## MUST violations (blocking)\n\n*None.*"));
    }

    #[test]
    fn cross_pass_dedup_highest_severity_wins() {
        // Same (rule, file) with overlapping ranges from two passes → 1 MUST.
        let tmp = spec_repo("001-x", "status: in-progress\ndependencies: []");
        let mut args = base_args("001-x");
        args.findings = vec![
            finding("SEC-BE-001", "should", "src/a.rs", "10-20", "high"),
            finding("SEC-BE-001", "must", "src/a.rs", "15-25", "high"),
        ];
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.must_violations, 1);
        assert_eq!(result.should_violations, 0);
    }

    #[test]
    fn dedup_keeps_non_overlapping_same_rule_file() {
        let tmp = spec_repo("001-x", "status: in-progress\ndependencies: []");
        let mut args = base_args("001-x");
        args.findings = vec![
            finding("SEC-BE-001", "must", "src/a.rs", "10-20", "high"),
            finding("SEC-BE-001", "must", "src/a.rs", "50-60", "high"),
        ];
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.must_violations, 2);
    }

    #[test]
    fn blocking_true_when_must_violations_exceed_zero() {
        let tmp = spec_repo("001-x", "status: in-progress\ndependencies: []");
        let mut args = base_args("001-x");
        args.findings = vec![finding("SEC-BE-002", "must", "src/a.rs", "1-5", "high")];
        let result = run(&args, tmp.path()).unwrap();
        assert!(result.blocking);
        assert_eq!(result.exit_code, 1);
        let spec = spec_md(&tmp, "001-x");
        assert!(spec.contains("blocking: true"));
        assert!(spec.contains("must-violations: 1"));
    }

    #[test]
    fn low_confidence_finding_routed_to_low_bucket() {
        // A low-confidence MUST-severity finding counts as low-confidence, not
        // a MUST violation → not blocking.
        let tmp = spec_repo("001-x", "status: in-progress\ndependencies: []");
        let mut args = base_args("001-x");
        args.findings = vec![finding("SIM-001", "must", "src/a.rs", "1-5", "low")];
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.must_violations, 0);
        assert_eq!(result.low_confidence, 1);
        assert!(!result.blocking);
        assert!(review_md(&tmp, "001-x").contains("### LOW-CONFIDENCE: SIM-001"));
    }

    #[test]
    fn single_findings_array_ingestion_buckets_by_section() {
        let tmp = spec_repo("001-x", "status: in-progress\ndependencies: []");
        let mut args = base_args("001-x");
        args.findings = vec![
            finding("SEC-BE-001", "must", "src/a.rs", "1-5", "high"),
            finding("QUAL-002", "should", "src/b.rs", "1-5", "high"),
            finding("SIM-003", "should", "src/c.rs", "1-5", "low"),
        ];
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.must_violations, 1);
        assert_eq!(result.should_violations, 1);
        assert_eq!(result.low_confidence, 1);
        let report = review_md(&tmp, "001-x");
        assert!(report.contains("### MUST: SEC-BE-001"));
        assert!(report.contains("### SHOULD: QUAL-002"));
        assert!(report.contains("### LOW-CONFIDENCE: SIM-003"));
    }

    #[test]
    fn applied_waiver_excludes_finding_from_must_count() {
        let tmp = spec_repo("001-x", "status: in-progress\ndependencies: []");
        let mut args = base_args("001-x");
        args.findings = vec![finding(
            "SEC-BE-014",
            "must",
            "src/internal.ts",
            "1-5",
            "high",
        )];
        args.applied_waivers = vec![waiver("SEC-BE-014", "src/internal.ts")];
        let result = run(&args, tmp.path()).unwrap();
        assert_eq!(result.must_violations, 0);
        assert_eq!(result.waived, 1);
        assert!(!result.blocking);
        let report = review_md(&tmp, "001-x");
        assert!(report.contains("### WAIVED: SEC-BE-014"));
        assert!(report.contains("- **Waived**: Justified for now."));
    }

    #[test]
    fn skipped_passes_recorded_in_frontmatter_and_section() {
        let tmp = spec_repo("001-x", "status: in-progress\ndependencies: []");
        let mut args = base_args("001-x");
        args.skipped_passes = vec!["security".into(), "simplicity".into()];
        run(&args, tmp.path()).unwrap();
        let report = review_md(&tmp, "001-x");
        assert!(report.contains("skipped-passes: [security, simplicity]"));
        assert!(report.contains("## Skipped passes\n\n- security\n- simplicity"));
    }

    #[test]
    fn captured_issues_rendered_and_counted() {
        let tmp = spec_repo("001-x", "status: in-progress\ndependencies: []");
        let mut args = base_args("001-x");
        args.captured_issues = vec!["- leak in a.rs".into(), "missing check in b.rs".into()];
        run(&args, tmp.path()).unwrap();
        let report = review_md(&tmp, "001-x");
        assert!(report.contains("captured-issues: 2"));
        assert!(report.contains("- leak in a.rs"));
        assert!(report.contains("- missing check in b.rs"));
    }

    #[test]
    fn frontmatter_review_block_inserted_when_absent() {
        let tmp = spec_repo("001-x", "status: in-progress\ndependencies: []");
        let mut args = base_args("001-x");
        args.findings = vec![
            finding("A-1", "must", "src/a.rs", "1-2", "high"),
            finding("B-2", "should", "src/b.rs", "1-2", "high"),
        ];
        run(&args, tmp.path()).unwrap();
        let spec = spec_md(&tmp, "001-x");
        assert!(spec.contains("review:"));
        assert!(spec.contains("last-run: 2026-07-02T12:00:00Z"));
        assert!(spec.contains("reviewed-against: abc1234"));
        assert!(spec.contains("must-violations: 1"));
        assert!(spec.contains("should-violations: 1"));
        assert!(spec.contains("low-confidence: 0"));
        assert!(spec.contains("blocking: true"));
        // Untouched keys preserved.
        assert!(spec.contains("status: in-progress"));
        assert!(spec.contains("dependencies: []"));
    }

    #[test]
    fn frontmatter_review_block_replaced_when_present() {
        let tmp = spec_repo(
            "001-x",
            "status: in-progress\ndependencies: []\nreview:\n  last-run: 2020-01-01T00:00:00Z\n  must-violations: 9\n  blocking: true",
        );
        let args = base_args("001-x");
        run(&args, tmp.path()).unwrap();
        let spec = spec_md(&tmp, "001-x");
        assert!(spec.contains("must-violations: 0"));
        assert!(spec.contains("blocking: false"));
        assert!(!spec.contains("must-violations: 9"));
        assert!(!spec.contains("2020-01-01"));
        // The frontmatter still parses and keeps sibling keys.
        assert!(spec.contains("status: in-progress"));
        let (fm, _) = split_frontmatter(&spec, Path::new("spec.md")).unwrap();
        let parsed: SpecReviewFm = serde_norway::from_str(fm).unwrap();
        assert!(parsed.review.is_some());
    }

    #[test]
    fn expired_waiver_pruned_from_spec_frontmatter() {
        let frontmatter = "status: in-progress\ndependencies: []\nreview:\n  last-run: 2026-01-01T00:00:00Z\n  must-violations: 0\n  blocking: false\n  waivers:\n    - rule: SEC-BE-014\n      file: src/gone.ts\n      reason: No longer relevant.\n      waived-at: 2026-01-01T00:00:00Z\n      waived-by: dev@example.com\n    - rule: SEC-BE-020\n      file: src/keep.ts\n      reason: Still valid.\n      waived-at: 2026-01-02T00:00:00Z\n      waived-by: dev@example.com";
        let tmp = spec_repo("001-x", frontmatter);
        let mut args = base_args("001-x");
        args.expired_waivers = vec![waiver("SEC-BE-014", "src/gone.ts")];
        run(&args, tmp.path()).unwrap();
        let spec = spec_md(&tmp, "001-x");
        // Expired anchor gone; surviving waiver kept with all its fields.
        assert!(!spec.contains("src/gone.ts"));
        assert!(spec.contains("rule: SEC-BE-020"));
        assert!(spec.contains("file: src/keep.ts"));
        assert!(spec.contains("waived-by: dev@example.com"));
        assert!(spec.contains("Still valid."));
        // The rewritten block still parses.
        let (fm, _) = split_frontmatter(&spec, Path::new("spec.md")).unwrap();
        let parsed: SpecReviewFm = serde_norway::from_str(fm).unwrap();
        assert_eq!(parsed.review.unwrap().waivers.len(), 1);
    }

    #[test]
    fn idempotent_rerun_reproduces_identical_review_md() {
        let tmp = spec_repo("001-x", "status: in-progress\ndependencies: []");
        let mut args = base_args("001-x");
        args.findings = vec![finding("SEC-BE-001", "must", "src/a.rs", "1-5", "high")];
        run(&args, tmp.path()).unwrap();
        let first = review_md(&tmp, "001-x");
        run(&args, tmp.path()).unwrap();
        let second = review_md(&tmp, "001-x");
        assert_eq!(first, second);
    }

    #[test]
    fn missing_feature_is_operational_error() {
        let tmp = tempdir().unwrap();
        let err = run(&base_args("999-nope"), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::FeatureNotFound { .. }));
    }

    #[test]
    fn dropping_named_tempfile_leaves_no_review_md() {
        use std::io::Write;
        let tmp = spec_repo("001-x", "status: in-progress\ndependencies: []");
        let dir = tmp.path().join("specs/001-x");
        let dest = dir.join("review.md");
        {
            let mut tf = tempfile::NamedTempFile::new_in(&dir).unwrap();
            tf.write_all(b"INTERRUPTED").unwrap();
        }
        assert!(!dest.exists());
    }

    #[test]
    fn yaml_scalar_quotes_only_when_needed() {
        assert_eq!(yaml_scalar("2026-07-02T12:00:00Z"), "2026-07-02T12:00:00Z");
        assert_eq!(yaml_scalar("SEC-BE-014"), "SEC-BE-014");
        assert_eq!(yaml_scalar("src/api/internal.ts"), "src/api/internal.ts");
        assert_eq!(
            yaml_scalar("Endpoint is internal-only."),
            "Endpoint is internal-only."
        );
        assert_eq!(yaml_scalar("see: this"), "\"see: this\"");
        assert_eq!(yaml_scalar(""), "\"\"");
    }
}
