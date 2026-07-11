//! Procedure parser for slash command Instructions sections.
//!
//! Walks a `pulldown-cmark` event stream and produces a typed
//! [`Procedure`] (see [`crate::schema::procedure`]). The recognizer is a
//! small state machine driven by these conventions:
//!
//! - The procedure body lives in the `## Instructions` section (any
//!   heading level whose text equals `Instructions`).
//! - Numbered list items are procedure steps; nested ordered lists become
//!   sub-step numbers (`1.1`, `1.2`). Nested *unordered* lists are not
//!   steps: their bullets fold into the enclosing step's prose.
//! - Step numbers honor the document: an ordered list's `start` value
//!   seeds the step counter, and consecutive ordered lists separated only
//!   by HTML comments and blank lines (the `<!-- audit:ignore-promotion -->`
//!   shape) continue the previous numbering rather than restarting. A
//!   list separated from the previous one by real prose re-seeds from its
//!   literal `start` value.
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
//! span raises [`ParseError::Invalid`] only when it is plausibly a typo'd
//! primitive invocation: it sits in primitive-invoking position (the step
//! prose immediately before it ends with the word "invoke" or "call") and
//! looks like a primitive name (kebab-case verb-noun), or it is within
//! edit distance 2 of a known primitive name. Ordinary kebab-case
//! vocabulary (`no-checkbox`, `keep-pending`, `cli-config-dir`) parses as
//! prose.

#![allow(clippy::module_name_repetitions)]

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::schema::procedure::{Procedure, SourceRange, Step, StepNumber};

/// Closed set of primitive names recognized by the parser — defined from
/// the canonical registry (`crate::schema::registry::PRIMITIVE_REGISTRY`),
/// so it is identical to the MCP tool list in
/// [`crate::mcp::server::TOOL_NAMES`] by construction.
pub const PRIMITIVE_NAMES: &[&str] = crate::schema::registry::PRIMITIVE_REGISTRY;

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
    /// Stack of currently open lists inside Instructions. Both ordered
    /// and unordered lists are pushed so that bullets nested in a
    /// numbered step never masquerade as top-level steps.
    list_stack: Vec<ListKind>,
    /// Byte offset just past the last content of the most recent
    /// top-level ordered list. Used to decide whether the next ordered
    /// list continues the same step sequence (separated only by HTML
    /// comments / blank lines) or starts fresh.
    last_top_list_end: Option<usize>,
    top_index: u32,
    sub_index: u32,
    current_step: Option<StepBuilder>,
    steps: Vec<Step>,
    found_any: bool,
    suspicious_spans: Vec<(String, SourceRange)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ListKind {
    Ordered,
    Unordered,
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
            list_stack: Vec::new(),
            last_top_list_end: None,
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
            self.handle(source, &event, range);
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

    fn handle(&mut self, source: &str, event: &Event<'_>, range: std::ops::Range<usize>) {
        // While inside a top-level ordered list, keep the high-water byte
        // offset of its content current so the next list's continuation
        // check inspects exactly the gap between the two lists.
        if self.state == State::InInstructions
            && self.list_stack.first() == Some(&ListKind::Ordered)
        {
            let end = range.end.min(self.source_len);
            self.last_top_list_end = Some(self.last_top_list_end.map_or(end, |e| e.max(end)));
        }
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
                    || (matches!(level, HeadingLevel::H3) && self.list_stack.is_empty()) =>
            {
                // A sibling heading closes the Instructions section.
                if let Some(step) = self.current_step.take() {
                    self.finalize_step(step);
                }
                self.state = State::Done;
            }
            (State::InInstructions, Event::Start(Tag::List(start))) => {
                self.handle_list_start(source, *start, &range);
            }
            (State::InInstructions, Event::End(TagEnd::List(_))) => {
                self.list_stack.pop();
            }
            (State::InInstructions, Event::Start(Tag::Item)) => {
                self.handle_item_start(&range);
            }
            (State::InInstructions, Event::End(TagEnd::Item)) => {
                // Only an ordered item boundary closes a step; the end of
                // a nested bullet leaves the enclosing step open.
                if self.list_stack.last() == Some(&ListKind::Ordered)
                    && let Some(step) = self.current_step.take()
                {
                    self.finalize_step(step);
                }
            }
            (State::InInstructions, Event::Code(code)) => {
                let source_range = self.byte_range_to_source_range(&range);
                if let Some(step) = self.current_step.as_mut() {
                    if PRIMITIVE_NAMES.contains(&code.as_ref()) {
                        step.primitive_name = Some(code.as_ref().into());
                    } else if span_is_suspicious(code.as_ref(), &step.prose) {
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

    /// Open a list inside Instructions. A top-level ordered list seeds the
    /// step counter from the document's literal numbering; when it is
    /// separated from the previous top-level ordered list only by HTML
    /// comments and blank lines it continues that list's step sequence
    /// instead (an HTML comment between items splits one logical markdown
    /// list into several parser-level lists).
    fn handle_list_start(
        &mut self,
        source: &str,
        start: Option<u64>,
        range: &std::ops::Range<usize>,
    ) {
        if self.list_stack.is_empty()
            && let Some(start) = start
        {
            let seed = u32::try_from(start.saturating_sub(1)).unwrap_or(u32::MAX);
            let continues = self.last_top_list_end.is_some_and(|end| {
                source
                    .get(end..range.start)
                    .is_some_and(gap_is_only_comments_and_blank)
            });
            self.top_index = if continues {
                self.top_index.max(seed)
            } else {
                seed
            };
        }
        self.list_stack.push(if start.is_some() {
            ListKind::Ordered
        } else {
            ListKind::Unordered
        });
    }

    /// Open a list item inside Instructions. An ordered item starts a new
    /// step (top-level or sub-numbered by nesting depth); an unordered
    /// item stays part of the enclosing step's prose.
    fn handle_item_start(&mut self, range: &std::ops::Range<usize>) {
        match self.list_stack.last() {
            Some(ListKind::Ordered) => {
                if let Some(step) = self.current_step.take() {
                    self.finalize_step(step);
                }
                if self.list_stack.len() == 1 {
                    self.top_index += 1;
                    self.sub_index = 0;
                    self.current_step = Some(StepBuilder::new(
                        StepNumber(vec![self.top_index]),
                        self.byte_range_to_source_range(range),
                    ));
                } else {
                    self.sub_index += 1;
                    self.current_step = Some(StepBuilder::new(
                        StepNumber(vec![self.top_index, self.sub_index]),
                        self.byte_range_to_source_range(range),
                    ));
                }
            }
            Some(ListKind::Unordered) => {
                // A bullet nested in a numbered step stays part of that
                // step: keep the current builder and separate the bullet's
                // text from what came before.
                if let Some(step) = self.current_step.as_mut()
                    && !step.prose.is_empty()
                {
                    step.prose.push(' ');
                }
            }
            None => {}
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

/// Whether the gap between two top-level ordered lists consists only of
/// whitespace and HTML comments (`<!-- ... -->`). Such a gap means the
/// second list continues the first's step sequence.
fn gap_is_only_comments_and_blank(gap: &str) -> bool {
    let mut rest = gap.trim_start();
    while !rest.is_empty() {
        let Some(after_open) = rest.strip_prefix("<!--") else {
            return false;
        };
        let Some(close) = after_open.find("-->") else {
            return false;
        };
        rest = after_open.get(close + 3..).unwrap_or_default().trim_start();
    }
    true
}

/// Whether a code span that failed the exact `PRIMITIVE_NAMES` match is
/// plausibly a typo'd primitive invocation. Two signals qualify:
///
/// - The span sits in primitive-invoking position — the step prose
///   accumulated so far ends with the word "invoke" or "call" — and has
///   the kebab-case shape of a primitive name.
/// - The span is within edit distance 2 of a known primitive name.
///
/// Anything else is ordinary vocabulary (`no-checkbox`, `keep-pending`,
/// `cli-config-dir`) and parses as prose.
fn span_is_suspicious(span: &str, preceding_prose: &str) -> bool {
    if span.is_empty() || span.contains(char::is_whitespace) {
        return false;
    }
    if looks_like_primitive(span) && in_invoking_position(preceding_prose) {
        return true;
    }
    PRIMITIVE_NAMES
        .iter()
        .any(|name| within_edit_distance(span, name, 2))
}

/// Whether the prose immediately preceding a code span puts that span in
/// primitive-invoking position: its last whitespace-separated word is
/// "invoke" or "call" (case-insensitive).
fn in_invoking_position(preceding_prose: &str) -> bool {
    preceding_prose
        .split_whitespace()
        .next_back()
        .is_some_and(|word| {
            word.eq_ignore_ascii_case("invoke") || word.eq_ignore_ascii_case("call")
        })
}

/// Bounded Levenshtein distance check: true when `a` and `b` are within
/// `max` single-character edits (insert / delete / substitute) of each
/// other. Bails out early once every path exceeds `max`.
fn within_edit_distance(a: &str, b: &str, max: usize) -> bool {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    if a.len().abs_diff(b.len()) > max {
        return false;
    }
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = Vec::with_capacity(b.len() + 1);
        cur.push(i + 1);
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            let val = (prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1);
            cur.push(val);
        }
        if cur.iter().min().copied().unwrap_or(usize::MAX) > max {
            return false;
        }
        prev = cur;
    }
    prev.last().copied().unwrap_or(usize::MAX) <= max
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
    fn primitive_names_match_mcp_tool_names_exactly() {
        // Guard against the six-site wiring gap (AGENTS.md gotcha,
        // recorded 2026-06-14 for `resolve-references`): a primitive
        // exposed over MCP must also be recognizable in command prose.
        // The two registries are exact-set equal today; if a deliberate
        // asymmetry ever appears, downgrade this to a superset check
        // with a comment naming the exception.
        use std::collections::BTreeSet;
        let parser: BTreeSet<&str> = PRIMITIVE_NAMES.iter().copied().collect();
        let mcp: BTreeSet<&str> = crate::mcp::server::TOOL_NAMES.iter().copied().collect();
        assert_eq!(
            parser, mcp,
            "parser PRIMITIVE_NAMES and mcp TOOL_NAMES diverged — wire the missing sites (see AGENTS.md Gotchas)"
        );
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

    fn step_numbers(procedure: &Procedure) -> Vec<Vec<u32>> {
        procedure
            .steps
            .iter()
            .map(|s| match s {
                Step::Primitive { number, .. }
                | Step::Extension { number, .. }
                | Step::Prose { number, .. } => number.0.clone(),
            })
            .collect()
    }

    #[test]
    fn comment_separated_lists_continue_one_step_sequence() {
        // `<!-- audit:ignore-promotion -->` between items splits one
        // markdown list into several parser-level lists; the step
        // sequence must continue across the split (prune.md's shape).
        let source = "# Cmd\n\n## Instructions\n\n\
            1. Invoke `read-spec` first.\n\n\
            2. Second step.\n\n\
            <!-- audit:ignore-promotion -->\n\
            3. Invoke `read-tasks` third.\n\n\
            4. Fourth step.\n\n\
            <!-- audit:ignore-promotion -->\n\
            5. Fifth step.\n";
        let procedure = parse_str(source).unwrap();
        assert_eq!(
            step_numbers(&procedure),
            vec![vec![1], vec![2], vec![3], vec![4], vec![5]]
        );
    }

    #[test]
    fn lazy_numbering_across_comment_splits_continues_sequence() {
        // status.md's shape: every item is literally `1.` and every item
        // is preceded by an ignore-promotion comment. The comment-split
        // lists continue the sequence, so the steps number 1..N.
        let source = "# Cmd\n\n## Instructions\n\n\
            <!-- audit:ignore-promotion -->\n\
            1. Invoke `dashboard` to load state.\n\n\
            <!-- audit:ignore-promotion -->\n\
            1. Render the preamble.\n\n\
            <!-- audit:ignore-promotion -->\n\
            1. Render the table.\n";
        let procedure = parse_str(source).unwrap();
        assert_eq!(step_numbers(&procedure), vec![vec![1], vec![2], vec![3]]);
    }

    #[test]
    fn ordered_list_start_value_seeds_the_counter() {
        // A list separated from the previous one by real prose re-seeds
        // from its literal start value instead of restarting at 1.
        let source = "# Cmd\n\n## Instructions\n\n\
            1. Invoke `read-spec` first.\n\n\
            A prose interlude between the lists.\n\n\
            4. Invoke `read-tasks` fourth.\n\
            5. Fifth step.\n";
        let procedure = parse_str(source).unwrap();
        assert_eq!(step_numbers(&procedure), vec![vec![1], vec![4], vec![5]]);
    }

    #[test]
    fn nested_bullets_stay_part_of_their_parent_step() {
        // Bullets nested in a numbered step are not steps of their own —
        // they fold into the parent step's prose, and the following
        // numbered item keeps the right top-level number.
        // NB: `concat!` (not `\`-continuation) so the bullets keep the
        // three-space indent that nests them under step 1.
        let source = concat!(
            "# Cmd\n\n## Instructions\n\n",
            "1. Invoke `read-spec` and check:\n",
            "   - the first bullet\n",
            "   - the second bullet\n",
            "2. Invoke `read-tasks` next.\n",
        );
        let procedure = parse_str(source).unwrap();
        assert_eq!(step_numbers(&procedure), vec![vec![1], vec![2]]);
        match &procedure.steps[0] {
            Step::Primitive { name, prose, .. } => {
                assert_eq!(name, "read-spec");
                assert!(
                    prose.contains("the first bullet") && prose.contains("the second bullet"),
                    "bullets fold into the parent step's prose: {prose:?}"
                );
            }
            other => panic!("expected primitive, got {other:?}"),
        }
    }

    #[test]
    fn ordinary_kebab_vocabulary_does_not_invalidate_new_format_files() {
        // Kebab-case spans that are neither in invoking position nor near
        // a primitive name are plain vocabulary (prune.md's shape).
        let source = "# Cmd\n\n## Instructions\n\n\
            1. Invoke `prune-tasks` in preview mode (`apply: false`); sections \
            classify as `spent` / `pending` / `no-checkbox`, default is a \
            `keep-pending` prune honoring `cli-config-dir`.\n";
        let procedure = parse_str(source).unwrap();
        match &procedure.steps[0] {
            Step::Primitive { name, .. } => assert_eq!(name, "prune-tasks"),
            other => panic!("expected primitive, got {other:?}"),
        }
    }

    #[test]
    fn unknown_name_in_invoking_position_is_invalid() {
        // "Invoke `X`" is primitive-invoking position: an unknown
        // kebab-case name there fails parsing even when it is nowhere
        // near a known primitive name.
        let source = "# Cmd\n\n## Instructions\n\n\
            1. Invoke `frobnicate-widget` on the target.\n\
            2. Invoke `read-tasks` to load tasks.\n";
        let err = parse_str(source).unwrap_err();
        match err {
            ParseError::Invalid { message, .. } => {
                assert!(message.contains("frobnicate-widget"));
            }
            ParseError::LegacyProse => panic!("expected Invalid, got LegacyProse"),
        }
    }

    #[test]
    fn near_miss_typo_outside_invoking_position_is_invalid() {
        // `read-spek` is edit distance 1 from `read-spec` — a typo even
        // without an "Invoke"/"call" cue in front of it.
        let source = "# Cmd\n\n## Instructions\n\n\
            1. The `read-spek` result feeds the next step.\n\
            2. Invoke `read-tasks` to load tasks.\n";
        let err = parse_str(source).unwrap_err();
        match err {
            ParseError::Invalid { message, .. } => {
                assert!(message.contains("read-spek"));
            }
            ParseError::LegacyProse => panic!("expected Invalid, got LegacyProse"),
        }
    }

    #[test]
    fn within_edit_distance_bounds() {
        assert!(within_edit_distance("read-spek", "read-spec", 2));
        assert!(within_edit_distance("read-spec", "read-spec", 2));
        assert!(within_edit_distance("gateconfirm", "gate-confirm", 2));
        assert!(!within_edit_distance("no-checkbox", "check-stuck", 2));
        assert!(!within_edit_distance("keep-pending", "append-task", 2));
        assert!(!within_edit_distance("cli-config-dir", "gate-confirm", 2));
    }

    /// Every rewritten command file must parse with step numbers equal to
    /// the document's literal numbering: monotonic 1..N at the top level
    /// (lazy `1.` numbering split by ignore-promotion comments renders as
    /// 1..N, and explicitly numbered documents carry 1..N literally).
    #[test]
    fn rewritten_command_files_parse_with_document_step_numbers() {
        let commands_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("runtime/.. exists")
            .join("framework/commands");
        for command in [
            "status",
            "target",
            "analyze",
            "implement",
            "plan",
            "specify",
            "review",
            "audit",
            "link",
            "prune",
            "clarify",
            "groom",
            "log",
        ] {
            let path = commands_dir.join(format!("{command}.md"));
            let source = std::fs::read_to_string(&path)
                .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
            let procedure = parse(&source, command)
                .unwrap_or_else(|err| panic!("{command}.md must parse: {err}"));

            let top_level: Vec<u32> = step_numbers(&procedure)
                .into_iter()
                .filter(|n| n.len() == 1)
                .map(|n| n[0])
                .collect();
            let literals = top_level_literal_markers(&source);
            assert_eq!(
                top_level.len(),
                literals.len(),
                "{command}.md: parsed top-level step count differs from the \
                 document's numbered items (parsed {top_level:?}, literal {literals:?})"
            );
            let expected: Vec<u32> = (1..=u32::try_from(literals.len()).unwrap()).collect();
            assert_eq!(
                top_level, expected,
                "{command}.md: steps must number monotonically 1..N \
                 (literal markers: {literals:?})"
            );
            // Explicitly numbered documents (not lazy all-`1.` style) must
            // match their literal numbers exactly.
            if literals.windows(2).all(|w| w[1] > w[0]) {
                assert_eq!(
                    top_level, literals,
                    "{command}.md: parsed numbers must equal the document's \
                     explicit literal numbers"
                );
            }
        }
    }

    /// Top-level ordered-list markers (`N. `) inside the `## Instructions`
    /// section, in document order, skipping fenced code blocks.
    fn top_level_literal_markers(source: &str) -> Vec<u32> {
        let mut in_instructions = false;
        let mut in_fence = false;
        let mut markers = Vec::new();
        for line in source.lines() {
            if line.starts_with("```") {
                in_fence = !in_fence;
                continue;
            }
            if in_fence {
                continue;
            }
            if line.starts_with("## ") {
                in_instructions = line.trim() == "## Instructions";
                continue;
            }
            if !in_instructions {
                continue;
            }
            let digits: String = line.chars().take_while(char::is_ascii_digit).collect();
            if !digits.is_empty() && line[digits.len()..].starts_with(". ") {
                markers.push(digits.parse::<u32>().expect("digits parse"));
            }
        }
        markers
    }
}
