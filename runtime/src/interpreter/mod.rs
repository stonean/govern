//! Procedure walker and JSON-over-stdio protocol I/O.
//!
//! [`Walker`] is the synchronous engine that consumes a parsed
//! [`Procedure`] step by step. For each step it either:
//!
//! - Dispatches to a primitive's pure-Rust function, merges its structured
//!   result into the walker context (so a later step's payload builder or a
//!   later primitive can read prior results), and emits a `progress`
//!   envelope (`Step::Primitive`). See [`Walker::merge_primitive_result`]
//!   for the merge policy.
//! - Emits an `llm-request` envelope and reads a matching
//!   `llm-response` from stdin (`Step::Extension`).
//! - Detects a gate trigger in the prose ("Ask the user to approve") and
//!   emits a `gate-confirm`, reading a `gate-response` back.
//! - Otherwise no-op (`Step::Prose`).
//!
//! At the end of the procedure the walker emits `complete`. Operational
//! errors halt the walk and emit an `error` envelope before returning.
//! Step ordering and message emission are deterministic given the same
//! procedure + inputs; the runtime panics on malformed JSON on stdin
//! (host-implementation bug, not a recoverable runtime condition).

#![allow(clippy::module_name_repetitions)]

pub mod payload;

use std::collections::HashSet;
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::io::{read_envelope, write_envelope};
use crate::primitives;
use crate::schema::extensions::{self, ValidationError, WriteCodeResponse};
use crate::schema::primitives::{
    AppendTaskArgs, ApplyManifestArgs, CheckRuleIdsArgs, CheckStuckArgs, ComputeReviewScopeArgs,
    CreateScenarioArgs, DashboardArgs, DeriveBoundaryArgs, DiscoverRuleFilesArgs,
    EnforceManifestArgs, ExtractArchiveArgs, FetchArchiveArgs, GateConfirmArgs, LintMarkdownArgs,
    MarkCriterionArgs, MarkTaskArgs, MergeClaudeMdArgs, MergeManagedBlockArgs,
    MergePermissionsArgs, MigrateSessionFileArgs, ProcessWaiversArgs, ReadSpecArgs, ReadTasksArgs,
    ResolveAnchorArgs, RunGeneratorArgs, SetStatusArgs, SubstituteTemplatesArgs, TraverseDepsArgs,
    ValidateFrontmatterArgs, WriteReviewArgs, WriteSessionArgs,
};
use crate::schema::procedure::{Procedure, Step, StepNumber};
use crate::schema::protocol::{ErrorLocation, ProtocolMessage};

const GATE_TRIGGER: &str = "ask the user to approve";

/// One run of the walker. The caller owns the procedure, repo path, and
/// reader/writer streams; the walker borrows them for its lifetime.
pub struct Walker<'a, R: BufRead, W: Write> {
    procedure: &'a Procedure,
    repo: PathBuf,
    context: Map<String, Value>,
    /// Keys present in `context` at construction — the session-seeded
    /// bindings (e.g. `feature`, `write-boundary`). A primitive result may
    /// never overwrite one of these; see [`Walker::merge_primitive_result`].
    seeded_keys: HashSet<String>,
    reader: &'a mut R,
    writer: &'a mut W,
    request_counter: u64,
}

/// Top-level outcome of [`Walker::run`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WalkOutcome {
    /// Walker emitted `complete` and exited cleanly.
    Complete,
    /// Walker emitted `error` and halted.
    Errored {
        /// Machine-readable error code.
        code: String,
        /// Human-readable description.
        message: String,
    },
}

impl<'a, R: BufRead, W: Write> Walker<'a, R, W> {
    /// Build a walker against `procedure`, rooted at `repo`. `context`
    /// carries CLI-supplied bindings (e.g., `feature`) that primitives
    /// deserialize their args from.
    pub fn new(
        procedure: &'a Procedure,
        repo: PathBuf,
        context: Map<String, Value>,
        reader: &'a mut R,
        writer: &'a mut W,
    ) -> Self {
        let seeded_keys = context.keys().cloned().collect();
        Self {
            procedure,
            repo,
            context,
            seeded_keys,
            reader,
            writer,
            request_counter: 0,
        }
    }

    /// Walk the procedure to completion. Emits envelopes as a side effect.
    ///
    /// # Errors
    ///
    /// Returns an I/O error if writing to `writer` fails or reading from
    /// `reader` fails. Operational errors from primitives are surfaced as
    /// `error` envelopes (not propagated as I/O errors); the walker
    /// returns `Ok(WalkOutcome::Errored)` in that case.
    pub fn run(&mut self) -> std::io::Result<WalkOutcome> {
        let steps = self.procedure.steps.clone();
        for step in &steps {
            if let Some(outcome) = self.handle_step(step)? {
                return Ok(outcome);
            }
        }
        self.emit_complete()?;
        Ok(WalkOutcome::Complete)
    }

    fn handle_step(&mut self, step: &Step) -> std::io::Result<Option<WalkOutcome>> {
        // Gate trigger pre-empts primitive/extension dispatch — any step
        // whose prose contains the gate trigger emits a gate-confirm
        // regardless of step type. The interpreter treats a denied gate
        // as a clean `complete` per the partial-failure resolution.
        let prose = step_prose(step);
        if prose.to_lowercase().contains(GATE_TRIGGER) {
            let number = step_number(step);
            let outcome = self.handle_gate(&number, prose)?;
            return Ok(outcome);
        }
        match step {
            Step::Primitive {
                number,
                name,
                prose: _,
                location,
            } => self.handle_primitive(number, name, *location),
            Step::Extension {
                number,
                identifier,
                prose,
                location: _,
            } => {
                let outcome = self.handle_extension(number, identifier, prose)?;
                Ok(outcome)
            }
            Step::Prose { .. } => Ok(None),
        }
    }

    fn handle_primitive(
        &mut self,
        number: &StepNumber,
        name: &str,
        location: crate::schema::procedure::SourceRange,
    ) -> std::io::Result<Option<WalkOutcome>> {
        let step_label = format_step_number(number);
        self.emit_progress(
            format!("dispatching primitive `{name}`"),
            Some(step_label.clone()),
            Some(name.into()),
        )?;
        match dispatch_primitive(name, &self.context, &self.repo) {
            Ok(result) => {
                self.merge_primitive_result(result);
                Ok(None)
            }
            Err(err) => {
                let code = match &err {
                    DispatchError::UnknownPrimitive(_) => "unknown-primitive".to_string(),
                    DispatchError::BadArgs(_) => "primitive-args-mismatch".to_string(),
                    DispatchError::Primitive(_) => "primitive-failure".to_string(),
                };
                let message = err.to_string();
                self.emit_error(
                    code.clone(),
                    message.clone(),
                    Some(ErrorLocation {
                        file: self.procedure.command.clone(),
                        line: location.start_line,
                        col: location.start_col,
                    }),
                )?;
                Ok(Some(WalkOutcome::Errored { code, message }))
            }
        }
    }

    /// Merge a primitive's structured result into the walker context so a
    /// later step can read prior results — e.g. `compute-review-scope`'s
    /// `scope`/`diff-base` and `discover-rule-files`'s `selected`/`rules-dir`
    /// feed `build_perform_review_request`, and `write-review` reads
    /// `diff-base` plus the accumulated `findings`.
    ///
    /// Merge policy (kept deliberately explicit):
    /// - Only an object result merges; a non-object result (array, scalar,
    ///   null) is ignored — there are no top-level keys to thread.
    /// - Each top-level key of the result is inserted into the context,
    ///   **except** a session-seeded key (one present at construction, such
    ///   as `write-boundary` or `feature`), which is load-bearing and is
    ///   never overwritten by a primitive result.
    /// - Among keys first introduced by primitives, last-write-wins.
    ///
    /// Results merge at the top level rather than under a per-primitive
    /// namespace because the payload builders and primitive arg binders read
    /// prior results by their bare key (`scope`, `selected`, `rules-dir`,
    /// `diff-base`); namespacing would hide them.
    fn merge_primitive_result(&mut self, result: Value) {
        let Value::Object(map) = result else {
            return;
        };
        for (key, value) in map {
            if self.seeded_keys.contains(&key) {
                continue;
            }
            self.context.insert(key, value);
        }
    }

    fn handle_extension(
        &mut self,
        number: &StepNumber,
        identifier: &str,
        prose: &str,
    ) -> std::io::Result<Option<WalkOutcome>> {
        let request_id = self.fresh_request_id();
        let request = match payload::build_extension_request(
            identifier,
            &self.context,
            &self.repo,
            &self.procedure.command,
            prose,
        ) {
            Ok(value) => value,
            Err(err) => {
                let code = err.code().to_string();
                let message = err.to_string();
                self.emit_error(code.clone(), message.clone(), None)?;
                return Ok(Some(WalkOutcome::Errored { code, message }));
            }
        };
        self.emit_llm_request(identifier, &request_id, request)?;
        let parsed = self.read_envelope()?;
        match parsed {
            ProtocolMessage::LlmResponse {
                request_id: response_id,
                response,
            } if response_id == request_id => {
                if let Some(outcome) = self.validate_llm_response(identifier, &response)? {
                    return Ok(Some(outcome));
                }
                // `performReview` runs once per pass; accumulate each pass's
                // findings into the shared `findings` context key so a later
                // `write-review` step consumes the union across all passes.
                if identifier == "performReview"
                    && let Some(Value::Array(findings)) = response.get("findings")
                {
                    let findings = findings.clone();
                    match self.context.get_mut("findings") {
                        Some(Value::Array(existing)) => existing.extend(findings),
                        _ => {
                            self.context
                                .insert("findings".into(), Value::Array(findings));
                        }
                    }
                }
                self.context.insert(format!("llm:{identifier}"), response);
                self.emit_progress(
                    format!("received llm-response for `{identifier}`"),
                    Some(format_step_number(number)),
                    None,
                )?;
                Ok(None)
            }
            ProtocolMessage::LlmResponse {
                request_id: other, ..
            } => {
                let message =
                    format!("llm-response request-id mismatch: expected {request_id}, got {other}");
                self.emit_error("protocol-mismatch".into(), message.clone(), None)?;
                Ok(Some(WalkOutcome::Errored {
                    code: "protocol-mismatch".into(),
                    message,
                }))
            }
            other => {
                let message = format!("expected llm-response, got {other:?}");
                self.emit_error("protocol-mismatch".into(), message.clone(), None)?;
                Ok(Some(WalkOutcome::Errored {
                    code: "protocol-mismatch".into(),
                    message,
                }))
            }
        }
    }

    fn handle_gate(
        &mut self,
        number: &StepNumber,
        prose: &str,
    ) -> std::io::Result<Option<WalkOutcome>> {
        let request_id = self.fresh_request_id();
        let gate = format!("step-{}", format_step_number(number));
        self.emit_gate_confirm(&gate, &request_id, prose)?;
        let parsed = self.read_envelope()?;
        match parsed {
            ProtocolMessage::GateResponse {
                request_id: response_id,
                confirmed,
            } if response_id == request_id => {
                self.emit_progress(
                    format!(
                        "gate `{gate}` {}",
                        if confirmed { "confirmed" } else { "denied" }
                    ),
                    Some(format_step_number(number)),
                    None,
                )?;
                if confirmed {
                    Ok(None)
                } else {
                    // Denial is a clean exit per §partial-failure-semantics.
                    self.emit_complete_with(
                        serde_json::json!({ "confirmed": false, "gate": gate }),
                    )?;
                    Ok(Some(WalkOutcome::Complete))
                }
            }
            ProtocolMessage::GateResponse {
                request_id: other, ..
            } => {
                let message = format!(
                    "gate-response request-id mismatch: expected {request_id}, got {other}"
                );
                self.emit_error("protocol-mismatch".into(), message.clone(), None)?;
                Ok(Some(WalkOutcome::Errored {
                    code: "protocol-mismatch".into(),
                    message,
                }))
            }
            other => {
                let message = format!("expected gate-response, got {other:?}");
                self.emit_error("protocol-mismatch".into(), message.clone(), None)?;
                Ok(Some(WalkOutcome::Errored {
                    code: "protocol-mismatch".into(),
                    message,
                }))
            }
        }
    }

    fn fresh_request_id(&mut self) -> String {
        self.request_counter += 1;
        format!("req-{}", self.request_counter)
    }

    fn emit(&mut self, message: &ProtocolMessage) -> std::io::Result<()> {
        write_envelope(self.writer, message)
    }

    fn emit_progress(
        &mut self,
        message: String,
        step: Option<String>,
        primitive: Option<String>,
    ) -> std::io::Result<()> {
        self.emit(&ProtocolMessage::Progress {
            message,
            step,
            primitive,
        })
    }

    fn emit_llm_request(
        &mut self,
        extension_point: &str,
        request_id: &str,
        request: Value,
    ) -> std::io::Result<()> {
        self.emit(&ProtocolMessage::LlmRequest {
            extension_point: extension_point.into(),
            request_id: request_id.into(),
            request,
        })
    }

    fn emit_gate_confirm(
        &mut self,
        gate: &str,
        request_id: &str,
        prompt: &str,
    ) -> std::io::Result<()> {
        self.emit(&ProtocolMessage::GateConfirm {
            gate: gate.into(),
            request_id: request_id.into(),
            prompt: prompt.trim().into(),
        })
    }

    fn emit_complete(&mut self) -> std::io::Result<()> {
        self.emit_complete_with(Value::Object(Map::new()))
    }

    fn emit_complete_with(&mut self, result: Value) -> std::io::Result<()> {
        self.emit(&ProtocolMessage::Complete {
            result,
            runtime_version: env!("CARGO_PKG_VERSION").into(),
        })
    }

    fn emit_error(
        &mut self,
        code: String,
        message: String,
        location: Option<ErrorLocation>,
    ) -> std::io::Result<()> {
        self.emit(&ProtocolMessage::Error {
            code,
            message,
            runtime_version: env!("CARGO_PKG_VERSION").into(),
            location,
        })
    }

    fn read_envelope(&mut self) -> std::io::Result<ProtocolMessage> {
        read_envelope(self.reader)
    }

    /// Validate an incoming `llm-response` payload against the schema for
    /// the extension point that emitted the request, and (for `writeCode`)
    /// reject edits whose path escapes the write boundary. Returns
    /// `Ok(Some(Errored))` when validation fails (the caller emits the
    /// error envelope and halts); `Ok(None)` when the response is well-formed.
    fn validate_llm_response(
        &mut self,
        identifier: &str,
        response: &Value,
    ) -> std::io::Result<Option<WalkOutcome>> {
        if let Err(err) = extensions::validate_response(identifier, response) {
            let (code, message) = match &err {
                ValidationError::UnknownExtension(_) => {
                    ("unknown-extension".to_string(), err.to_string())
                }
                ValidationError::Schema { .. } => ("schema-mismatch".to_string(), err.to_string()),
                ValidationError::OutOfBoundary { .. } => {
                    // Reachable only through validate_write_code_boundary;
                    // validate_response never produces it.
                    ("out-of-boundary-edit".to_string(), err.to_string())
                }
            };
            self.emit_error(code.clone(), message.clone(), None)?;
            return Ok(Some(WalkOutcome::Errored { code, message }));
        }
        if identifier == "writeCode" {
            // `validate_response` already confirmed the shape, so the
            // boundary check on the typed struct can't fail to deserialize.
            let parsed: WriteCodeResponse =
                serde_json::from_value(response.clone()).map_err(|err| {
                    std::io::Error::other(format!("re-parse of validated payload failed: {err}"))
                })?;
            let boundary = self.write_boundary();
            if let Err(err) = extensions::validate_write_code_boundary(&parsed, &boundary) {
                let message = err.to_string();
                self.emit_error("out-of-boundary-edit".into(), message.clone(), None)?;
                return Ok(Some(WalkOutcome::Errored {
                    code: "out-of-boundary-edit".into(),
                    message,
                }));
            }
        }
        Ok(None)
    }

    /// Read the write boundary from the walker's context. The
    /// `write-boundary` context key is expected to be a `Vec<String>` of
    /// glob patterns; absent or malformed values yield an empty boundary,
    /// which rejects every path.
    fn write_boundary(&self) -> Vec<String> {
        self.context
            .get("write-boundary")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(str::to_string))
                    .collect()
            })
            .unwrap_or_default()
    }
}

fn step_prose(step: &Step) -> &str {
    match step {
        Step::Primitive { prose, .. } | Step::Extension { prose, .. } => prose,
        Step::Prose { text, .. } => text,
    }
}

fn step_number(step: &Step) -> StepNumber {
    match step {
        Step::Primitive { number, .. }
        | Step::Extension { number, .. }
        | Step::Prose { number, .. } => number.clone(),
    }
}

fn format_step_number(number: &StepNumber) -> String {
    number
        .0
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(".")
}

#[derive(Debug, thiserror::Error)]
enum DispatchError {
    #[error("unknown primitive `{0}`")]
    UnknownPrimitive(String),
    #[error("failed to bind args for primitive: {0}")]
    BadArgs(serde_json::Error),
    #[error("{0}")]
    Primitive(#[from] primitives::PrimitiveError),
}

/// Dispatch a primitive by name. Args are deserialized from `context`
/// — any keys it doesn't need are ignored, so callers can pass a single
/// merged binding map. Returns the primitive's result as a JSON value.
fn dispatch_primitive(
    name: &str,
    context: &Map<String, Value>,
    repo: &Path,
) -> Result<Value, DispatchError> {
    let value = Value::Object(context.clone());
    macro_rules! call {
        ($args:ty, $module:ident) => {{
            let args: $args = serde_json::from_value(value).map_err(DispatchError::BadArgs)?;
            let result = primitives::$module::run(&args, repo)?;
            Ok(serde_json::to_value(result).unwrap_or(Value::Null))
        }};
    }
    match name {
        "read-spec" => call!(ReadSpecArgs, read_spec),
        "read-tasks" => call!(ReadTasksArgs, read_tasks),
        "mark-task" => call!(MarkTaskArgs, mark_task),
        "mark-criterion" => call!(MarkCriterionArgs, mark_criterion),
        "set-status" => call!(SetStatusArgs, set_status),
        "derive-boundary" => call!(DeriveBoundaryArgs, derive_boundary),
        "discover-rule-files" => call!(DiscoverRuleFilesArgs, discover_rule_files),
        "process-waivers" => call!(ProcessWaiversArgs, process_waivers),
        "compute-review-scope" => call!(ComputeReviewScopeArgs, compute_review_scope),
        "write-review" => call!(WriteReviewArgs, write_review),
        "check-stuck" => call!(CheckStuckArgs, check_stuck),
        "validate-frontmatter" => call!(ValidateFrontmatterArgs, validate_frontmatter),
        "resolve-anchor" => call!(ResolveAnchorArgs, resolve_anchor),
        "traverse-deps" => call!(TraverseDepsArgs, traverse_deps),
        "check-rule-ids" => call!(CheckRuleIdsArgs, check_rule_ids),
        "run-generator" => call!(RunGeneratorArgs, run_generator),
        "lint-markdown" => call!(LintMarkdownArgs, lint_markdown),
        "fetch-archive" => call!(FetchArchiveArgs, fetch_archive),
        "extract-archive" => call!(ExtractArchiveArgs, extract_archive),
        "substitute-templates" => call!(SubstituteTemplatesArgs, substitute_templates),
        "merge-claude-md" => call!(MergeClaudeMdArgs, merge_claude_md),
        "apply-manifest" => call!(ApplyManifestArgs, apply_manifest),
        "enforce-manifest" => call!(EnforceManifestArgs, enforce_manifest),
        "merge-managed-block" => call!(MergeManagedBlockArgs, merge_managed_block),
        "merge-permissions" => call!(MergePermissionsArgs, merge_permissions),
        "migrate-session-file" => call!(MigrateSessionFileArgs, migrate_session_file),
        "create-scenario" => call!(CreateScenarioArgs, create_scenario),
        "append-task" => call!(AppendTaskArgs, append_task),
        "dashboard" => call!(DashboardArgs, dashboard),
        "write-session" => call!(WriteSessionArgs, write_session),
        "gate-confirm" => {
            // The interpreter-level gate handler emits gate-confirm via
            // its own path (see handle_gate). When a primitive named
            // `gate-confirm` appears directly in a procedure, surface
            // the prompt payload as a domain result rather than
            // blocking — the interpreter routes via the prose-trigger
            // mechanism.
            let args: GateConfirmArgs =
                serde_json::from_value(value).map_err(DispatchError::BadArgs)?;
            let payload = primitives::gate_confirm::prompt_payload(
                &args,
                &primitives::gate_confirm::fresh_request_id(),
            );
            Ok(serde_json::to_value(payload).unwrap_or(Value::Null))
        }
        other => Err(DispatchError::UnknownPrimitive(other.into())),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::schema::procedure::SourceRange;
    use std::io::Cursor;

    fn loc() -> SourceRange {
        SourceRange {
            start_line: 1,
            start_col: 1,
            end_line: 1,
            end_col: 1,
        }
    }

    fn ctx_with_feature(feature: &str) -> Map<String, Value> {
        let mut m = Map::new();
        m.insert("feature".into(), Value::String(feature.into()));
        m
    }

    fn fixture_repo() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/primitives/sample-repo")
    }

    #[test]
    fn empty_procedure_emits_complete_only() {
        let procedure = Procedure {
            command: "noop".into(),
            steps: vec![],
        };
        let mut reader = Cursor::new(String::new());
        let mut writer: Vec<u8> = Vec::new();
        let mut walker = Walker::new(
            &procedure,
            fixture_repo(),
            Map::new(),
            &mut reader,
            &mut writer,
        );
        let outcome = walker.run().unwrap();
        assert_eq!(outcome, WalkOutcome::Complete);
        let lines: Vec<&str> = std::str::from_utf8(&writer).unwrap().lines().collect();
        assert_eq!(lines.len(), 1);
        let value: Value = serde_json::from_str(lines[0]).unwrap();
        assert_eq!(value["type"], "complete");
    }

    #[test]
    fn primitive_step_dispatches_and_emits_progress() {
        let procedure = Procedure {
            command: "test".into(),
            steps: vec![Step::Primitive {
                number: StepNumber(vec![1]),
                name: "read-spec".into(),
                prose: String::new(),
                location: loc(),
            }],
        };
        let mut reader = Cursor::new(String::new());
        let mut writer: Vec<u8> = Vec::new();
        let mut walker = Walker::new(
            &procedure,
            fixture_repo(),
            ctx_with_feature("001-basic"),
            &mut reader,
            &mut writer,
        );
        let outcome = walker.run().unwrap();
        assert_eq!(outcome, WalkOutcome::Complete);
        let lines: Vec<Value> = std::str::from_utf8(&writer)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0]["type"], "progress");
        assert_eq!(lines[0]["primitive"], "read-spec");
        assert_eq!(lines[1]["type"], "complete");
    }

    #[test]
    fn primitive_failure_emits_error_and_halts() {
        let procedure = Procedure {
            command: "test".into(),
            steps: vec![
                Step::Primitive {
                    number: StepNumber(vec![1]),
                    name: "read-spec".into(),
                    prose: String::new(),
                    location: loc(),
                },
                Step::Primitive {
                    number: StepNumber(vec![2]),
                    name: "read-tasks".into(),
                    prose: String::new(),
                    location: loc(),
                },
            ],
        };
        let mut reader = Cursor::new(String::new());
        let mut writer: Vec<u8> = Vec::new();
        let mut walker = Walker::new(
            &procedure,
            fixture_repo(),
            ctx_with_feature("999-nonexistent"),
            &mut reader,
            &mut writer,
        );
        let outcome = walker.run().unwrap();
        match outcome {
            WalkOutcome::Errored { code, .. } => assert_eq!(code, "primitive-failure"),
            WalkOutcome::Complete => panic!("expected Errored, got Complete"),
        }
        let lines: Vec<Value> = std::str::from_utf8(&writer)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        // progress(read-spec), error — second primitive never runs.
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0]["type"], "progress");
        assert_eq!(lines[1]["type"], "error");
    }

    #[test]
    fn extension_step_emits_llm_request_and_consumes_response() {
        let procedure = Procedure {
            command: "test".into(),
            steps: vec![Step::Extension {
                number: StepNumber(vec![1]),
                identifier: "assessSpecQuality".into(),
                prose: String::new(),
                location: loc(),
            }],
        };
        let response =
            "{\"type\":\"llm-response\",\"request-id\":\"req-1\",\"response\":{\"passed\":true}}\n";
        let mut reader = Cursor::new(response.to_string());
        let mut writer: Vec<u8> = Vec::new();
        let mut walker = Walker::new(
            &procedure,
            fixture_repo(),
            Map::new(),
            &mut reader,
            &mut writer,
        );
        let outcome = walker.run().unwrap();
        assert_eq!(outcome, WalkOutcome::Complete);
        let lines: Vec<Value> = std::str::from_utf8(&writer)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        // llm-request, progress(received), complete
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0]["type"], "llm-request");
        assert_eq!(lines[0]["extension-point"], "assessSpecQuality");
        assert_eq!(lines[0]["request-id"], "req-1");
        assert_eq!(lines[1]["type"], "progress");
        assert_eq!(lines[2]["type"], "complete");
    }

    #[test]
    fn perform_review_emits_one_llm_request_per_pass_step() {
        // One `performReview` step per pass → one llm-request each. A skipped
        // pass is simply an absent step (this procedure carries three of the
        // five), so no request is emitted for it.
        let step = |n: u32| Step::Extension {
            number: StepNumber(vec![n]),
            identifier: "performReview".into(),
            prose: String::new(),
            location: loc(),
        };
        let procedure = Procedure {
            command: "review".into(),
            steps: vec![step(1), step(2), step(3)],
        };
        let responses = concat!(
            "{\"type\":\"llm-response\",\"request-id\":\"req-1\",\"response\":{\"findings\":[]}}\n",
            "{\"type\":\"llm-response\",\"request-id\":\"req-2\",\"response\":{\"findings\":[]}}\n",
            "{\"type\":\"llm-response\",\"request-id\":\"req-3\",\"response\":{\"findings\":[]}}\n",
        );
        let mut reader = Cursor::new(responses.to_string());
        let mut writer: Vec<u8> = Vec::new();
        let mut walker = Walker::new(
            &procedure,
            fixture_repo(),
            Map::new(),
            &mut reader,
            &mut writer,
        );
        assert_eq!(walker.run().unwrap(), WalkOutcome::Complete);
        let lines: Vec<Value> = std::str::from_utf8(&writer)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        let requests: Vec<&Value> = lines
            .iter()
            .filter(|l| l["type"] == "llm-request")
            .collect();
        assert_eq!(requests.len(), 3);
        for request in requests {
            assert_eq!(request["extension-point"], "performReview");
        }
    }

    #[test]
    fn perform_review_findings_flow_into_write_review() {
        // Two pass steps then a write-review step: each pass's findings
        // accumulate into `context["findings"]`, which write-review consumes.
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("specs/001-x");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("spec.md"),
            "---\nstatus: in-progress\ndependencies: []\n---\n\n# x\n",
        )
        .unwrap();

        let procedure = Procedure {
            command: "review".into(),
            steps: vec![
                Step::Extension {
                    number: StepNumber(vec![1]),
                    identifier: "performReview".into(),
                    prose: String::new(),
                    location: loc(),
                },
                Step::Extension {
                    number: StepNumber(vec![2]),
                    identifier: "performReview".into(),
                    prose: String::new(),
                    location: loc(),
                },
                Step::Primitive {
                    number: StepNumber(vec![3]),
                    name: "write-review".into(),
                    prose: String::new(),
                    location: loc(),
                },
            ],
        };
        let responses = concat!(
            "{\"type\":\"llm-response\",\"request-id\":\"req-1\",\"response\":{\"findings\":[{\"rule\":\"SEC-BE-001\",\"severity\":\"must\",\"file\":\"runtime/src/a.rs\",\"line-range\":\"1-5\",\"confidence\":\"high\"}]}}\n",
            "{\"type\":\"llm-response\",\"request-id\":\"req-2\",\"response\":{\"findings\":[{\"rule\":\"QUAL-002\",\"severity\":\"should\",\"file\":\"runtime/src/b.rs\",\"line-range\":\"1-5\",\"confidence\":\"high\"}]}}\n",
        );
        let mut context = Map::new();
        context.insert("feature".into(), Value::String("001-x".into()));
        context.insert(
            "reviewed-at".into(),
            Value::String("2026-07-04T00:00:00Z".into()),
        );
        context.insert("reviewed-against".into(), Value::String("abc1234".into()));
        context.insert("diff-base".into(), Value::String("def5678".into()));

        let mut reader = Cursor::new(responses.to_string());
        let mut writer: Vec<u8> = Vec::new();
        let mut walker = Walker::new(
            &procedure,
            tmp.path().to_path_buf(),
            context,
            &mut reader,
            &mut writer,
        );
        assert_eq!(walker.run().unwrap(), WalkOutcome::Complete);

        // write-review rendered the union of both passes' findings.
        let review = std::fs::read_to_string(dir.join("review.md")).unwrap();
        assert!(review.contains("must-violations: 1"));
        assert!(review.contains("should-violations: 1"));
        assert!(review.contains("### MUST: SEC-BE-001"));
        assert!(review.contains("### SHOULD: QUAL-002"));
        // Blocking flowed to the spec review block.
        let spec = std::fs::read_to_string(dir.join("spec.md")).unwrap();
        assert!(spec.contains("blocking: true"));
    }

    #[test]
    fn malformed_llm_response_emits_schema_mismatch_error() {
        let procedure = Procedure {
            command: "test".into(),
            steps: vec![Step::Extension {
                number: StepNumber(vec![1]),
                identifier: "assessSpecQuality".into(),
                prose: String::new(),
                location: loc(),
            }],
        };
        // Missing required `passed` field.
        let response = "{\"type\":\"llm-response\",\"request-id\":\"req-1\",\"response\":{\"finding\":null}}\n";
        let mut reader = Cursor::new(response.to_string());
        let mut writer: Vec<u8> = Vec::new();
        let mut walker = Walker::new(
            &procedure,
            fixture_repo(),
            Map::new(),
            &mut reader,
            &mut writer,
        );
        let outcome = walker.run().unwrap();
        match outcome {
            WalkOutcome::Errored { code, .. } => assert_eq!(code, "schema-mismatch"),
            WalkOutcome::Complete => panic!("expected Errored, got Complete"),
        }
        let envelopes: Vec<Value> = std::str::from_utf8(&writer)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        // llm-request then error — no progress.
        assert_eq!(envelopes.len(), 2);
        assert_eq!(envelopes[0]["type"], "llm-request");
        assert_eq!(envelopes[1]["type"], "error");
        assert_eq!(envelopes[1]["code"], "schema-mismatch");
    }

    #[test]
    fn out_of_boundary_write_code_edit_emits_error() {
        let procedure = Procedure {
            command: "test".into(),
            steps: vec![Step::Extension {
                number: StepNumber(vec![1]),
                identifier: "writeCode".into(),
                prose: String::new(),
                location: loc(),
            }],
        };
        // Schema-valid writeCode response with an edit outside the boundary.
        let response = "{\"type\":\"llm-response\",\"request-id\":\"req-1\",\"response\":{\"edits\":[{\"path\":\"framework/constitution.md\",\"action\":\"edit\",\"content\":\"malicious\"}],\"summary\":\"x\"}}\n";
        let mut reader = Cursor::new(response.to_string());
        let mut writer: Vec<u8> = Vec::new();
        let mut context = Map::new();
        context.insert(
            "write-boundary".into(),
            Value::Array(vec![Value::String("runtime/**".into())]),
        );
        let mut walker = Walker::new(
            &procedure,
            fixture_repo(),
            context,
            &mut reader,
            &mut writer,
        );
        let outcome = walker.run().unwrap();
        match outcome {
            WalkOutcome::Errored { code, .. } => assert_eq!(code, "out-of-boundary-edit"),
            WalkOutcome::Complete => panic!("expected Errored, got Complete"),
        }
        let envelopes: Vec<Value> = std::str::from_utf8(&writer)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        assert_eq!(envelopes.last().unwrap()["code"], "out-of-boundary-edit");
        assert!(
            envelopes.last().unwrap()["message"]
                .as_str()
                .unwrap()
                .contains("framework/constitution.md")
        );
    }

    #[test]
    fn gate_trigger_in_prose_emits_gate_confirm_and_resumes_on_confirmed() {
        let procedure = Procedure {
            command: "test".into(),
            steps: vec![Step::Prose {
                number: StepNumber(vec![1]),
                text: "Ask the user to approve the transition.".into(),
                location: loc(),
            }],
        };
        let response = "{\"type\":\"gate-response\",\"request-id\":\"req-1\",\"confirmed\":true}\n";
        let mut reader = Cursor::new(response.to_string());
        let mut writer: Vec<u8> = Vec::new();
        let mut walker = Walker::new(
            &procedure,
            fixture_repo(),
            Map::new(),
            &mut reader,
            &mut writer,
        );
        let outcome = walker.run().unwrap();
        assert_eq!(outcome, WalkOutcome::Complete);
        let lines: Vec<Value> = std::str::from_utf8(&writer)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        // gate-confirm, progress(confirmed), complete
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[0]["type"], "gate-confirm");
        assert_eq!(lines[2]["type"], "complete");
    }

    #[test]
    fn gate_denial_exits_cleanly_with_confirmed_false() {
        let procedure = Procedure {
            command: "test".into(),
            steps: vec![Step::Prose {
                number: StepNumber(vec![1]),
                text: "Ask the user to approve the destructive op.".into(),
                location: loc(),
            }],
        };
        let response =
            "{\"type\":\"gate-response\",\"request-id\":\"req-1\",\"confirmed\":false}\n";
        let mut reader = Cursor::new(response.to_string());
        let mut writer: Vec<u8> = Vec::new();
        let mut walker = Walker::new(
            &procedure,
            fixture_repo(),
            Map::new(),
            &mut reader,
            &mut writer,
        );
        let outcome = walker.run().unwrap();
        assert_eq!(outcome, WalkOutcome::Complete);
        let lines: Vec<Value> = std::str::from_utf8(&writer)
            .unwrap()
            .lines()
            .map(|l| serde_json::from_str(l).unwrap())
            .collect();
        // gate-confirm, progress(denied), complete(confirmed: false)
        assert_eq!(lines.len(), 3);
        assert_eq!(lines[2]["type"], "complete");
        assert_eq!(lines[2]["result"]["confirmed"], false);
    }

    #[test]
    fn prose_step_is_noop() {
        let procedure = Procedure {
            command: "test".into(),
            steps: vec![Step::Prose {
                number: StepNumber(vec![1]),
                text: "Do the thing.".into(),
                location: loc(),
            }],
        };
        let mut reader = Cursor::new(String::new());
        let mut writer: Vec<u8> = Vec::new();
        let mut walker = Walker::new(
            &procedure,
            fixture_repo(),
            Map::new(),
            &mut reader,
            &mut writer,
        );
        let outcome = walker.run().unwrap();
        assert_eq!(outcome, WalkOutcome::Complete);
        let lines: Vec<&str> = std::str::from_utf8(&writer).unwrap().lines().collect();
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("\"complete\""));
    }
}
