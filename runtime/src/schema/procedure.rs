//! Procedure AST — produced by the parser, consumed by the interpreter.
//!
//! Types mirror the AST description in
//! `specs/022-deterministic-runtime/data-model.md`. They serialize via `serde`
//! only for the `runtime parse <file>` debug surface; the runtime never
//! persists the AST to disk.

#![allow(clippy::module_name_repetitions)]

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Numbered path through nested list items.
///
/// `[1, 2]` represents step "1.2"; `[3]` represents step "3".
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct StepNumber(pub Vec<u32>);

/// 1-based line + column range into the source file.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub struct SourceRange {
    /// 1-based line where the range starts.
    pub start_line: u32,
    /// 1-based column where the range starts.
    pub start_col: u32,
    /// 1-based line where the range ends.
    pub end_line: u32,
    /// 1-based column where the range ends.
    pub end_col: u32,
}

/// One step in a parsed procedure.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Step {
    /// A backtick-quoted primitive call inside a numbered step.
    Primitive {
        /// Step number path.
        number: StepNumber,
        /// Primitive name; matches an entry in §The primitive library.
        name: String,
        /// Surrounding prose preserved for the markdown-only path.
        prose: String,
        /// Source location of the step body.
        location: SourceRange,
    },
    /// An HTML-comment LLM extension-point marker on a step.
    Extension {
        /// Step number path.
        number: StepNumber,
        /// Extension-point identifier (e.g., "writeCode").
        identifier: String,
        /// Surrounding prose preserved for the markdown-only path.
        prose: String,
        /// Source location of the step body.
        location: SourceRange,
    },
    /// A non-primitive, non-extension prose step.
    Prose {
        /// Step number path.
        number: StepNumber,
        /// Step body text.
        text: String,
        /// Source location of the step body.
        location: SourceRange,
    },
}

/// A parsed slash command Instructions section.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Procedure {
    /// Command name (e.g., "status").
    pub command: String,
    /// Steps in declaration order.
    pub steps: Vec<Step>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::{Procedure, SourceRange, Step, StepNumber};

    fn sample_procedure() -> Procedure {
        Procedure {
            command: "status".into(),
            steps: vec![
                Step::Primitive {
                    number: StepNumber(vec![1]),
                    name: "read-spec".into(),
                    prose: "Read the spec for the target feature.".into(),
                    location: SourceRange {
                        start_line: 12,
                        start_col: 1,
                        end_line: 12,
                        end_col: 64,
                    },
                },
                Step::Extension {
                    number: StepNumber(vec![2]),
                    identifier: "writeCode".into(),
                    prose: "Write the code for the task.".into(),
                    location: SourceRange {
                        start_line: 14,
                        start_col: 1,
                        end_line: 14,
                        end_col: 80,
                    },
                },
                Step::Prose {
                    number: StepNumber(vec![3]),
                    text: "Verify the result by hand.".into(),
                    location: SourceRange {
                        start_line: 16,
                        start_col: 1,
                        end_line: 16,
                        end_col: 30,
                    },
                },
            ],
        }
    }

    #[test]
    fn round_trip_procedure() {
        let original = sample_procedure();
        let text = serde_json::to_string(&original).unwrap();
        let parsed: Procedure = serde_json::from_str(&text).unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn step_serializes_with_kebab_case_tag() {
        let step = Step::Primitive {
            number: StepNumber(vec![1, 2]),
            name: "mark-task".into(),
            prose: String::new(),
            location: SourceRange {
                start_line: 1,
                start_col: 1,
                end_line: 1,
                end_col: 1,
            },
        };
        let value: serde_json::Value = serde_json::to_value(&step).unwrap();
        assert_eq!(value["kind"], "primitive");
        assert_eq!(value["name"], "mark-task");
    }
}
