//! Extension-point request builders.
//!
//! Bundles the static context (`constitution-excerpts`, `plan-relevant-files`,
//! `write-boundary`) that LLM extension points need into the typed
//! [`WriteCodeRequest`] / [`WriteSpecBodyRequest`] shapes defined in
//! [`crate::schema::extensions`]. The interpreter calls
//! [`build_extension_request`] just before emitting an `llm-request` envelope;
//! the result replaces the previous "dump the walker context as the request"
//! behavior with a payload whose field order is cache-anchored per the
//! spec 022 LLM extension points contract (stable prefix front, per-task
//! variable suffix last).
//!
//! [`WriteCodeRequest`]: crate::schema::extensions::WriteCodeRequest
//! [`WriteSpecBodyRequest`]: crate::schema::extensions::WriteSpecBodyRequest

#![allow(clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use regex::Regex;
use serde_json::{Map, Value};

use crate::primitives::read_tasks;
use crate::schema::extensions::{
    PlanRelevantFile, WriteCodeRequest, WriteCodeTask, WriteSpecBodyRequest,
};
use crate::schema::primitives::ReadTasksArgs;

/// Errors that abort payload construction. The interpreter surfaces these
/// as structured `error` envelopes (e.g.,
/// [`SecretExfiltration`](PayloadError::SecretExfiltration) → code
/// `secret-exfiltration-blocked`).
#[derive(Debug, thiserror::Error)]
pub enum PayloadError {
    /// A path listed in the plan's Affected Files matched a secret-bearing
    /// pattern (`.env`, `credentials*`, etc.) or was marked ignored by
    /// `.gitignore`.
    #[error("secret-exfiltration-blocked: '{path}' matches pattern '{pattern}'")]
    SecretExfiltration {
        /// Offending repo-relative path.
        path: String,
        /// Pattern that matched (a glob name or `.gitignore`).
        pattern: String,
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
///   with hosts that already parse them.
/// - `writeSpecBody` — appends `existing-content` to the context dump when
///   the procedure's step prose names a section that already has body
///   content on disk. Other typed fields are left to the host (the runtime
///   does not yet template the spec/plan templates here).
/// - any other identifier — passthrough; emits the walker context as-is.
///
/// # Errors
///
/// Returns [`PayloadError::SecretExfiltration`] when any path in
/// `plan-relevant-files` matches a secret-bearing pattern or `.gitignore`.
pub fn build_extension_request(
    identifier: &str,
    context: &Map<String, Value>,
    repo: &Path,
    command_name: &str,
    step_prose: &str,
) -> Result<Value, PayloadError> {
    match identifier {
        "writeCode" => build_write_code_request(context, repo, command_name),
        "writeSpecBody" => Ok(build_write_spec_body_request(context, repo, step_prose)),
        _ => Ok(Value::Object(context.clone())),
    }
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
    // then legacy context fields that hosts may already parse. Keys present
    // in both prefer the typed value.
    let typed_value = serde_json::to_value(&typed).unwrap_or(Value::Null);
    let mut object = match typed_value {
        Value::Object(map) => map,
        _ => Map::new(),
    };
    for (key, value) in context {
        object.entry(key.clone()).or_insert_with(|| value.clone());
    }
    Ok(Value::Object(object))
}

fn build_write_spec_body_request(
    context: &Map<String, Value>,
    repo: &Path,
    step_prose: &str,
) -> Value {
    let object = context.clone();
    if let Some(section) = extract_section_name(step_prose) {
        // Resolve which file the section belongs to from the running command
        // (writeSpecBody is called from /gov:plan against plan.md and from
        // /gov:specify against spec.md). The context's `feature` (or `path`)
        // keys point at the feature directory.
        let feature = context.get("feature").and_then(Value::as_str);
        let path_hint = context.get("path").and_then(Value::as_str);
        if let Some(existing) = read_existing_section(&section, feature, path_hint, repo) {
            // Build the typed payload merged on top so the `section` /
            // `existing-content` fields land in the request alongside the
            // legacy context dump. Other typed fields stay empty here; the
            // host fills them when it has more context.
            let typed = WriteSpecBodyRequest {
                template_path: String::new(),
                template_content: String::new(),
                section: section.clone(),
                feature_description: String::new(),
                existing_content: Some(existing),
            };
            if let Ok(Value::Object(map)) = serde_json::to_value(&typed) {
                // Insert the typed fields ahead of (or merged with) the
                // context dump, but skip empty placeholders so we don't
                // pollute the envelope with empty `template-path` etc.
                let mut prefixed: Map<String, Value> = Map::new();
                for (key, value) in map {
                    if matches!(&value, Value::String(s) if s.is_empty()) {
                        continue;
                    }
                    prefixed.insert(key, value);
                }
                for (key, value) in object {
                    prefixed.entry(key).or_insert(value);
                }
                return Value::Object(prefixed);
            }
        }
    }
    Value::Object(object)
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
    let plan_path = repo.join("specs").join(feature).join("plan.md");
    let Ok(plan_content) = std::fs::read_to_string(&plan_path) else {
        return Ok(Vec::new());
    };
    let paths = parse_affected_files(&plan_content);
    let mut out = Vec::new();
    for rel in paths {
        if let Some(pattern) = secret_pattern(&rel) {
            return Err(PayloadError::SecretExfiltration {
                path: rel,
                pattern: pattern.into(),
            });
        }
        if is_gitignored(repo, &rel) {
            return Err(PayloadError::SecretExfiltration {
                path: rel,
                pattern: ".gitignore".into(),
            });
        }
        let abs = repo.join(&rel);
        let Ok(content) = std::fs::read_to_string(&abs) else {
            // Planned-new file or rename target — omit, don't error.
            continue;
        };
        out.push(PlanRelevantFile { path: rel, content });
    }
    Ok(out)
}

/// Parse the `## Affected Files` markdown table in a plan body and return
/// the first-column path entries in document order. Tolerates rows with
/// backtick-wrapped paths and skips the header separator row.
#[must_use]
pub fn parse_affected_files(plan_content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut in_section = false;
    let mut in_fence = false;
    let mut saw_header = false;
    for line in plan_content.lines() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("## ") {
            // Heading boundary: enter the section when we hit its header,
            // exit on any other H2.
            in_section = rest.trim().eq_ignore_ascii_case("Affected Files");
            saw_header = false;
            continue;
        }
        if !in_section {
            continue;
        }
        if !trimmed.starts_with('|') {
            continue;
        }
        // Skip the separator row (e.g., `| --- | --- | --- |`).
        if trimmed
            .bytes()
            .all(|b| matches!(b, b'|' | b'-' | b':' | b' '))
        {
            saw_header = true;
            continue;
        }
        if !saw_header {
            // First row is the header (`| File | Action | ... |`) — skip
            // until the separator passes.
            continue;
        }
        // Strip the leading `|`, take the first cell.
        let after_pipe = trimmed.trim_start_matches('|');
        let Some((cell, _)) = after_pipe.split_once('|') else {
            continue;
        };
        let path = cell.trim().trim_matches('`').trim().to_string();
        if path.is_empty() {
            continue;
        }
        out.push(path);
    }
    out
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
    for rel in [
        format!("framework/commands/{command_name}.md"),
        format!(".claude/commands/gov/{command_name}.md"),
        format!("framework/bootstrap/{command_name}.md"),
    ] {
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
    let after_marker_line = match content[start..].find('\n') {
        Some(rel) => start + rel + 1,
        None => return None,
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

/// Read a section body from a spec or plan file. Resolves the file from the
/// running command:
///
/// - `/gov:plan` → `specs/{feature}/plan.md`
/// - `/gov:specify` → `specs/{feature}/spec.md`
///
/// Returns `None` when the file does not exist or the section is absent or
/// empty. Whitespace-only bodies count as empty.
fn read_existing_section(
    section: &str,
    feature: Option<&str>,
    path_hint: Option<&str>,
    repo: &Path,
) -> Option<String> {
    let feature_dir = match feature {
        Some(f) if !f.is_empty() => repo.join("specs").join(f),
        _ => repo.join(path_hint.unwrap_or_default()),
    };
    // Try both candidate files — the command name isn't threaded into the
    // section reader, so prefer plan.md when it exists (writeSpecBody for
    // /gov:plan), fall back to spec.md (writeSpecBody for /gov:specify).
    for filename in ["plan.md", "spec.md"] {
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
/// no such phrase is present.
fn extract_section_name(prose: &str) -> Option<String> {
    static R: OnceLock<Regex> = OnceLock::new();
    let re = R.get_or_init(|| {
        Regex::new(r"(?i)Fill\s+the\s+(.+?)\s+section").expect("hard-coded regex compiles")
    });
    re.captures(prose).map(|c| c[1].trim().to_string())
}

/// Match a path against the v1 secret-exfiltration patterns. Returns the
/// matched pattern label when blocked; `None` otherwise. Patterns:
///
/// - `.env` and `.env.*` (e.g., `.env.production`)
/// - `*-secrets.*` (e.g., `db-secrets.yaml`)
/// - `credentials*` (any extension)
fn secret_pattern(path: &str) -> Option<&'static str> {
    let basename = Path::new(path)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("");
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
    fn parse_affected_files_extracts_first_column_paths() {
        let plan = "# Plan\n\n\
                    ## Affected Files\n\n\
                    | File | Action | Purpose |\n\
                    | --- | --- | --- |\n\
                    | `runtime/src/foo.rs` | Create | Foo |\n\
                    | `runtime/src/bar.rs` | Edit | Bar |\n\
                    | scripts/baz.sh | Create | Baz |\n\n\
                    ## Trade-offs\n\nIrrelevant.\n";
        let paths = parse_affected_files(plan);
        assert_eq!(
            paths,
            vec![
                "runtime/src/foo.rs".to_string(),
                "runtime/src/bar.rs".to_string(),
                "scripts/baz.sh".to_string()
            ]
        );
    }

    #[test]
    fn parse_affected_files_handles_missing_section() {
        let plan = "# Plan\n\n## Trade-offs\n\nNo affected files.\n";
        let paths = parse_affected_files(plan);
        assert!(paths.is_empty());
    }

    #[test]
    fn parse_affected_files_ignores_table_inside_fenced_block() {
        let plan = "# Plan\n\n\
                    ## Affected Files\n\n\
                    ```text\n\
                    | not | a | table |\n\
                    | --- | --- | --- |\n\
                    | `nope.md` | Create | Fake |\n\
                    ```\n\n\
                    | File | Action | Purpose |\n\
                    | --- | --- | --- |\n\
                    | `real.md` | Create | Real |\n";
        let paths = parse_affected_files(plan);
        assert_eq!(paths, vec!["real.md".to_string()]);
    }

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
        let value = build_write_spec_body_request(&ctx, tmp.path(), prose);
        let obj = value.as_object().unwrap();
        assert_eq!(obj["section"], "Technical Decisions");
        assert_eq!(
            obj["existing-content"].as_str().unwrap(),
            "Use the standard library."
        );
    }

    #[test]
    fn build_write_spec_body_request_passes_through_when_no_existing_content() {
        let tmp = tempdir().unwrap();
        let mut ctx = Map::new();
        ctx.insert("feature".into(), Value::String("999-empty".into()));
        let prose = "Fill the Motivation section of the spec.";
        let value = build_write_spec_body_request(&ctx, tmp.path(), prose);
        let obj = value.as_object().unwrap();
        // No existing content found → no `existing-content` key added.
        assert!(!obj.contains_key("existing-content"));
        // Context dump is preserved.
        assert_eq!(obj["feature"], "999-empty");
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
}
