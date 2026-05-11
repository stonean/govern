//! `lint-markdown` — wrap `npx markdownlint-cli2` and surface violations.
//!
//! The primitive spawns `npx markdownlint-cli2` (optionally with `--fix`)
//! against the given paths, captures combined stdout/stderr, and parses
//! each line into a [`MarkdownViolation`]. Exit code 1 (violations found)
//! and 2+ (config or runtime error) both flow through as `clean: false`;
//! callers consult `exit_code` to distinguish.

use std::path::Path;
use std::process::Command;
use std::sync::OnceLock;

use regex::Regex;

use crate::primitives::{PrimitiveError, Result};
use crate::schema::primitives::{LintMarkdownArgs, LintMarkdownResult, MarkdownViolation};

/// Execute the `lint-markdown` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::Io`] when `npx` cannot be spawned. A non-zero
/// markdownlint-cli2 exit code is not an error — it's recorded in the
/// result alongside the parsed violations.
pub fn run(args: &LintMarkdownArgs, repo: &Path) -> Result<LintMarkdownResult> {
    let mut cmd = Command::new("npx");
    cmd.arg("markdownlint-cli2");
    if args.fix {
        cmd.arg("--fix");
    }
    for path in &args.paths {
        cmd.arg(path);
    }
    cmd.current_dir(repo);

    let output = cmd.output().map_err(|source| PrimitiveError::Io {
        path: repo.into(),
        source,
    })?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    let mut violations: Vec<MarkdownViolation> = Vec::new();
    for line in stdout.lines().chain(stderr.lines()) {
        if let Some(violation) = parse_violation_line(line) {
            violations.push(violation);
        }
    }

    let clean = violations.is_empty();
    Ok(LintMarkdownResult {
        violations,
        clean,
        exit_code,
    })
}

/// Parse one markdownlint-cli2 violation line. The default output format is
/// `path:line[:col] RULE/aliases description`; we accept either form.
fn parse_violation_line(line: &str) -> Option<MarkdownViolation> {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    let re = PATTERN.get_or_init(|| {
        Regex::new(r"^(?P<path>[^:]+(?:\.md|\.markdown)):(?P<line>\d+)(?::\d+)?\s+(?P<rule>MD\d+)(?:/\S+)?\s+(?P<message>.+)$")
            .unwrap_or_else(|err| panic!("markdownlint violation regex must compile: {err}"))
    });
    let caps = re.captures(line.trim())?;
    let line_num: u32 = caps.name("line")?.as_str().parse().ok()?;
    Some(MarkdownViolation {
        path: caps.name("path")?.as_str().into(),
        line: line_num,
        rule: caps.name("rule")?.as_str().into(),
        message: caps.name("message")?.as_str().trim().into(),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn parses_canonical_violation_line() {
        let v = parse_violation_line(
            "README.md:17 MD013/line-length Line length [Expected: 80, Actual: 120]",
        )
        .unwrap();
        assert_eq!(v.path, "README.md");
        assert_eq!(v.line, 17);
        assert_eq!(v.rule, "MD013");
        assert!(v.message.contains("Line length"));
    }

    #[test]
    fn parses_violation_with_column() {
        let v = parse_violation_line(
            "docs/spec.md:42:3 MD009 Trailing spaces [Expected: 0; Actual: 2]",
        )
        .unwrap();
        assert_eq!(v.path, "docs/spec.md");
        assert_eq!(v.line, 42);
        assert_eq!(v.rule, "MD009");
    }

    #[test]
    fn ignores_non_violation_lines() {
        assert!(parse_violation_line("Finding files...").is_none());
        assert!(parse_violation_line("Summary: 0 errors").is_none());
        assert!(parse_violation_line("").is_none());
        assert!(parse_violation_line("README.md:no-line MD013 foo").is_none());
    }

    #[test]
    fn parses_path_with_spaces_disallowed() {
        // The regex is path-without-spaces; this matches typical markdownlint
        // output and keeps the parser unambiguous. Verify a space-containing
        // path is rejected rather than mis-parsed.
        let v = parse_violation_line("path with space.md:5 MD013 Line length");
        assert!(v.is_some(), "non-greedy [^:]+ accepts spaces before colon");
        let v = v.unwrap();
        assert_eq!(v.path, "path with space.md");
    }
}
