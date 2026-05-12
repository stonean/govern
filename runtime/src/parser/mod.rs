//! Procedure parser for slash command Instructions sections.
//!
//! Walks a `pulldown-cmark` event stream and produces a typed
//! [`Procedure`] (see [`crate::schema::procedure`]). The recognizer is a
//! small state machine driven by these conventions:
//!
//! - The procedure body lives in the `## Instructions` section (any
//!   heading level whose text equals `Instructions`).
//! - Numbered list items are procedure steps; nested ordered lists become
//!   sub-step numbers (`1.1`, `1.2`).
//! - A backtick-quoted code span whose text matches a known primitive name
//!   (`PRIMITIVE_NAMES`) marks the step as `Step::Primitive`.
//! - An HTML comment of the form `<!-- llm:<identifier> -->` marks the
//!   step as `Step::Extension`. Extension wins if both markers appear in
//!   the same step.
//! - A step that contains neither becomes `Step::Prose`.
//!
//! A file with no Instructions section, or one whose Instructions section
//! contains no recognized primitives or extension markers, is treated as
//! legacy prose: the parser returns [`ParseError::LegacyProse`]. A code
//! span that looks like a primitive name (kebab-case verb-noun) but does
//! not match the known set raises [`ParseError::Invalid`].

#![allow(clippy::module_name_repetitions)]

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::schema::procedure::{Procedure, SourceRange, Step, StepNumber};

/// Closed set of primitive names recognized by the parser. Mirrors the
/// MCP tool list without the `gov-rt:` prefix.
pub const PRIMITIVE_NAMES: &[&str] = &[
    "read-spec",
    "read-tasks",
    "mark-task",
    "mark-criterion",
    "set-status",
    "derive-boundary",
    "check-stuck",
    "validate-frontmatter",
    "resolve-anchor",
    "traverse-deps",
    "check-rule-ids",
    "run-generator",
    "lint-markdown",
    "gate-confirm",
    "fetch-archive",
    "extract-archive",
    "substitute-templates",
    "merge-claude-md",
];

/// Parse errors raised by [`parse`] and [`check`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// The file does not declare a parseable Instructions section, or its
    /// Instructions section contains no recognized primitive or extension
    /// markers. The markdown-only path may still walk the prose.
    LegacyProse,
    /// The file attempted to declare structure but the structure is
    /// malformed (e.g., a backticked code span looks like a primitive name
    /// but doesn't match the known set).
    Invalid {
        /// Human-readable description of the problem.
        message: String,
        /// Source location of the offending token, when known.
        location: Option<SourceRange>,
    },
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::LegacyProse => write!(
                f,
                "file is in legacy prose format (no parseable Instructions section)"
            ),
            ParseError::Invalid { message, location } => {
                if let Some(loc) = location {
                    write!(
                        f,
                        "invalid procedure: {message} at line {}:{}",
                        loc.start_line, loc.start_col
                    )
                } else {
                    write!(f, "invalid procedure: {message}")
                }
            }
        }
    }
}

impl std::error::Error for ParseError {}

/// Parse the slash command at `command_name` from the markdown `source`.
///
/// `command_name` is forwarded into [`Procedure::command`] verbatim — the
/// parser does not derive it from the file name.
///
/// # Errors
///
/// See [`ParseError`].
pub fn parse(source: &str, command_name: &str) -> Result<Procedure, ParseError> {
    let mut walker = Walker::new(source);
    walker.drive(source)?;
    Ok(Procedure {
        command: command_name.into(),
        steps: walker.steps,
    })
}

/// Lightweight wrapper around [`parse`] that discards the AST. Used by
/// `runtime parse --check` and `scripts/lint-procedure-parseability.sh`.
///
/// # Errors
///
/// See [`ParseError`].
pub fn check(source: &str) -> Result<(), ParseError> {
    parse(source, "").map(|_| ())
}

struct Walker {
    line_starts: Vec<usize>,
    source_len: usize,
    state: State,
    list_depth: u32,
    top_index: u32,
    sub_index: u32,
    current_step: Option<StepBuilder>,
    steps: Vec<Step>,
    found_any: bool,
    suspicious_spans: Vec<(String, SourceRange)>,
}

#[derive(Debug, PartialEq, Eq)]
enum State {
    BeforeInstructions,
    InInstructionsHeading,
    InInstructions,
    Done,
}

struct StepBuilder {
    number: StepNumber,
    range: SourceRange,
    prose: String,
    primitive_name: Option<String>,
    extension_id: Option<String>,
}

impl Walker {
    fn new(source: &str) -> Self {
        let mut line_starts = vec![0];
        for (idx, b) in source.bytes().enumerate() {
            if b == b'\n' {
                line_starts.push(idx + 1);
            }
        }
        Self {
            line_starts,
            source_len: source.len(),
            state: State::BeforeInstructions,
            list_depth: 0,
            top_index: 0,
            sub_index: 0,
            current_step: None,
            steps: Vec::new(),
            found_any: false,
            suspicious_spans: Vec::new(),
        }
    }

    fn drive(&mut self, source: &str) -> Result<(), ParseError> {
        let mut opts = Options::empty();
        opts.insert(Options::ENABLE_TABLES);
        let parser = Parser::new_ext(source, opts);
        for (event, range) in parser.into_offset_iter() {
            self.handle(&event, range);
        }
        if let Some(step) = self.current_step.take() {
            self.finalize_step(step);
        }
        if !self.found_any {
            // No primitives or extensions: treat the file as legacy prose,
            // regardless of any suspicious-looking code spans (the prose may
            // legitimately reference field names like `spec-ref`).
            return Err(ParseError::LegacyProse);
        }
        // New-format file: any suspicious-looking primitive name that isn't
        // in the known set is a typo and must be fixed.
        if let Some((span, range)) = self.suspicious_spans.first() {
            return Err(ParseError::Invalid {
                message: format!(
                    "code span `{span}` looks like a primitive name but is not in the known set"
                ),
                location: Some(*range),
            });
        }
        Ok(())
    }

    fn handle(&mut self, event: &Event<'_>, range: std::ops::Range<usize>) {
        match (&self.state, event) {
            (State::BeforeInstructions, Event::Start(Tag::Heading { .. })) => {
                self.state = State::InInstructionsHeading;
            }
            (State::InInstructionsHeading, Event::Text(text)) => {
                if text.as_ref() == "Instructions" {
                    // Stay in heading state until End fires; then enter list mode.
                } else {
                    self.state = State::BeforeInstructions;
                }
            }
            (State::InInstructionsHeading, Event::End(TagEnd::Heading(_))) => {
                self.state = State::InInstructions;
            }
            (State::InInstructions, Event::Start(Tag::Heading { level, .. }))
                if *level <= HeadingLevel::H2
                    || (matches!(level, HeadingLevel::H3) && self.list_depth == 0) =>
            {
                // A sibling heading closes the Instructions section.
                if let Some(step) = self.current_step.take() {
                    self.finalize_step(step);
                }
                self.state = State::Done;
            }
            (State::InInstructions, Event::Start(Tag::List(Some(_)))) => {
                self.list_depth += 1;
                if self.list_depth == 1 {
                    self.top_index = 0;
                }
            }
            (State::InInstructions, Event::End(TagEnd::List(true))) => {
                self.list_depth = self.list_depth.saturating_sub(1);
            }
            (State::InInstructions, Event::Start(Tag::Item)) => {
                if let Some(step) = self.current_step.take() {
                    self.finalize_step(step);
                }
                if self.list_depth == 1 {
                    self.top_index += 1;
                    self.sub_index = 0;
                    self.current_step = Some(StepBuilder::new(
                        StepNumber(vec![self.top_index]),
                        self.byte_range_to_source_range(&range),
                    ));
                } else if self.list_depth >= 2 {
                    self.sub_index += 1;
                    self.current_step = Some(StepBuilder::new(
                        StepNumber(vec![self.top_index, self.sub_index]),
                        self.byte_range_to_source_range(&range),
                    ));
                }
            }
            (State::InInstructions, Event::End(TagEnd::Item)) => {
                if let Some(step) = self.current_step.take() {
                    self.finalize_step(step);
                }
            }
            (State::InInstructions, Event::Code(code)) => {
                let source_range = self.byte_range_to_source_range(&range);
                if let Some(step) = self.current_step.as_mut() {
                    if PRIMITIVE_NAMES.contains(&code.as_ref()) {
                        step.primitive_name = Some(code.as_ref().into());
                    } else if looks_like_primitive(code.as_ref()) {
                        self.suspicious_spans
                            .push((code.as_ref().into(), source_range));
                    }
                    step.prose.push('`');
                    step.prose.push_str(code.as_ref());
                    step.prose.push('`');
                }
            }
            (State::InInstructions, Event::Html(html) | Event::InlineHtml(html)) => {
                if let Some(step) = self.current_step.as_mut() {
                    if let Some(id) = parse_extension_marker(html.as_ref()) {
                        step.extension_id = Some(id);
                    }
                    step.prose.push_str(html.as_ref());
                }
            }
            (State::InInstructions, Event::Text(text)) => {
                if let Some(step) = self.current_step.as_mut() {
                    step.prose.push_str(text.as_ref());
                }
            }
            (State::InInstructions, Event::SoftBreak | Event::HardBreak) => {
                if let Some(step) = self.current_step.as_mut() {
                    step.prose.push(' ');
                }
            }
            _ => {}
        }
    }

    fn finalize_step(&mut self, builder: StepBuilder) {
        let StepBuilder {
            number,
            range,
            prose,
            primitive_name,
            mut extension_id,
        } = builder;
        let prose = prose.trim().to_string();
        // pulldown-cmark sometimes emits HTML comments inside paragraphs
        // as raw text rather than InlineHtml events. Scan the assembled
        // prose for `<!-- llm:<id> -->` markers as a fallback.
        if extension_id.is_none() {
            extension_id = find_inline_extension_marker(&prose);
        }
        let step = if let Some(id) = extension_id {
            self.found_any = true;
            Step::Extension {
                number,
                identifier: id,
                prose,
                location: range,
            }
        } else if let Some(name) = primitive_name {
            self.found_any = true;
            Step::Primitive {
                number,
                name,
                prose,
                location: range,
            }
        } else {
            Step::Prose {
                number,
                text: prose,
                location: range,
            }
        };
        self.steps.push(step);
    }

    fn byte_range_to_source_range(&self, range: &std::ops::Range<usize>) -> SourceRange {
        let (start_line, start_col) = self.offset_to_line_col(range.start);
        let end_offset = range.end.min(self.source_len);
        let (end_line, end_col) = self.offset_to_line_col(end_offset);
        SourceRange {
            start_line,
            start_col,
            end_line,
            end_col,
        }
    }

    fn offset_to_line_col(&self, offset: usize) -> (u32, u32) {
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(idx) => idx,
            Err(idx) => idx.saturating_sub(1),
        };
        let line_start = self.line_starts[line_idx];
        let col = offset.saturating_sub(line_start);
        (
            u32::try_from(line_idx)
                .unwrap_or(u32::MAX)
                .saturating_add(1),
            u32::try_from(col).unwrap_or(u32::MAX).saturating_add(1),
        )
    }
}

fn looks_like_primitive(text: &str) -> bool {
    // kebab-case verb-noun: lowercase letters with at least one `-`, no
    // whitespace, no leading/trailing hyphens.
    if text.is_empty() || text.contains(char::is_whitespace) {
        return false;
    }
    if !text.contains('-') {
        return false;
    }
    if text.starts_with('-') || text.ends_with('-') {
        return false;
    }
    text.chars().all(|c| c.is_ascii_lowercase() || c == '-')
}

fn find_inline_extension_marker(prose: &str) -> Option<String> {
    let start = prose.find("<!--")?;
    let after = &prose[start..];
    let end = after.find("-->")?;
    parse_extension_marker(&after[..end + 3])
}

fn parse_extension_marker(html: &str) -> Option<String> {
    let trimmed = html.trim();
    let inner = trimmed.strip_prefix("<!--")?.strip_suffix("-->")?;
    let inner = inner.trim();
    let identifier = inner.strip_prefix("llm:")?.trim();
    if identifier.is_empty() {
        return None;
    }
    Some(identifier.to_string())
}

impl StepBuilder {
    fn new(number: StepNumber, range: SourceRange) -> Self {
        Self {
            number,
            range,
            prose: String::new(),
            primitive_name: None,
            extension_id: None,
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::schema::procedure::Step;

    fn parse_str(source: &str) -> Result<Procedure, ParseError> {
        parse(source, "test")
    }

    #[test]
    fn empty_file_is_legacy_prose() {
        let err = parse_str("").unwrap_err();
        assert_eq!(err, ParseError::LegacyProse);
    }

    #[test]
    fn file_without_instructions_section_is_legacy_prose() {
        let source = "# Header\n\n## Purpose\n\n1. Do a thing.\n2. Do another.\n";
        let err = parse_str(source).unwrap_err();
        assert_eq!(err, ParseError::LegacyProse);
    }

    #[test]
    fn instructions_with_only_prose_is_legacy_prose() {
        let source = "# Header\n\n## Instructions\n\n1. Read the spec.\n2. Walk the tasks.\n";
        let err = parse_str(source).unwrap_err();
        assert_eq!(err, ParseError::LegacyProse);
    }

    #[test]
    fn well_formed_procedure_parses() {
        let source = "# Cmd\n\n## Instructions\n\n1. Invoke `read-spec` on the target.\n2. Invoke `read-tasks` to load tasks.\n3. <!-- llm:writeCode --> Write the code.\n";
        let procedure = parse_str(source).unwrap();
        assert_eq!(procedure.steps.len(), 3);
        match &procedure.steps[0] {
            Step::Primitive { number, name, .. } => {
                assert_eq!(number.0, vec![1]);
                assert_eq!(name, "read-spec");
            }
            other => panic!("expected primitive, got {other:?}"),
        }
        match &procedure.steps[2] {
            Step::Extension {
                number, identifier, ..
            } => {
                assert_eq!(number.0, vec![3]);
                assert_eq!(identifier, "writeCode");
            }
            other => panic!("expected extension, got {other:?}"),
        }
    }

    #[test]
    fn unknown_primitive_backtick_in_new_format_file_is_invalid() {
        // A typo (`read-spek`) inside a file that otherwise uses the new
        // conventions (has at least one valid primitive `read-tasks`) is
        // flagged as Invalid. In a legacy file with no valid primitives, the
        // same span would be treated as prose (covered by
        // `code_span_without_valid_primitive_is_legacy_prose`).
        let source = "# Cmd\n\n## Instructions\n\n1. Call `read-spek` for the target.\n2. Then `read-tasks` to load tasks.\n";
        let err = parse_str(source).unwrap_err();
        match err {
            ParseError::Invalid { message, .. } => {
                assert!(
                    message.contains("read-spek"),
                    "message mentions the offending span"
                );
            }
            ParseError::LegacyProse => panic!("expected Invalid, got LegacyProse"),
        }
    }

    #[test]
    fn code_span_without_valid_primitive_is_legacy_prose() {
        // The prose references a kebab-case identifier (`spec-ref`) that
        // happens to look like a primitive name but isn't. In a legacy
        // file (no valid primitives anywhere), the parser treats this as
        // legacy prose rather than flagging it.
        let source = "# Cmd\n\n## Instructions\n\n1. Read the `spec-ref` field from frontmatter.\n";
        let err = parse_str(source).unwrap_err();
        assert_eq!(err, ParseError::LegacyProse);
    }

    #[test]
    fn extension_marker_wins_over_primitive_on_same_step() {
        let source = "# Cmd\n\n## Instructions\n\n1. Call `read-spec` then <!-- llm:writeCode --> write code.\n";
        let procedure = parse_str(source).unwrap();
        assert_eq!(procedure.steps.len(), 1);
        match &procedure.steps[0] {
            Step::Extension { identifier, .. } => {
                assert_eq!(identifier, "writeCode");
            }
            other => panic!("expected extension, got {other:?}"),
        }
    }

    #[test]
    fn nested_list_produces_subnumbered_steps() {
        let source = "# Cmd\n\n## Instructions\n\n1. Top-level work.\n   1. Sub-step calling `read-spec`.\n   2. Another sub-step.\n2. Final step.\n";
        let procedure = parse_str(source).unwrap();
        let numbers: Vec<&[u32]> = procedure
            .steps
            .iter()
            .map(|s| match s {
                Step::Primitive { number, .. }
                | Step::Extension { number, .. }
                | Step::Prose { number, .. } => number.0.as_slice(),
            })
            .collect();
        assert!(
            numbers.iter().any(|n| n == &[1u32, 1u32].as_slice()),
            "expected a 1.1 sub-step, got {numbers:?}"
        );
    }

    #[test]
    fn instructions_followed_by_sibling_heading_closes_section() {
        let source = "# Cmd\n\n## Instructions\n\n1. `read-spec` here.\n\n## Done\n\n1. `mark-task` should be ignored — outside Instructions.\n";
        let procedure = parse_str(source).unwrap();
        assert_eq!(procedure.steps.len(), 1);
        match &procedure.steps[0] {
            Step::Primitive { name, .. } => assert_eq!(name, "read-spec"),
            other => panic!("expected primitive, got {other:?}"),
        }
    }

    #[test]
    fn extension_marker_recognizes_identifier_inside_html_comment() {
        assert_eq!(
            parse_extension_marker("<!-- llm:writeCode -->"),
            Some("writeCode".into())
        );
        assert_eq!(
            parse_extension_marker("<!--llm:writeSpecBody-->"),
            Some("writeSpecBody".into())
        );
        assert_eq!(parse_extension_marker("<!-- comment -->"), None);
        assert_eq!(parse_extension_marker("not a comment"), None);
    }

    #[test]
    fn looks_like_primitive_excludes_obvious_non_matches() {
        assert!(looks_like_primitive("foo-bar"));
        assert!(looks_like_primitive("read-spec"));
        assert!(!looks_like_primitive("foo"));
        assert!(!looks_like_primitive("Foo-Bar"));
        assert!(!looks_like_primitive(""));
        assert!(!looks_like_primitive("-foo"));
        assert!(!looks_like_primitive("foo with space"));
    }
}
