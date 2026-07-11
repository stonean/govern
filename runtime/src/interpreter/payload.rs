//! Extension-point request builders.
//!
//! Bundles the static context (`constitution-excerpts`, `plan-relevant-files`,
//! `write-boundary`) that LLM extension points need into the typed
//! [`WriteCodeRequest`] / [`WriteSpecBodyRequest`] /
//! [`AssessSpecQualityRequest`] / [`AskClarifyQuestionRequest`] /
//! [`RouteInboxItemRequest`] shapes defined in
//! [`crate::schema::extensions`]. The interpreter calls
//! [`build_extension_request`] just before emitting an `llm-request` envelope;
//! the result replaces the previous "dump the walker context as the request"
//! behavior with a payload whose field order is cache-anchored per the
//! spec 022 LLM extension points contract (stable prefix front, per-task
//! variable suffix last).
//!
//! Per the extension-request-hygiene scenario, walker-internal accumulator
//! keys (prior `llm:*` response echoes and the cross-pass `findings`
//! array) are filtered from every legacy-compat context merge, and an
//! unknown extension identifier is a structured error rather than a raw
//! context dump.
//!
//! [`WriteCodeRequest`]: crate::schema::extensions::WriteCodeRequest
//! [`WriteSpecBodyRequest`]: crate::schema::extensions::WriteSpecBodyRequest
//! [`AssessSpecQualityRequest`]: crate::schema::extensions::AssessSpecQualityRequest
//! [`AskClarifyQuestionRequest`]: crate::schema::extensions::AskClarifyQuestionRequest
//! [`RouteInboxItemRequest`]: crate::schema::extensions::RouteInboxItemRequest

#![allow(clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use regex::Regex;
use serde_json::{Map, Value};

use crate::host::Host;
use crate::primitives::read_tasks;
use crate::schema::extensions::{
    AskClarifyQuestionRequest, AssessSpecQualityRequest, AssessSpecQualityRule, ClarifyQuestion,
    PerformReviewRequest, PlanRelevantFile, ReviewRuleFile, ReviewScopeFile, RouteInboxItemRequest,
    RouteInboxSpec, WriteCodeRequest, WriteCodeTask, WriteSpecBodyRequest,
};
use crate::schema::primitives::{Frontmatter, ReadTasksArgs};

/// Errors that abort payload construction. The interpreter surfaces these
/// as structured `error` envelopes (e.g.,
/// [`SecretExfiltration`](PayloadError::SecretExfiltration) → code
/// `secret-exfiltration-blocked`).
#[derive(Debug, thiserror::Error)]
pub enum PayloadError {
    /// A path listed in the plan's Affected Files matched a secret-bearing
    /// pattern (`.env`, `credentials*`, etc.), was marked ignored by
    /// `.gitignore`, or canonicalized to a location outside the repo root
    /// (path traversal — pattern label `out-of-repo`).
    #[error("secret-exfiltration-blocked: '{path}' matches pattern '{pattern}'")]
    SecretExfiltration {
        /// Offending repo-relative path.
        path: String,
        /// Pattern that matched: a glob name (`.env`, `.env.*`,
        /// `*-secrets.*`, `credentials*`), `.gitignore`, or `out-of-repo`
        /// for paths whose canonical form escapes the repo root.
        pattern: String,
    },
    /// The extension identifier has no typed request builder in this
    /// runtime version. Emitting the raw walker context (the pre-hygiene
    /// fallback) would leak accumulator state to the host, so the walk
    /// halts with a structured error instead
    /// (extension-request-hygiene scenario).
    #[error("unknown extension point `{identifier}`: no typed request builder in this runtime")]
    UnknownExtension {
        /// The unrecognized extension identifier from the step marker.
        identifier: String,
    },
}

/// Machine-readable code emitted in the `error` envelope when payload
/// construction fails. Kept stable for host integrations.
impl PayloadError {
    /// Return the envelope code that corresponds to this variant.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::SecretExfiltration { .. } => "secret-exfiltration-blocked",
            // Matches the code the walker emits when a response arrives
            // for an identifier `validate_response` does not know.
            Self::UnknownExtension { .. } => "unknown-extension",
        }
    }
}

/// Build the request payload for an extension point, returning a JSON value
/// the walker can drop straight into the `llm-request.request` field.
///
/// Behavior by `identifier`:
///
/// - `writeCode` — builds [`WriteCodeRequest`] from the targeted feature's
///   `plan.md` (for `plan-relevant-files`), the command file's `Reference:`
///   line (for `constitution-excerpts`), the walker context's
///   `write-boundary`, and the current task pulled from `tasks.md` (using
///   `feature` + `task-number` from the walker context). Legacy context
///   fields are appended after the typed prefix for backward compatibility
///   with hosts that already parse them (walker-internal accumulator keys
///   filtered — see [`is_walker_internal_key`]).
/// - `writeSpecBody` — builds [`WriteSpecBodyRequest`]: the template the
///   running command fills (`/gov:plan` → the plan template, `/gov:specify`
///   → the spec template), the section named by the step prose, the
///   `feature-description` context key, and `existing-content` when the
///   named section already has body content in the file the running
///   command owns. Filtered legacy context fields follow the typed prefix.
/// - `performReview` — builds [`PerformReviewRequest`] (see
///   [`build_perform_review_request`]); filtered legacy context fields
///   follow the typed prefix, preserving the primitive results the pass
///   needs (`scope`/`diff-base`, `selected`/`rules-dir`/`notices`).
/// - `assessSpecQuality` — builds [`AssessSpecQualityRequest`] from the
///   walker context's `path` (the spec under review, read off disk for
///   `spec-content`) and the rule under assessment resolved from
///   `citations` / `rule-files`, with the severity tier taken from the
///   step prose (`MUST-tier` / `SHOULD-tier`). Emits the documented typed
///   shape only — the data model sanctions no legacy context dump here.
/// - `askClarifyQuestion` — builds [`AskClarifyQuestionRequest`] from the
///   spec resolved via `path`/`feature` and the question from the
///   `question` context value (falling back to the first merged
///   `open-questions` entry). Typed shape only.
/// - `routeInboxItem` — builds [`RouteInboxItemRequest`] from the
///   `item-text` context key, the fixed route vocabulary, and a scan of
///   the spec root for available features. Typed shape only.
/// - any other identifier — [`PayloadError::UnknownExtension`]; the raw
///   context dump fallback was removed by the extension-request-hygiene
///   scenario.
///
/// # Errors
///
/// Returns [`PayloadError::SecretExfiltration`] when any path in
/// `plan-relevant-files` matches a secret-bearing pattern or `.gitignore`,
/// and [`PayloadError::UnknownExtension`] for an identifier with no typed
/// builder.
pub fn build_extension_request(
    identifier: &str,
    context: &Map<String, Value>,
    repo: &Path,
    command_name: &str,
    step_prose: &str,
) -> Result<Value, PayloadError> {
    match identifier {
        "writeCode" => build_write_code_request(context, repo, command_name),
        "writeSpecBody" => Ok(build_write_spec_body_request(
            context,
            repo,
            command_name,
            step_prose,
        )),
        "performReview" => Ok(build_perform_review_request(context, repo)),
        "assessSpecQuality" => Ok(build_assess_spec_quality_request(context, repo, step_prose)),
        "askClarifyQuestion" => Ok(build_ask_clarify_question_request(context, repo)),
        "routeInboxItem" => Ok(build_route_inbox_item_request(context, repo)),
        other => Err(PayloadError::UnknownExtension {
            identifier: other.to_string(),
        }),
    }
}

/// `true` for walker-internal accumulator keys that never belong in an
/// outbound request: prior `llm:<identifier>` response echoes and the
/// cross-pass `findings` array the walker accumulates for `write-review`.
/// Primitive results threaded through the context (`scope`, `diff-base`,
/// `selected`, `rules-dir`, `notices`, …) are NOT accumulator state and
/// pass through the merge untouched.
fn is_walker_internal_key(key: &str) -> bool {
    key == "findings" || key.starts_with("llm:")
}

/// Append the legacy-compat context fields after a typed prefix, skipping
/// walker-internal accumulator keys ([`is_walker_internal_key`]) and any
/// key the typed prefix already emitted (typed values win).
fn merge_legacy_context(object: &mut Map<String, Value>, context: &Map<String, Value>) {
    for (key, value) in context {
        if is_walker_internal_key(key) {
            continue;
        }
        object.entry(key.clone()).or_insert_with(|| value.clone());
    }
}

/// Build the `performReview` request for one pass. Loads the in-scope files
/// (`scope`, from `compute-review-scope`) and the pass's rule files
/// (`selected` basenames under `rules-dir`, from `discover-rule-files`) off
/// disk, and pairs them with the `pass` name. Missing/unreadable files are
/// skipped rather than erroring — the pass reviews what it can read. The
/// typed prefix leads (cache-anchor order: `scope-files` is stable across
/// passes); legacy context fields follow for hosts that already parse them.
fn build_perform_review_request(context: &Map<String, Value>, repo: &Path) -> Value {
    let pass = context
        .get("pass")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let scope_files = load_scope_files(context, repo);
    let rule_files = load_rule_files(context, repo);

    let typed = PerformReviewRequest {
        scope_files,
        rule_files,
        pass,
    };
    let typed_value = serde_json::to_value(&typed).unwrap_or(Value::Null);
    let mut object = match typed_value {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    // Filtered merge: pass N must not see passes 1..N-1's accumulated
    // `findings` or raw `llm:performReview` echoes, but keeps the
    // primitive results it legitimately needs (`scope`/`diff-base` from
    // compute-review-scope, `selected`/`rules-dir`/`notices` from
    // discover-rule-files).
    merge_legacy_context(&mut object, context);
    Value::Object(object)
}

/// Classification of a repo-relative path against the canonicalized repo root
/// — the BE-INPUT-004 containment primitive shared by the scope, rule, and
/// plan file readers.
enum Contained {
    /// Canonical absolute path that stays inside the repo root.
    Inside(PathBuf),
    /// Path does not resolve to an existing file (`canonicalize` failed).
    Missing,
    /// Canonical path escapes the repo root — an absolute joinee, a `..`
    /// traversal, or a symlink whose target lands outside the repo.
    Outside,
}

/// Resolve `rel` against the already-canonicalized `canon_repo` and classify
/// whether its canonical form stays within the repo. Callers decide how to
/// treat an `Outside` path: the best-effort review readers skip it, while the
/// writeCode plan reader treats it as an exfiltration attempt and errors.
fn classify_contained(canon_repo: &Path, rel: &Path) -> Contained {
    match canon_repo.join(rel).canonicalize() {
        Ok(abs) if abs.starts_with(canon_repo) => Contained::Inside(abs),
        Ok(_) => Contained::Outside,
        Err(_) => Contained::Missing,
    }
}

/// Load `scope` paths (from `compute-review-scope`) into `ReviewScopeFile`
/// records, reading each file's content.
///
/// BE-INPUT-004: `scope` originates from plan-authored `## Affected Files`
/// entries (via `compute-review-scope`'s `read_plan_affected`), so each path
/// is canonicalized and confined to the repo root before it is opened — an
/// absolute or traversing entry is skipped, never read into the review
/// payload. Missing and unreadable paths are likewise skipped (best-effort).
fn load_scope_files(context: &Map<String, Value>, repo: &Path) -> Vec<ReviewScopeFile> {
    let Ok(canon_repo) = repo.canonicalize() else {
        return Vec::new();
    };
    string_array(context, "scope")
        .into_iter()
        .filter_map(
            |path| match classify_contained(&canon_repo, Path::new(&path)) {
                Contained::Inside(abs) => std::fs::read_to_string(&abs)
                    .ok()
                    .map(|content| ReviewScopeFile { path, content }),
                Contained::Missing | Contained::Outside => None,
            },
        )
        .collect()
}

/// Load the pass's `selected` rule basenames (from `discover-rule-files`)
/// under `rules-dir` into `ReviewRuleFile` records. Unreadable files are
/// skipped.
fn load_rule_files(context: &Map<String, Value>, repo: &Path) -> Vec<ReviewRuleFile> {
    let rules_dir = context
        .get("rules-dir")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let Ok(canon_repo) = repo.canonicalize() else {
        return Vec::new();
    };
    // `rules-dir` and `selected` come from `discover-rule-files` (a directory
    // walk of trusted basenames), but the same BE-INPUT-004 containment check
    // is applied defensively so the reader cannot escape the repo regardless.
    string_array(context, "selected")
        .into_iter()
        .filter_map(|name| {
            let rel = Path::new(rules_dir).join(&name);
            match classify_contained(&canon_repo, &rel) {
                Contained::Inside(abs) => std::fs::read_to_string(&abs)
                    .ok()
                    .map(|content| ReviewRuleFile { name, content }),
                Contained::Missing | Contained::Outside => None,
            }
        })
        .collect()
}

/// Read a context key as a `Vec<String>`, dropping non-string members.
/// Empty when the key is absent or not an array.
fn string_array(context: &Map<String, Value>, key: &str) -> Vec<String> {
    context
        .get(key)
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn build_write_code_request(
    context: &Map<String, Value>,
    repo: &Path,
    command_name: &str,
) -> Result<Value, PayloadError> {
    let feature = context
        .get("feature")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();

    let plan_relevant_files = load_plan_relevant_files(&feature, repo)?;
    let constitution_excerpts = load_constitution_excerpts(command_name, repo);
    let write_boundary = read_write_boundary(context);
    let task = load_current_task(&feature, context, repo);

    let typed = WriteCodeRequest {
        constitution_excerpts,
        plan_relevant_files,
        write_boundary,
        task,
    };

    // Merge: typed prefix first (declaration order = cache-anchor order),
    // then legacy context fields that hosts may already parse (filtered:
    // no `llm:*` echoes or accumulated `findings` — the whole-session dump
    // eroded the cache anchor). Keys present in both prefer the typed value.
    let typed_value = serde_json::to_value(&typed).unwrap_or(Value::Null);
    let mut object = match typed_value {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    merge_legacy_context(&mut object, context);
    Ok(Value::Object(object))
}

/// Build the `writeSpecBody` request. All documented typed fields are
/// populated from the walker context and disk (mirroring
/// `build_assess_spec_quality_request`'s "typed even when bare"
/// discipline — a field the run cannot derive is an empty string, never a
/// dropped key):
///
/// - `template-path` / `template-content` — the template the running
///   command fills, resolved by [`load_template`].
/// - `section` — the section heading named by the step prose; empty when
///   the step fills a whole body rather than one section (`/gov:specify`).
/// - `feature-description` — the `feature-description` context key the
///   host seeds from the slash command's `$ARGUMENTS`; empty when unset.
/// - `existing-content` — the section's current body in the file the
///   running command owns (plan.md for `/gov:plan`, spec.md for
///   `/gov:specify`); omitted when absent or empty.
///
/// Filtered legacy context fields follow the typed prefix for hosts that
/// already parse them.
fn build_write_spec_body_request(
    context: &Map<String, Value>,
    repo: &Path,
    command_name: &str,
    step_prose: &str,
) -> Value {
    let section = extract_section_name(step_prose).unwrap_or_default();
    let (template_path, template_content) = load_template(command_name, repo);
    let feature_description = context
        .get("feature-description")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let existing_content = if section.is_empty() {
        None
    } else {
        let feature = context.get("feature").and_then(Value::as_str);
        let path_hint = context.get("path").and_then(Value::as_str);
        read_existing_section(&section, feature, path_hint, repo, command_name)
    };
    let typed = WriteSpecBodyRequest {
        template_path,
        template_content,
        section,
        feature_description,
        existing_content,
    };
    let typed_value = serde_json::to_value(&typed).unwrap_or(Value::Null);
    let mut object = match typed_value {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    merge_legacy_context(&mut object, context);
    Value::Object(object)
}

/// Resolve the template file the running command fills. `/gov:plan` fills
/// plan sections from the plan template; `/gov:specify` fills the spec
/// body from the spec template. Candidates, in order: the installed
/// adopter layout `{specs-root}/templates/<file>` (what `/gov:init`
/// scaffolds and the command prose names), then the framework source
/// layout `framework/templates/spec/<file>` (the govern repo itself).
/// Returns `(repo-relative path, content)` for the first candidate on
/// disk, or empty strings when the command fills no template or none
/// exists.
fn load_template(command_name: &str, repo: &Path) -> (String, String) {
    let file = match command_name {
        "plan" => "plan.md",
        "specify" => "spec.md",
        _ => return (String::new(), String::new()),
    };
    let specs_root = crate::schema::paths::Paths::load(repo).specs_root;
    let candidates = [
        format!("{specs_root}/templates/{file}"),
        format!("framework/templates/spec/{file}"),
    ];
    for rel in candidates {
        if let Some(content) = read_repo_file(repo, &rel) {
            return (rel, content);
        }
    }
    (String::new(), String::new())
}

/// Build the `askClarifyQuestion` request (reserved by the
/// clarify-command-acceleration scenario). Typed shape only, mirroring
/// `assessSpecQuality` — no legacy context dump:
///
/// - `spec-path` / `spec-content` — the spec under clarification,
///   resolved by [`resolve_spec_path`] and read repo-confined
///   (BE-INPUT-004); content empty when missing.
/// - `question` — an explicit `question` context value when the walker
///   seeds one (string, or object with `text` / optional `section`),
///   falling back to the first entry of `read-spec`'s merged
///   `open-questions` result.
fn build_ask_clarify_question_request(context: &Map<String, Value>, repo: &Path) -> Value {
    let spec_path = resolve_spec_path(context, repo);
    let spec_content = read_repo_file(repo, &spec_path).unwrap_or_default();
    let question = resolve_clarify_question(context);
    let typed = AskClarifyQuestionRequest {
        spec_path,
        spec_content,
        question,
    };
    serde_json::to_value(&typed).unwrap_or(Value::Null)
}

/// Resolve the repo-relative path of the spec file the walker targets.
/// Preference order: an explicit `.md`-shaped `path` context value (the
/// analyze fixtures seed the spec file directly); `feature` joined under
/// the configured specs root; a directory-shaped `path` (the session
/// target's feature directory) joined with `spec.md`. Empty when the
/// context carries neither.
fn resolve_spec_path(context: &Map<String, Value>, repo: &Path) -> String {
    let path = context
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let is_markdown_file = Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("md"));
    if is_markdown_file {
        return path.to_string();
    }
    let feature = context
        .get("feature")
        .and_then(Value::as_str)
        .unwrap_or_default();
    if !feature.is_empty() {
        let specs_root = crate::schema::paths::Paths::load(repo).specs_root;
        return format!("{specs_root}/{feature}/spec.md");
    }
    if !path.is_empty() {
        return format!("{path}/spec.md");
    }
    String::new()
}

/// Resolve the question an `askClarifyQuestion` round trip carries. An
/// explicit `question` context value wins (a clarify walker loop seeds
/// one per round trip); otherwise the first entry of the merged
/// `open-questions` result. The typed shape is always present — an
/// unseeded context yields an empty question text.
fn resolve_clarify_question(context: &Map<String, Value>) -> ClarifyQuestion {
    if let Some(question) = context.get("question") {
        if let Some(text) = question.as_str() {
            return ClarifyQuestion {
                text: text.to_string(),
                section: None,
            };
        }
        if let Some(object) = question.as_object() {
            return ClarifyQuestion {
                text: object
                    .get("text")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_string(),
                section: object
                    .get("section")
                    .and_then(Value::as_str)
                    .map(str::to_string),
            };
        }
    }
    let text = context
        .get("open-questions")
        .and_then(Value::as_array)
        .and_then(|arr| arr.first())
        .and_then(|q| q.get("text"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    ClarifyQuestion {
        text,
        section: None,
    }
}

/// The groom decision tree's route vocabulary, in walk order (see the
/// groom-command-acceleration scenario: rule promotion → missing spec →
/// spec edit [`spec`] → scenario vs. chore → discard).
const INBOX_ROUTES: [&str; 5] = ["rule", "spec", "scenario", "chore", "discard"];

/// Build the `routeInboxItem` request (reserved by the
/// groom-command-acceleration scenario). Typed shape only — no legacy
/// context dump:
///
/// - `item-text` — the `item-text` context key (seeded per inbox item by
///   the groom walk); empty when unseeded.
/// - `routes` — the fixed [`INBOX_ROUTES`] vocabulary, so hosts need not
///   parse the command prose to learn the closed decision set.
/// - `available-specs` — `NNN-slug` directories under the spec root with
///   each spec's frontmatter `status` (status drives the
///   done → in-progress reopen consent on a scenario route).
fn build_route_inbox_item_request(context: &Map<String, Value>, repo: &Path) -> Value {
    let item_text = context
        .get("item-text")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let typed = RouteInboxItemRequest {
        item_text,
        routes: INBOX_ROUTES.iter().map(ToString::to_string).collect(),
        available_specs: load_available_specs(repo),
    };
    serde_json::to_value(&typed).unwrap_or(Value::Null)
}

/// Scan the configured spec root for `NNN-slug` feature directories and
/// read each spec's frontmatter `status`. Best-effort: an unreadable
/// directory yields an empty list, an unreadable or malformed `spec.md`
/// yields an empty status (the feature's existence still matters to the
/// router). Sorted by slug for deterministic payloads.
fn load_available_specs(repo: &Path) -> Vec<RouteInboxSpec> {
    let specs_dir = crate::schema::paths::specs_dir(repo);
    let Ok(read_dir) = std::fs::read_dir(&specs_dir) else {
        return Vec::new();
    };
    let mut slugs: Vec<String> = read_dir
        .filter_map(std::result::Result::ok)
        .filter(|entry| entry.path().is_dir())
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| crate::primitives::is_feature_slug(name))
        .collect();
    slugs.sort();
    slugs
        .into_iter()
        .map(|slug| {
            let status = read_spec_status(&specs_dir.join(&slug).join("spec.md"));
            RouteInboxSpec {
                feature: slug,
                status,
            }
        })
        .collect()
}

/// Read a spec file's frontmatter `status`, best-effort: empty string
/// when the file is missing or the frontmatter does not parse.
fn read_spec_status(spec_path: &Path) -> String {
    let Ok(content) = std::fs::read_to_string(spec_path) else {
        return String::new();
    };
    let Ok((fm_text, _body)) = crate::primitives::split_frontmatter(&content, spec_path) else {
        return String::new();
    };
    serde_norway::from_str::<Frontmatter>(fm_text)
        .map(|fm| fm.status)
        .unwrap_or_default()
}

/// Build the `assessSpecQuality` request for one per-rule Verification
/// read (`/gov:analyze` steps 8–9). Mirrors `build_write_code_request`'s
/// structure: typed fields sourced from the walker context and disk.
///
/// - `spec-path` — the context's `path` (seeded by `/gov:target`, echoed
///   by `read-spec`).
/// - `spec-content` — the spec read off disk, repo-confined
///   (BE-INPUT-004); empty when missing or out of repo.
/// - `rule` — the rule under assessment: the first `citations` entry
///   (from `check-rule-ids`) that is found and not deprecated, falling
///   back to the first rule defined in the loaded `rule-files`. Its
///   `**Verification:**` phrase is extracted from the rule file; the
///   severity tier comes from the step prose (`MUST-tier` → `must`,
///   `SHOULD-tier` → `should`).
///
/// Unlike `writeCode`, no legacy context fields are appended: the data
/// model documents the bare typed shape for this point, and the previous
/// raw walker-context dump is exactly the behavior this builder replaces.
fn build_assess_spec_quality_request(
    context: &Map<String, Value>,
    repo: &Path,
    step_prose: &str,
) -> Value {
    let spec_path = context
        .get("path")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    let spec_content = read_repo_file(repo, &spec_path).unwrap_or_default();
    let severity = severity_from_step_prose(step_prose);
    let rule = resolve_assessed_rule(context, repo, severity);
    let typed = AssessSpecQualityRequest {
        spec_path,
        spec_content,
        rule,
    };
    serde_json::to_value(&typed).unwrap_or(Value::Null)
}

/// Read a repo-relative file with the BE-INPUT-004 containment check.
/// `None` when `rel` is empty, the repo cannot be canonicalized, the path
/// escapes the repo, or the file is missing/unreadable.
fn read_repo_file(repo: &Path, rel: &str) -> Option<String> {
    if rel.is_empty() {
        return None;
    }
    let canon_repo = repo.canonicalize().ok()?;
    match classify_contained(&canon_repo, Path::new(rel)) {
        Contained::Inside(abs) => std::fs::read_to_string(abs).ok(),
        Contained::Missing | Contained::Outside => None,
    }
}

/// Map the step prose's rule-tier phrase to the request severity:
/// `MUST-tier` → `must`, `SHOULD-tier` → `should`, `INFO-tier` → `info`
/// (case-insensitive). Empty when the prose names no tier.
fn severity_from_step_prose(prose: &str) -> String {
    let lower = prose.to_lowercase();
    for tier in ["must", "should", "info"] {
        if lower.contains(&format!("{tier}-tier")) {
            return tier.to_string();
        }
    }
    String::new()
}

/// Resolve the rule an `assessSpecQuality` request assesses. Preference
/// order: the first cited rule (`citations`, found and not deprecated)
/// whose definition and Verification phrase resolve in the loaded
/// `rule-files`; then the first rule defined in those files; then a
/// placeholder carrying the first cited ID (empty when none) so the typed
/// shape is always present.
fn resolve_assessed_rule(
    context: &Map<String, Value>,
    repo: &Path,
    severity: String,
) -> AssessSpecQualityRule {
    let cited: Vec<String> = context
        .get("citations")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter(|c| {
                    c.get("found").and_then(Value::as_bool).unwrap_or(false)
                        && !c
                            .get("deprecated")
                            .and_then(Value::as_bool)
                            .unwrap_or(false)
                })
                .filter_map(|c| c.get("rule-id").and_then(Value::as_str).map(str::to_string))
                .collect()
        })
        .unwrap_or_default();
    let contents: Vec<String> = string_array(context, "rule-files")
        .iter()
        .filter_map(|rel| read_repo_file(repo, rel))
        .collect();
    for id in &cited {
        for content in &contents {
            if let Some(verification) = extract_rule_verification(content, id) {
                return AssessSpecQualityRule {
                    id: id.clone(),
                    verification,
                    severity,
                };
            }
        }
    }
    for content in &contents {
        if let Some((id, verification)) = first_rule_with_verification(content) {
            return AssessSpecQualityRule {
                id,
                verification,
                severity,
            };
        }
    }
    AssessSpecQualityRule {
        id: cited.first().cloned().unwrap_or_default(),
        verification: String::new(),
        severity,
    }
}

/// Extract the `**Verification:**` phrase for `rule_id` from a rule file.
/// Rule sections open with a level-3 heading holding only the ID
/// (`### CFG-CONST-001`); the Verification field is a paragraph starting
/// `**Verification:**` whose text may wrap across lines. Wrapped lines are
/// joined with single spaces. `None` when the rule or its Verification
/// field is absent or empty.
fn extract_rule_verification(content: &str, rule_id: &str) -> Option<String> {
    let mut in_rule = false;
    let mut collecting: Option<Vec<&str>> = None;
    for line in content.lines() {
        let trimmed = line.trim();
        // A heading of any level ends the current rule section.
        if trimmed.starts_with('#') {
            if in_rule {
                break;
            }
            in_rule = trimmed.strip_prefix("### ").map(str::trim) == Some(rule_id);
            continue;
        }
        if !in_rule {
            continue;
        }
        if let Some(acc) = collecting.as_mut() {
            // The paragraph ends at a blank line or the next `**Field:**`.
            if trimmed.is_empty() || trimmed.starts_with("**") {
                break;
            }
            acc.push(trimmed);
        } else if let Some(rest) = trimmed.strip_prefix("**Verification:**") {
            collecting = Some(vec![rest.trim()]);
        }
    }
    collecting
        .map(|parts| parts.join(" ").trim().to_string())
        .filter(|phrase| !phrase.is_empty())
}

/// First rule in a rule file (in heading order) whose Verification phrase
/// resolves, as an `(id, verification)` pair.
fn first_rule_with_verification(content: &str) -> Option<(String, String)> {
    for line in content.lines() {
        if let Some(id) = line.trim().strip_prefix("### ") {
            let id = id.trim();
            if let Some(verification) = extract_rule_verification(content, id) {
                return Some((id.to_string(), verification));
            }
        }
    }
    None
}

fn read_write_boundary(context: &Map<String, Value>) -> Vec<String> {
    context
        .get("write-boundary")
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

fn load_current_task(feature: &str, context: &Map<String, Value>, repo: &Path) -> WriteCodeTask {
    let task_number = context
        .get("task-number")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if feature.is_empty() {
        return WriteCodeTask {
            number: task_number,
            heading: String::new(),
            subtasks: Vec::new(),
        };
    }
    let args = ReadTasksArgs {
        feature: feature.to_string(),
    };
    let Ok(result) = read_tasks::run(&args, repo) else {
        return WriteCodeTask {
            number: task_number,
            heading: String::new(),
            subtasks: Vec::new(),
        };
    };
    // Locate by explicit task-number; fall back to the first incomplete
    // task when the context did not seed one.
    let task = if task_number.is_empty() {
        result
            .tasks
            .iter()
            .find(|t| t.subtasks.iter().any(|s| !s.checked))
            .or_else(|| result.tasks.first())
    } else {
        result.tasks.iter().find(|t| t.number == task_number)
    };
    match task {
        Some(t) => WriteCodeTask {
            number: t.number.clone(),
            heading: t.heading.clone(),
            subtasks: t.subtasks.iter().map(|s| s.text.clone()).collect(),
        },
        None => WriteCodeTask {
            number: task_number,
            heading: String::new(),
            subtasks: Vec::new(),
        },
    }
}

fn load_plan_relevant_files(
    feature: &str,
    repo: &Path,
) -> Result<Vec<PlanRelevantFile>, PayloadError> {
    if feature.is_empty() {
        return Ok(Vec::new());
    }
    // Canonicalize repo once so the containment check below operates on the
    // resolved form (e.g., macOS `/var/folders/...` → `/private/var/...`).
    // A non-canonicalizable repo path mirrors the "no plan, no files" posture.
    let Ok(canon_repo) = repo.canonicalize() else {
        return Ok(Vec::new());
    };
    let plan_path = crate::schema::paths::specs_dir(&canon_repo)
        .join(feature)
        .join("plan.md");
    let Ok(plan_content) = std::fs::read_to_string(&plan_path) else {
        return Ok(Vec::new());
    };
    let paths = crate::primitives::parse_affected_files(&plan_content);
    let mut out = Vec::new();
    for rel in paths {
        if let Some(pattern) = secret_pattern(&rel) {
            return Err(PayloadError::SecretExfiltration {
                path: rel,
                pattern: pattern.into(),
            });
        }
        if is_gitignored(&canon_repo, &rel) {
            return Err(PayloadError::SecretExfiltration {
                path: rel,
                pattern: ".gitignore".into(),
            });
        }
        let canon_abs = match classify_contained(&canon_repo, Path::new(&rel)) {
            // Planned-new file or rename target — omit, don't error.
            // (`canonicalize` errors on missing files; existing behavior
            // preserved.)
            Contained::Missing => continue,
            // Path traversal: `../foo`, absolute path, or symlink whose
            // canonical target escapes the repo root. BE-INPUT-004
            // defense-in-depth — the basename-only secret-pattern check
            // above doesn't catch this class.
            Contained::Outside => {
                return Err(PayloadError::SecretExfiltration {
                    path: rel,
                    pattern: "out-of-repo".into(),
                });
            }
            Contained::Inside(abs) => abs,
        };
        let Ok(content) = std::fs::read_to_string(&canon_abs) else {
            continue;
        };
        out.push(PlanRelevantFile { path: rel, content });
    }
    Ok(out)
}

fn load_constitution_excerpts(command_name: &str, repo: &Path) -> Vec<String> {
    let Some(command_path) = locate_command_file(command_name, repo) else {
        return Vec::new();
    };
    let Ok(command_content) = std::fs::read_to_string(&command_path) else {
        return Vec::new();
    };
    let anchors = parse_command_references(&command_content);
    if anchors.is_empty() {
        return Vec::new();
    }
    let constitution_path = repo.join("framework/constitution.md");
    let Ok(constitution) = std::fs::read_to_string(&constitution_path) else {
        return Vec::new();
    };
    anchors
        .into_iter()
        .filter_map(|anchor| extract_anchor_body(&constitution, &anchor))
        .collect()
}

fn locate_command_file(command_name: &str, repo: &Path) -> Option<PathBuf> {
    let host = Host::load(repo);
    let mut rels = vec![format!("framework/commands/{command_name}.md")];
    // Installed command file — `commands/` (claude-style) or singular
    // `command/` (opencode); see `Host::command_file_candidates`.
    rels.extend(host.command_file_candidates(command_name));
    rels.push(format!("framework/bootstrap/{command_name}.md"));
    for rel in rels {
        let candidate = repo.join(rel);
        if candidate.is_file() {
            return Some(candidate);
        }
    }
    None
}

/// Extract anchor names from a command file's `Reference: §a, §b, §c` line
/// under the `Scope Boundaries` section. The line may carry trailing
/// parenthetical prose; only `§<name>` tokens are returned.
///
/// # Panics
///
/// Panics only if the hard-coded anchor regex fails to compile — which
/// would indicate a corrupt `regex` crate, not user input.
#[must_use]
pub fn parse_command_references(command_content: &str) -> Vec<String> {
    static R: OnceLock<Regex> = OnceLock::new();
    let anchor_re = R.get_or_init(|| {
        Regex::new(r"§([A-Za-z][A-Za-z0-9_-]*)").expect("hard-coded regex compiles")
    });
    for line in command_content.lines() {
        let trimmed = line.trim_start_matches(|c: char| c == '-' || c.is_whitespace());
        if !trimmed.starts_with("Reference:") {
            continue;
        }
        let mut anchors: Vec<String> = anchor_re
            .captures_iter(trimmed)
            .map(|c| c[1].to_string())
            .collect();
        anchors.dedup();
        return anchors;
    }
    Vec::new()
}

/// Return the body of an anchored section. The body is the content between
/// `<!-- §<anchor> -->` and the next `<!-- §<other> -->` marker (or EOF),
/// with the marker line itself excluded. Returns `None` when the anchor is
/// not present in `content`.
///
/// # Panics
///
/// Panics only if the hard-coded next-marker regex fails to compile —
/// which would indicate a corrupt `regex` crate, not user input.
#[must_use]
pub fn extract_anchor_body(content: &str, anchor: &str) -> Option<String> {
    static NEXT: OnceLock<Regex> = OnceLock::new();
    let marker = format!("<!-- §{anchor} -->");
    let start = content.find(&marker)?;
    // Skip past the marker line itself (find end of that line).
    let after_marker_line = {
        let rel = content[start..].find('\n')?;
        start + rel + 1
    };
    let rest = &content[after_marker_line..];
    // Find the next anchor marker (any name) and cut there.
    let next_re = NEXT.get_or_init(|| {
        Regex::new(r"<!--\s*§[A-Za-z][A-Za-z0-9_-]*\s*-->").expect("hard-coded regex compiles")
    });
    let end = match next_re.find(rest) {
        Some(m) => m.start(),
        None => rest.len(),
    };
    Some(rest[..end].trim_end_matches('\n').to_string())
}

/// Read a section body from a spec or plan file. The running command
/// selects the file explicitly (extension-request-hygiene — a
/// `/gov:specify` re-run on a feature that has since gained a plan must
/// not read plan.md's section):
///
/// - `plan` (`/gov:plan`) → `specs/{feature}/plan.md`
/// - `specify` (`/gov:specify`) → `specs/{feature}/spec.md`
/// - any other command → the historical plan.md-then-spec.md fallback
///   order (no other command invokes `writeSpecBody` today).
///
/// Returns `None` when the file does not exist or the section is absent or
/// empty. Whitespace-only bodies count as empty.
fn read_existing_section(
    section: &str,
    feature: Option<&str>,
    path_hint: Option<&str>,
    repo: &Path,
    command_name: &str,
) -> Option<String> {
    let feature_dir = match feature {
        Some(f) if !f.is_empty() => crate::schema::paths::specs_dir(repo).join(f),
        _ => repo.join(path_hint.unwrap_or_default()),
    };
    let filenames: &[&str] = match command_name {
        "plan" => &["plan.md"],
        "specify" => &["spec.md"],
        _ => &["plan.md", "spec.md"],
    };
    for filename in filenames {
        let candidate = feature_dir.join(filename);
        let Ok(content) = std::fs::read_to_string(&candidate) else {
            continue;
        };
        if let Some(body) = extract_section_body(&content, section) {
            let trimmed = body.trim();
            if trimmed.is_empty() {
                return None;
            }
            return Some(trimmed.to_string());
        }
    }
    None
}

/// Pull a level-2 (`## …`) section body from a markdown file. The body runs
/// from after the heading line to the next level-1 or level-2 heading (or
/// EOF). Fenced code blocks inside the body do not terminate it.
fn extract_section_body(content: &str, section: &str) -> Option<String> {
    let mut in_fence = false;
    let mut collected: Option<Vec<&str>> = None;
    for line in content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            if let Some(acc) = collected.as_mut() {
                acc.push(line);
            }
            continue;
        }
        if !in_fence {
            if let Some(rest) = trimmed.strip_prefix("## ") {
                if collected.is_some() {
                    break;
                }
                if rest.trim().eq_ignore_ascii_case(section) {
                    collected = Some(Vec::new());
                    continue;
                }
            } else if trimmed.starts_with("# ")
                && !trimmed.starts_with("## ")
                && collected.is_some()
            {
                break;
            }
        }
        if let Some(acc) = collected.as_mut() {
            acc.push(line);
        }
    }
    collected.map(|lines| lines.join("\n"))
}

/// Extract the section name from a step prose like
/// `Fill the Technical Decisions section of the plan.` Returns `None` when
/// no such phrase is present. The name charset is restricted to plain
/// heading words (`[A-Za-z0-9 _/-]`) so a whole-body fill step whose prose
/// happens to mention "… section" much later — `/gov:specify`'s "Fill the
/// new spec body following §spec-requirements: a Motivation section …" —
/// does not smuggle intervening punctuation into the typed `section` field.
fn extract_section_name(prose: &str) -> Option<String> {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| {
        Regex::new(r"(?i)Fill\s+the\s+([A-Za-z][A-Za-z0-9 _/-]*?)\s+section")
            .expect("hard-coded regex compiles")
    });
    re.captures(prose).map(|c| c[1].trim().to_string())
}

/// Match a path against the v1 secret-exfiltration patterns. Returns the
/// matched pattern label when blocked; `None` otherwise. Matching is
/// ASCII-case-insensitive on the basename so a plan entry of `.ENV` or
/// `Credentials.json` cannot bypass the guard on a case-insensitive
/// filesystem (macOS APFS by default). Patterns:
///
/// - `.env` and `.env.*` (e.g., `.env.production`)
/// - `*-secrets.*` (e.g., `db-secrets.yaml`)
/// - `credentials*` (any extension)
fn secret_pattern(path: &str) -> Option<&'static str> {
    let basename = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    if basename == ".env" {
        return Some(".env");
    }
    if basename.starts_with(".env.") {
        return Some(".env.*");
    }
    if basename.starts_with("credentials") {
        return Some("credentials*");
    }
    // *-secrets.* — split into stem and extension; the stem must end with
    // `-secrets` and there must be at least one extension.
    if let Some((stem, _ext)) = basename.rsplit_once('.')
        && stem.ends_with("-secrets")
    {
        return Some("*-secrets.*");
    }
    None
}

/// Ask libgit2 whether `path` is gitignored from `repo`'s perspective.
/// Returns `false` when the directory isn't a git repo or libgit2 errors
/// — the secret-pattern check above is the floor; gitignore is an opt-in
/// second layer.
fn is_gitignored(repo: &Path, path: &str) -> bool {
    use git2::Repository;
    let Ok(repository) = Repository::discover(repo) else {
        return false;
    };
    repository
        .status_should_ignore(Path::new(path))
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn parse_command_references_extracts_anchor_names() {
        let cmd = "## Scope Boundaries\n\n\
                   - The runtime write boundary is derived in step 2.\n\
                   - Do NOT read source code speculatively.\n\
                   - Reference: §implement-phase, §pipeline-boundaries, §text-first-artifacts, plus extras.\n";
        let anchors = parse_command_references(cmd);
        assert_eq!(
            anchors,
            vec![
                "implement-phase".to_string(),
                "pipeline-boundaries".to_string(),
                "text-first-artifacts".to_string()
            ]
        );
    }

    #[test]
    fn parse_command_references_empty_when_absent() {
        let cmd = "## Scope Boundaries\n\nNo reference line here.\n";
        assert!(parse_command_references(cmd).is_empty());
    }

    #[test]
    fn extract_anchor_body_returns_section_between_markers() {
        let constitution = "<!-- §alpha -->\n\
                            ### Alpha\n\nBody of alpha.\n\n\
                            <!-- §beta -->\n\
                            ### Beta\n\nBody of beta.\n";
        let alpha = extract_anchor_body(constitution, "alpha").unwrap();
        assert!(alpha.contains("Body of alpha."));
        assert!(!alpha.contains("Body of beta."));
        assert!(!alpha.contains("<!-- §beta -->"));
    }

    #[test]
    fn extract_anchor_body_reads_until_eof_for_last_marker() {
        let content = "<!-- §only -->\n\nfinal body content\n";
        let body = extract_anchor_body(content, "only").unwrap();
        assert_eq!(body.trim(), "final body content");
    }

    #[test]
    fn extract_anchor_body_returns_none_when_anchor_missing() {
        let content = "<!-- §other -->\nbody\n";
        assert!(extract_anchor_body(content, "missing").is_none());
    }

    #[test]
    fn secret_pattern_matches_dotenv_family() {
        assert_eq!(secret_pattern(".env"), Some(".env"));
        assert_eq!(secret_pattern(".env.production"), Some(".env.*"));
        assert_eq!(secret_pattern("path/to/.env.local"), Some(".env.*"));
    }

    #[test]
    fn secret_pattern_matches_secrets_files() {
        assert_eq!(secret_pattern("db-secrets.yaml"), Some("*-secrets.*"));
        assert_eq!(
            secret_pattern("path/to/api-secrets.json"),
            Some("*-secrets.*")
        );
    }

    #[test]
    fn secret_pattern_matches_credentials_files() {
        assert_eq!(secret_pattern("credentials"), Some("credentials*"));
        assert_eq!(secret_pattern("credentials.json"), Some("credentials*"));
        assert_eq!(
            secret_pattern("path/to/credentials.gpg"),
            Some("credentials*")
        );
    }

    #[test]
    fn secret_pattern_passes_through_normal_files() {
        assert_eq!(secret_pattern("runtime/src/main.rs"), None);
        assert_eq!(secret_pattern("README.md"), None);
        assert_eq!(secret_pattern("framework/constitution.md"), None);
    }

    #[test]
    fn load_plan_relevant_files_omits_absent_paths_without_error() {
        let tmp = tempdir().unwrap();
        let feature_dir = tmp.path().join("specs/123-foo");
        fs::create_dir_all(&feature_dir).unwrap();
        fs::write(
            feature_dir.join("plan.md"),
            "## Affected Files\n\n\
             | File | Action |\n| --- | --- |\n\
             | `existing.txt` | Edit |\n\
             | `planned-but-absent.txt` | Create |\n",
        )
        .unwrap();
        fs::write(tmp.path().join("existing.txt"), "hello").unwrap();

        let files = load_plan_relevant_files("123-foo", tmp.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "existing.txt");
        assert_eq!(files[0].content, "hello");
    }

    #[test]
    fn load_plan_relevant_files_rejects_secret_pattern() {
        let tmp = tempdir().unwrap();
        let feature_dir = tmp.path().join("specs/123-foo");
        fs::create_dir_all(&feature_dir).unwrap();
        fs::write(
            feature_dir.join("plan.md"),
            "## Affected Files\n\n\
             | File | Action |\n| --- | --- |\n\
             | `.env.production` | Edit |\n",
        )
        .unwrap();

        let err = load_plan_relevant_files("123-foo", tmp.path()).unwrap_err();
        match err {
            PayloadError::SecretExfiltration { path, pattern } => {
                assert_eq!(path, ".env.production");
                assert_eq!(pattern, ".env.*");
            }
            other @ PayloadError::UnknownExtension { .. } => {
                panic!("expected SecretExfiltration, got {other:?}")
            }
        }
    }

    #[test]
    fn load_plan_relevant_files_rejects_gitignored_path() {
        let tmp = tempdir().unwrap();
        // Init a git repo so libgit2 can answer the gitignore query.
        let repo = git2::Repository::init(tmp.path()).unwrap();
        let _ = repo; // dropped — `discover` reopens later
        let feature_dir = tmp.path().join("specs/123-foo");
        fs::create_dir_all(&feature_dir).unwrap();
        fs::write(tmp.path().join(".gitignore"), "secret-config.toml\n").unwrap();
        fs::write(
            feature_dir.join("plan.md"),
            "## Affected Files\n\n\
             | File | Action |\n| --- | --- |\n\
             | `secret-config.toml` | Edit |\n",
        )
        .unwrap();
        fs::write(tmp.path().join("secret-config.toml"), "key=value").unwrap();

        let err = load_plan_relevant_files("123-foo", tmp.path()).unwrap_err();
        match err {
            PayloadError::SecretExfiltration { path, pattern } => {
                assert_eq!(path, "secret-config.toml");
                assert_eq!(pattern, ".gitignore");
            }
            other @ PayloadError::UnknownExtension { .. } => {
                panic!("expected SecretExfiltration, got {other:?}")
            }
        }
    }

    #[test]
    fn secret_pattern_is_case_insensitive() {
        // BE-INPUT-004 — case-fold bypass on case-insensitive filesystems.
        // `.ENV` on macOS APFS resolves to `.env` on disk; the basename
        // check must match regardless of case.
        assert_eq!(secret_pattern(".ENV"), Some(".env"));
        assert_eq!(secret_pattern(".Env.Production"), Some(".env.*"));
        assert_eq!(secret_pattern("Credentials.JSON"), Some("credentials*"));
        assert_eq!(secret_pattern("DB-Secrets.YAML"), Some("*-secrets.*"));
        assert_eq!(secret_pattern("README.md"), None);
    }

    #[test]
    fn load_plan_relevant_files_rejects_relative_escape() {
        // BE-INPUT-004 — a plan entry of `../outside.txt` resolves outside
        // the repo. Basename `outside.txt` does not match any secret pattern,
        // but the canonical-containment check catches it.
        let outer = tempdir().unwrap();
        let repo = outer.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        std::fs::write(outer.path().join("outside.txt"), "leaked").unwrap();
        let feature_dir = repo.join("specs/123-foo");
        std::fs::create_dir_all(&feature_dir).unwrap();
        std::fs::write(
            feature_dir.join("plan.md"),
            "## Affected Files\n\n\
             | File | Action |\n| --- | --- |\n\
             | `../outside.txt` | Edit |\n",
        )
        .unwrap();

        let err = load_plan_relevant_files("123-foo", &repo).unwrap_err();
        match err {
            PayloadError::SecretExfiltration { path, pattern } => {
                assert_eq!(path, "../outside.txt");
                assert_eq!(pattern, "out-of-repo");
            }
            other @ PayloadError::UnknownExtension { .. } => {
                panic!("expected SecretExfiltration, got {other:?}")
            }
        }
    }

    #[test]
    fn load_plan_relevant_files_rejects_absolute_escape() {
        // BE-INPUT-004 — `Path::join` lets an absolute joinee replace the
        // base, so `/etc/hosts` (or a sibling tempdir absolute path) would
        // be read without the containment check. Use a sibling tempdir
        // instead of /etc/hosts so the test is hermetic.
        let outer = tempdir().unwrap();
        let repo = outer.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        let sibling = outer.path().join("sibling.txt");
        std::fs::write(&sibling, "leaked").unwrap();
        // Resolve to the canonical absolute form so the test is robust
        // against tempdir symlinks (macOS `/var` → `/private/var`).
        let abs_str = sibling
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let feature_dir = repo.join("specs/123-foo");
        std::fs::create_dir_all(&feature_dir).unwrap();
        std::fs::write(
            feature_dir.join("plan.md"),
            format!(
                "## Affected Files\n\n\
                 | File | Action |\n| --- | --- |\n\
                 | `{abs_str}` | Edit |\n"
            ),
        )
        .unwrap();

        let err = load_plan_relevant_files("123-foo", &repo).unwrap_err();
        match err {
            PayloadError::SecretExfiltration { path, pattern } => {
                assert_eq!(path, abs_str);
                assert_eq!(pattern, "out-of-repo");
            }
            other @ PayloadError::UnknownExtension { .. } => {
                panic!("expected SecretExfiltration, got {other:?}")
            }
        }
    }

    #[test]
    fn load_plan_relevant_files_admits_in_repo_relative_path() {
        // Happy path — a normal in-repo relative entry resolves under
        // canon_repo and is bundled into the payload as today.
        let tmp = tempdir().unwrap();
        let feature_dir = tmp.path().join("specs/123-foo");
        std::fs::create_dir_all(&feature_dir).unwrap();
        std::fs::create_dir_all(tmp.path().join("src")).unwrap();
        std::fs::write(tmp.path().join("src/lib.rs"), "fn main() {}").unwrap();
        std::fs::write(
            feature_dir.join("plan.md"),
            "## Affected Files\n\n\
             | File | Action |\n| --- | --- |\n\
             | `src/lib.rs` | Edit |\n",
        )
        .unwrap();

        let files = load_plan_relevant_files("123-foo", tmp.path()).unwrap();
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].path, "src/lib.rs");
        assert_eq!(files[0].content, "fn main() {}");
    }

    #[test]
    fn load_plan_relevant_files_skips_case_fold_bypass() {
        // BE-INPUT-004 — `.ENV` lowercased to `.env` matches the pattern
        // and is rejected before the containment check runs. Important on
        // case-insensitive filesystems where the on-disk file is `.env`
        // but the plan author can spell it `.ENV` (or `.Env`) to bypass.
        let tmp = tempdir().unwrap();
        let feature_dir = tmp.path().join("specs/123-foo");
        std::fs::create_dir_all(&feature_dir).unwrap();
        std::fs::write(
            feature_dir.join("plan.md"),
            "## Affected Files\n\n\
             | File | Action |\n| --- | --- |\n\
             | `.ENV` | Edit |\n",
        )
        .unwrap();

        let err = load_plan_relevant_files("123-foo", tmp.path()).unwrap_err();
        match err {
            PayloadError::SecretExfiltration { path, pattern } => {
                assert_eq!(path, ".ENV");
                // Pattern label is the canonical lowercase form regardless
                // of which case the author spelled.
                assert_eq!(pattern, ".env");
            }
            other @ PayloadError::UnknownExtension { .. } => {
                panic!("expected SecretExfiltration, got {other:?}")
            }
        }
    }

    #[test]
    fn extract_section_name_pulls_section_from_step_prose() {
        let prose = "Fill the Technical Decisions section of the plan. The host returns the markdown body for the section; the walker forwards the response through the context.";
        assert_eq!(
            extract_section_name(prose).as_deref(),
            Some("Technical Decisions")
        );
    }

    #[test]
    fn extract_section_name_returns_none_when_phrase_absent() {
        let prose = "Do the thing. No section here.";
        assert!(extract_section_name(prose).is_none());
    }

    #[test]
    fn extract_section_body_pulls_body_until_next_h2() {
        let plan = "# Title\n\n\
                    ## Motivation\n\nWhy.\n\n\
                    ## Technical Decisions\n\nFirst decision.\n\nSecond decision.\n\n\
                    ## Affected Files\n\n| File | Action |\n";
        let body = extract_section_body(plan, "Technical Decisions").unwrap();
        assert!(body.contains("First decision."));
        assert!(body.contains("Second decision."));
        assert!(!body.contains("## Affected Files"));
    }

    #[test]
    fn build_write_spec_body_request_inlines_existing_section_content() {
        let tmp = tempdir().unwrap();
        let feature_dir = tmp.path().join("specs/123-foo");
        fs::create_dir_all(&feature_dir).unwrap();
        fs::write(
            feature_dir.join("plan.md"),
            "# Plan\n\n\
             ## Technical Decisions\n\n\
             Use the standard library.\n\n\
             ## Affected Files\n\n| File | Action |\n",
        )
        .unwrap();

        let mut ctx = Map::new();
        ctx.insert("feature".into(), Value::String("123-foo".into()));
        let prose = "Fill the Technical Decisions section of the plan.";
        let value = build_write_spec_body_request(&ctx, tmp.path(), "plan", prose);
        let obj = value.as_object().unwrap();
        assert_eq!(obj["section"], "Technical Decisions");
        assert_eq!(
            obj["existing-content"].as_str().unwrap(),
            "Use the standard library."
        );
        // The typed prefix leads in declaration order even when the
        // template fields could not be derived (no template on disk).
        let prefix: Vec<&str> = obj.keys().map(String::as_str).take(4).collect();
        assert_eq!(
            prefix,
            vec![
                "template-path",
                "template-content",
                "section",
                "feature-description"
            ]
        );
        assert_eq!(obj["template-path"], "");
    }

    #[test]
    fn build_write_spec_body_request_stays_typed_when_no_existing_content() {
        let tmp = tempdir().unwrap();
        let mut ctx = Map::new();
        ctx.insert("feature".into(), Value::String("999-empty".into()));
        let prose = "Fill the Motivation section of the spec.";
        let value = build_write_spec_body_request(&ctx, tmp.path(), "specify", prose);
        let obj = value.as_object().unwrap();
        // No existing content found → no `existing-content` key added.
        assert!(!obj.contains_key("existing-content"));
        // Typed fields are still emitted (empty, not dropped).
        assert_eq!(obj["section"], "Motivation");
        assert_eq!(obj["feature-description"], "");
        // Context fields are preserved after the typed prefix.
        assert_eq!(obj["feature"], "999-empty");
    }

    #[test]
    fn build_write_spec_body_request_selects_file_by_running_command() {
        // A /gov:specify re-run on a feature that has since gained a plan
        // must read spec.md's section, not plan.md's (and vice versa).
        let tmp = tempdir().unwrap();
        let feature_dir = tmp.path().join("specs/123-foo");
        fs::create_dir_all(&feature_dir).unwrap();
        fs::write(
            feature_dir.join("plan.md"),
            "# Plan\n\n## Motivation\n\nPlan motivation.\n",
        )
        .unwrap();
        fs::write(
            feature_dir.join("spec.md"),
            "# Spec\n\n## Motivation\n\nSpec motivation.\n",
        )
        .unwrap();
        let mut ctx = Map::new();
        ctx.insert("feature".into(), Value::String("123-foo".into()));
        let prose = "Fill the Motivation section of the file.";
        let plan = build_write_spec_body_request(&ctx, tmp.path(), "plan", prose);
        assert_eq!(plan["existing-content"], "Plan motivation.");
        let spec = build_write_spec_body_request(&ctx, tmp.path(), "specify", prose);
        assert_eq!(spec["existing-content"], "Spec motivation.");
    }

    #[test]
    fn build_write_spec_body_request_populates_template_and_description() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("specs/templates")).unwrap();
        fs::write(
            tmp.path().join("specs/templates/plan.md"),
            "# Plan template\n",
        )
        .unwrap();
        let mut ctx = Map::new();
        ctx.insert(
            "feature-description".into(),
            Value::String("webhook delivery".into()),
        );
        let prose = "Fill the Technical Decisions section of the plan.";
        let value = build_write_spec_body_request(&ctx, tmp.path(), "plan", prose);
        assert_eq!(value["template-path"], "specs/templates/plan.md");
        assert_eq!(value["template-content"], "# Plan template\n");
        assert_eq!(value["feature-description"], "webhook delivery");
        assert_eq!(value["section"], "Technical Decisions");
    }

    #[test]
    fn load_template_falls_back_to_framework_source_layout() {
        // No installed {specs-root}/templates/ → the framework source
        // layout (the govern repo itself) is the second candidate.
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("framework/templates/spec")).unwrap();
        fs::write(
            tmp.path().join("framework/templates/spec/spec.md"),
            "# Spec template\n",
        )
        .unwrap();
        let (path, content) = load_template("specify", tmp.path());
        assert_eq!(path, "framework/templates/spec/spec.md");
        assert_eq!(content, "# Spec template\n");
        // A command that fills no template resolves nothing.
        assert_eq!(
            load_template("review", tmp.path()),
            (String::new(), String::new())
        );
    }

    #[test]
    fn extract_section_name_ignores_whole_body_fill_prose() {
        // /gov:specify's step 2 fills the whole spec body; its prose
        // mentions "a Motivation section" behind punctuation the heading
        // charset excludes, so no section is extracted.
        let prose = "Fill the new spec body following §spec-requirements: a \
                     Motivation section, Acceptance Criteria with concrete and \
                     testable checkboxes, and Open Questions.";
        assert!(extract_section_name(prose).is_none());
    }

    #[test]
    fn build_write_code_request_emits_typed_prefix_in_declaration_order() {
        let tmp = tempdir().unwrap();
        let feature_dir = tmp.path().join("specs/123-foo");
        fs::create_dir_all(&feature_dir).unwrap();
        fs::write(
            feature_dir.join("tasks.md"),
            "# Tasks\n\n## 1. Stub a module\n\n- [ ] Create stub\n- **Done when**: file exists.\n",
        )
        .unwrap();

        let mut ctx = Map::new();
        ctx.insert("feature".into(), Value::String("123-foo".into()));
        ctx.insert("task-number".into(), Value::String("1".into()));
        ctx.insert(
            "write-boundary".into(),
            Value::Array(vec![Value::String("runtime/**".into())]),
        );
        // A legacy field that should appear in the merged output AFTER the
        // typed prefix.
        ctx.insert("legacy-extra".into(), Value::String("kept".into()));

        let value = build_write_code_request(&ctx, tmp.path(), "implement").unwrap();
        let keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        // Cache-anchor order: constitution-excerpts, plan-relevant-files,
        // write-boundary, task — then legacy keys.
        let prefix: Vec<&str> = keys.iter().take(4).copied().collect();
        assert_eq!(
            prefix,
            vec![
                "constitution-excerpts",
                "plan-relevant-files",
                "write-boundary",
                "task"
            ]
        );
        assert!(keys.contains(&"legacy-extra"));
        assert_eq!(value["task"]["number"], "1");
        assert_eq!(value["task"]["heading"], "Stub a module");
    }

    /// Stage a repo with a spec file and one rule file carrying a wrapped
    /// Verification paragraph, for the assessSpecQuality builder tests.
    fn stage_assess_fixture(repo: &Path) {
        fs::create_dir_all(repo.join("specs/003-analyze")).unwrap();
        fs::write(repo.join("specs/003-analyze/spec.md"), "# Spec body\n").unwrap();
        fs::create_dir_all(repo.join("framework/rules")).unwrap();
        fs::write(
            repo.join("framework/rules/configuration.md"),
            "# Configuration Rules\n\n\
             ### CFG-CONST-001\n\n\
             > **Statement:** Constants live in one central module.\n\n\
             **Rationale:** Centralizing makes drift impossible.\n\n\
             **Verification:** Every constant is sourced from\n\
             the central module.\n",
        )
        .unwrap();
    }

    fn assess_context() -> Map<String, Value> {
        let mut ctx = Map::new();
        ctx.insert("feature".into(), Value::String("003-analyze".into()));
        ctx.insert(
            "path".into(),
            Value::String("specs/003-analyze/spec.md".into()),
        );
        ctx.insert(
            "rule-files".into(),
            Value::Array(vec![Value::String(
                "framework/rules/configuration.md".into(),
            )]),
        );
        ctx.insert(
            "citations".into(),
            serde_json::json!([
                { "rule-id": "CFG-CONST-001", "found": true, "deprecated": false }
            ]),
        );
        // A legacy dump key that must NOT leak into the typed payload.
        ctx.insert("stdout".into(), Value::String("noise".into()));
        ctx
    }

    #[test]
    fn build_assess_spec_quality_request_emits_documented_typed_shape() {
        let tmp = tempdir().unwrap();
        stage_assess_fixture(tmp.path());
        let prose = "For every loaded MUST-tier rule whose Verification trigger \
                     fires against the spec, request a semantic assessment.";
        let value = build_assess_spec_quality_request(&assess_context(), tmp.path(), prose);
        let keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        // The typed fields lead — and, per the data model, are the entire
        // payload: no raw walker-context dump trails them.
        assert_eq!(keys, vec!["spec-path", "spec-content", "rule"]);
        assert_eq!(value["spec-path"], "specs/003-analyze/spec.md");
        assert_eq!(value["spec-content"], "# Spec body\n");
        assert_eq!(value["rule"]["id"], "CFG-CONST-001");
        assert_eq!(value["rule"]["severity"], "must");
        // Wrapped Verification lines are joined into one phrase.
        assert_eq!(
            value["rule"]["verification"],
            "Every constant is sourced from the central module."
        );
    }

    #[test]
    fn build_assess_spec_quality_request_reads_severity_from_step_tier() {
        let tmp = tempdir().unwrap();
        stage_assess_fixture(tmp.path());
        let prose = "For every loaded SHOULD-tier rule whose Verification trigger \
                     fires against the spec, request a semantic assessment.";
        let value = build_assess_spec_quality_request(&assess_context(), tmp.path(), prose);
        assert_eq!(value["rule"]["severity"], "should");
    }

    #[test]
    fn build_assess_spec_quality_request_falls_back_to_first_defined_rule() {
        // No usable citation (the only one is deprecated) → the first rule
        // defined in the loaded rule files is assessed instead.
        let tmp = tempdir().unwrap();
        stage_assess_fixture(tmp.path());
        let mut ctx = assess_context();
        ctx.insert(
            "citations".into(),
            serde_json::json!([
                { "rule-id": "CFG-CONST-001", "found": true, "deprecated": true }
            ]),
        );
        let prose = "For every loaded MUST-tier rule, request an assessment.";
        let value = build_assess_spec_quality_request(&ctx, tmp.path(), prose);
        assert_eq!(value["rule"]["id"], "CFG-CONST-001");
        assert!(
            value["rule"]["verification"]
                .as_str()
                .unwrap()
                .starts_with("Every constant")
        );
    }

    #[test]
    fn build_assess_spec_quality_request_is_typed_even_when_context_is_bare() {
        // Missing spec file, no citations, no rule files: the typed shape
        // still leads with empty fields rather than reverting to a dump.
        let tmp = tempdir().unwrap();
        let mut ctx = Map::new();
        ctx.insert("path".into(), Value::String("specs/absent/spec.md".into()));
        let value = build_assess_spec_quality_request(&ctx, tmp.path(), "no tier named");
        let keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, vec!["spec-path", "spec-content", "rule"]);
        assert_eq!(value["spec-content"], "");
        assert_eq!(value["rule"]["id"], "");
        assert_eq!(value["rule"]["severity"], "");
    }

    #[test]
    fn build_extension_request_routes_assess_spec_quality_to_typed_builder() {
        let tmp = tempdir().unwrap();
        stage_assess_fixture(tmp.path());
        let prose = "For every loaded MUST-tier rule, request an assessment.";
        let value = build_extension_request(
            "assessSpecQuality",
            &assess_context(),
            tmp.path(),
            "analyze",
            prose,
        )
        .unwrap();
        let obj = value.as_object().unwrap();
        assert!(obj.contains_key("spec-path"));
        assert!(obj.contains_key("rule"));
        // The raw context dump no longer reaches the host.
        assert!(!obj.contains_key("stdout"));
    }

    #[test]
    fn extract_rule_verification_scopes_to_the_named_rule() {
        let rules = "# Rules\n\n\
                     ### AAA-001\n\n\
                     **Verification:** First rule phrase.\n\n\
                     ### BBB-002\n\n\
                     **Verification:** Second rule phrase.\n";
        assert_eq!(
            extract_rule_verification(rules, "BBB-002").as_deref(),
            Some("Second rule phrase.")
        );
        assert_eq!(
            extract_rule_verification(rules, "AAA-001").as_deref(),
            Some("First rule phrase.")
        );
        assert!(extract_rule_verification(rules, "CCC-003").is_none());
    }

    #[test]
    fn build_perform_review_request_bundles_scope_and_rules() {
        let tmp = tempdir().unwrap();
        // In-scope file and a rule file the pass should read off disk.
        fs::create_dir_all(tmp.path().join("runtime/src")).unwrap();
        fs::write(tmp.path().join("runtime/src/main.rs"), "fn main() {}").unwrap();
        fs::create_dir_all(tmp.path().join("framework/rules")).unwrap();
        fs::write(
            tmp.path().join("framework/rules/security-backend.md"),
            "# Security\n",
        )
        .unwrap();

        let mut ctx = Map::new();
        ctx.insert("pass".into(), Value::String("security".into()));
        ctx.insert(
            "scope".into(),
            Value::Array(vec![
                Value::String("runtime/src/main.rs".into()),
                // An absent path is skipped, not an error.
                Value::String("runtime/src/absent.rs".into()),
            ]),
        );
        ctx.insert("rules-dir".into(), Value::String("framework/rules".into()));
        ctx.insert(
            "selected".into(),
            Value::Array(vec![Value::String("security-backend.md".into())]),
        );

        let value = build_perform_review_request(&ctx, tmp.path());
        let obj = value.as_object().unwrap();
        // Typed prefix leads in cache-anchor order.
        let prefix: Vec<&str> = obj.keys().map(String::as_str).take(3).collect();
        assert_eq!(prefix, vec!["scope-files", "rule-files", "pass"]);
        assert_eq!(value["pass"], "security");
        // Only the readable scope file is bundled.
        assert_eq!(value["scope-files"].as_array().unwrap().len(), 1);
        assert_eq!(value["scope-files"][0]["path"], "runtime/src/main.rs");
        assert_eq!(value["scope-files"][0]["content"], "fn main() {}");
        assert_eq!(value["rule-files"][0]["name"], "security-backend.md");
        assert_eq!(value["rule-files"][0]["content"], "# Security\n");
    }

    #[test]
    fn build_perform_review_request_is_empty_without_scope_or_rules() {
        let tmp = tempdir().unwrap();
        let mut ctx = Map::new();
        ctx.insert("pass".into(), Value::String("reuse".into()));
        let value = build_perform_review_request(&ctx, tmp.path());
        assert_eq!(value["pass"], "reuse");
        assert!(value["scope-files"].as_array().unwrap().is_empty());
        assert!(value["rule-files"].as_array().unwrap().is_empty());
    }

    #[test]
    fn load_scope_files_confines_reads_to_the_repo_root() {
        // BE-INPUT-004: a `scope` entry that escapes the repo (absolute or
        // `..` traversal) or does not exist is skipped, never read into the
        // performReview payload; only the in-repo file is bundled.
        let outer = tempdir().unwrap();
        let repo = outer.path().join("repo");
        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/in.rs"), "fn a() {}").unwrap();
        // A secret file OUTSIDE the repo the traversal/absolute entries target.
        fs::write(outer.path().join("secret.txt"), "leaked").unwrap();
        let abs_secret = outer
            .path()
            .join("secret.txt")
            .canonicalize()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        let mut ctx = Map::new();
        ctx.insert(
            "scope".into(),
            Value::Array(vec![
                Value::String("src/in.rs".into()),     // in-repo → read
                Value::String("../secret.txt".into()), // traversal → skipped
                Value::String(abs_secret),             // absolute → skipped
                Value::String("src/absent.rs".into()), // missing → skipped
            ]),
        );

        let files = load_scope_files(&ctx, &repo);
        assert_eq!(files.len(), 1, "only the in-repo scope file is read");
        assert_eq!(files[0].path, "src/in.rs");
        assert_eq!(files[0].content, "fn a() {}");
    }

    #[test]
    fn write_code_merge_filters_walker_accumulator_keys() {
        let tmp = tempdir().unwrap();
        let mut ctx = Map::new();
        ctx.insert("feature".into(), Value::String("123-foo".into()));
        ctx.insert("legacy-extra".into(), Value::String("kept".into()));
        // Walker-internal accumulator state that must not ride along.
        ctx.insert(
            "llm:writeSpecBody".into(),
            serde_json::json!({ "content": "prior response" }),
        );
        ctx.insert("findings".into(), serde_json::json!([{ "rule": "X" }]));

        let value = build_write_code_request(&ctx, tmp.path(), "implement").unwrap();
        let obj = value.as_object().unwrap();
        assert!(!obj.contains_key("llm:writeSpecBody"));
        assert!(!obj.contains_key("findings"));
        assert_eq!(obj["legacy-extra"], "kept");
        // The cache-anchor prefix is untouched by the filtering.
        let prefix: Vec<&str> = obj.keys().map(String::as_str).take(4).collect();
        assert_eq!(
            prefix,
            vec![
                "constitution-excerpts",
                "plan-relevant-files",
                "write-boundary",
                "task"
            ]
        );
    }

    #[test]
    fn perform_review_merge_keeps_primitive_results_drops_accumulators() {
        // Pass N must not see passes 1..N-1's findings or llm:* echoes,
        // but keeps the task-46 result threading: scope/diff-base from
        // compute-review-scope, selected/rules-dir/notices from
        // discover-rule-files.
        let tmp = tempdir().unwrap();
        let mut ctx = Map::new();
        ctx.insert("pass".into(), Value::String("reuse".into()));
        ctx.insert("scope".into(), serde_json::json!(["runtime/src/a.rs"]));
        ctx.insert("diff-base".into(), Value::String("abc1234".into()));
        ctx.insert("rules-dir".into(), Value::String("framework/rules".into()));
        ctx.insert("selected".into(), serde_json::json!(["reuse.md"]));
        ctx.insert("notices".into(), serde_json::json!(["fallback notice"]));
        ctx.insert(
            "findings".into(),
            serde_json::json!([{ "rule": "SEC-BE-001" }]),
        );
        ctx.insert(
            "llm:performReview".into(),
            serde_json::json!({ "findings": [] }),
        );

        let value = build_perform_review_request(&ctx, tmp.path());
        let obj = value.as_object().unwrap();
        assert!(!obj.contains_key("findings"));
        assert!(!obj.contains_key("llm:performReview"));
        for kept in ["scope", "diff-base", "rules-dir", "selected", "notices"] {
            assert!(obj.contains_key(kept), "primitive result `{kept}` dropped");
        }
    }

    #[test]
    fn build_ask_clarify_question_request_emits_typed_shape() {
        let tmp = tempdir().unwrap();
        let feature_dir = tmp.path().join("specs/007-clarify");
        fs::create_dir_all(&feature_dir).unwrap();
        fs::write(feature_dir.join("spec.md"), "# Spec body\n").unwrap();
        let mut ctx = Map::new();
        ctx.insert("feature".into(), Value::String("007-clarify".into()));
        // Session-style directory path (not the spec file).
        ctx.insert("path".into(), Value::String("specs/007-clarify".into()));
        // Merged read-spec result.
        ctx.insert(
            "open-questions".into(),
            serde_json::json!([{ "text": "Which auth mode?" }, { "text": "Second?" }]),
        );
        // A dump key that must NOT leak into the typed payload.
        ctx.insert("stdout".into(), Value::String("noise".into()));

        let value = build_ask_clarify_question_request(&ctx, tmp.path());
        let keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, vec!["spec-path", "spec-content", "question"]);
        assert_eq!(value["spec-path"], "specs/007-clarify/spec.md");
        assert_eq!(value["spec-content"], "# Spec body\n");
        assert_eq!(value["question"]["text"], "Which auth mode?");
        assert!(
            value["question"].get("section").is_none(),
            "unattributed question omits `section`"
        );
    }

    #[test]
    fn ask_clarify_question_prefers_explicit_question_value() {
        let tmp = tempdir().unwrap();
        let mut ctx = Map::new();
        ctx.insert(
            "question".into(),
            serde_json::json!({ "text": "Cap at 60s?", "section": "Behavior" }),
        );
        ctx.insert(
            "open-questions".into(),
            serde_json::json!([{ "text": "ignored fallback" }]),
        );
        let value = build_ask_clarify_question_request(&ctx, tmp.path());
        assert_eq!(value["question"]["text"], "Cap at 60s?");
        assert_eq!(value["question"]["section"], "Behavior");
    }

    #[test]
    fn build_route_inbox_item_request_lists_available_specs() {
        let tmp = tempdir().unwrap();
        for (slug, status) in [("001-alpha", "done"), ("002-beta", "draft")] {
            let dir = tmp.path().join("specs").join(slug);
            fs::create_dir_all(&dir).unwrap();
            fs::write(
                dir.join("spec.md"),
                format!("---\nstatus: {status}\ndependencies: []\n---\n\n# {slug}\n"),
            )
            .unwrap();
        }
        // Non-feature siblings are skipped.
        fs::create_dir_all(tmp.path().join("specs/templates")).unwrap();

        let mut ctx = Map::new();
        ctx.insert(
            "item-text".into(),
            Value::String("Bug: retry loop never backs off".into()),
        );
        ctx.insert("stdout".into(), Value::String("noise".into()));

        let value = build_route_inbox_item_request(&ctx, tmp.path());
        let keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, vec!["item-text", "routes", "available-specs"]);
        assert_eq!(value["item-text"], "Bug: retry loop never backs off");
        assert_eq!(
            value["routes"],
            serde_json::json!(["rule", "spec", "scenario", "chore", "discard"])
        );
        assert_eq!(
            value["available-specs"],
            serde_json::json!([
                { "feature": "001-alpha", "status": "done" },
                { "feature": "002-beta", "status": "draft" }
            ])
        );
    }

    #[test]
    fn build_extension_request_errors_on_unknown_identifier() {
        // The raw-context-dump fallback is gone: an identifier without a
        // typed builder is a structured error (extension-request-hygiene).
        let tmp = tempdir().unwrap();
        let err = build_extension_request("mysteryPoint", &Map::new(), tmp.path(), "test", "")
            .unwrap_err();
        assert_eq!(err.code(), "unknown-extension");
        assert!(err.to_string().contains("mysteryPoint"));
    }
}
