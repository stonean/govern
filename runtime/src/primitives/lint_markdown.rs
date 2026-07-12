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
    // Windows ships npx as a `.cmd` shim, which `Command::new("npx")`
    // cannot resolve (CreateProcess needs the explicit extension).
    // A path beginning with `-` would be parsed by markdownlint-cli2 as an
    // option, not a file — `--config=evil.json` can load a `customRules` JS
    // module, i.e. arbitrary code under this primitive's permission. Reject
    // it so `paths` names files only.
    for path in &args.paths {
        if path.starts_with('-') {
            return Err(PrimitiveError::InvalidArgument {
                primitive: "lint-markdown".into(),
                argument: "paths".into(),
                reason: "a path beginning with '-' would be parsed as a markdownlint-cli2 \
                         flag (e.g. --config loads arbitrary JS); pass file paths only"
                    .into(),
            });
        }
    }
    let npx = if cfg!(windows) { "npx.cmd" } else { "npx" };
    let mut cmd = Command::new(npx);
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

    // `clean` requires both no parsed violations AND a zero exit code. A
    // non-zero exit with an empty violations vec means markdownlint reported a
    // problem in a line shape the parser did not recognize, or a config/runtime
    // error — neither is clean. Deriving solely from `violations.is_empty()`
    // silently passed such runs. Mirrors run-generator's exit-code-derived
    // `drift`.
    let clean = violations.is_empty() && exit_code == 0;
    Ok(LintMarkdownResult {
        violations,
        clean,
        exit_code,
    })
}

/// Parse one markdownlint-cli2 violation line. Output shape is
/// `path:line[:col] [severity] RULE/aliases description`. The optional
/// `severity` token (`error`/`warning`) is emitted by markdownlint-cli2
/// v0.22.1+; older output omits it. Both forms are accepted.
fn parse_violation_line(line: &str) -> Option<MarkdownViolation> {
    static PATTERN: OnceLock<Regex> = OnceLock::new();
    let re = PATTERN.get_or_init(|| {
        Regex::new(r"^(?P<path>[^:]+(?:\.md|\.markdown)):(?P<line>\d+)(?::\d+)?\s+(?:(?:error|warning)\s+)?(?P<rule>MD\d+)(?:/\S+)?\s+(?P<message>.+)$")
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
    fn parses_violation_with_severity_token() {
        // markdownlint-cli2 v0.22.1+ inserts an `error`/`warning` severity
        // token between the location and the rule. This is the exact shape that
        // was silently dropped before the regex accepted the optional token.
        let v = parse_violation_line(
            "specs/028-multi-format-agents/spec.md:34 error MD028/no-blanks-blockquote Blank line inside blockquote",
        )
        .unwrap();
        assert_eq!(v.path, "specs/028-multi-format-agents/spec.md");
        assert_eq!(v.line, 34);
        assert_eq!(v.rule, "MD028");
        assert!(v.message.contains("Blank line inside blockquote"));
    }

    #[test]
    fn parses_violation_with_severity_and_column() {
        let v = parse_violation_line(
            "docs/spec.md:42:3 warning MD009/no-trailing-spaces Trailing spaces",
        )
        .unwrap();
        assert_eq!(v.path, "docs/spec.md");
        assert_eq!(v.line, 42);
        assert_eq!(v.rule, "MD009");
        assert!(v.message.contains("Trailing spaces"));
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
