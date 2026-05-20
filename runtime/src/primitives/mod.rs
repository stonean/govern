//! Deterministic primitive operations.
//!
//! Each primitive has a pure-Rust `run` function (no stdout/stderr I/O — the
//! caller wraps the result into a JSON envelope), a `clap`-derive args struct
//! from [`crate::schema::primitives`], and a unit test against a fixture file
//! under `runtime/tests/fixtures/primitives/`.

#![allow(clippy::module_name_repetitions)]

use std::io::Write;
use std::path::{Path, PathBuf};

pub mod append_task;
pub mod apply_manifest;
pub mod check_rule_ids;
pub mod check_stuck;
pub mod create_scenario;
pub mod derive_boundary;
pub mod enforce_manifest;
pub mod extract_archive;
pub mod fetch_archive;
pub mod gate_confirm;
pub mod lint_markdown;
pub mod mark_criterion;
pub mod mark_task;
pub mod merge_claude_md;
pub mod merge_managed_block;
pub mod merge_permissions;
pub mod read_spec;
pub mod read_tasks;
pub mod resolve_anchor;
pub mod run_generator;
pub mod set_status;
pub mod substitute_templates;
pub mod traverse_deps;
pub mod validate_frontmatter;

/// Operational errors common to every primitive. Domain outcomes (findings,
/// violations, drift) are reported through the result struct; this enum is
/// reserved for operational failures that halt the procedure.
#[derive(Debug, thiserror::Error)]
pub enum PrimitiveError {
    /// I/O failure on a specific path.
    #[error("I/O error on {path}: {source}")]
    Io {
        /// Path involved in the failed operation.
        path: PathBuf,
        /// Underlying I/O error.
        #[source]
        source: std::io::Error,
    },
    /// YAML parse failure for a frontmatter block.
    #[error("YAML parse error in {path}: {source}")]
    Yaml {
        /// Path of the file whose frontmatter failed to parse.
        path: PathBuf,
        /// Underlying YAML error.
        #[source]
        source: serde_yaml::Error,
    },
    /// File has no leading `---` frontmatter block.
    #[error("frontmatter missing in {path} (no leading `---` block)")]
    MissingFrontmatter {
        /// Path of the offending file.
        path: PathBuf,
    },
    /// Feature directory does not exist under `specs/`.
    #[error("feature directory not found: specs/{feature}")]
    FeatureNotFound {
        /// Requested feature name.
        feature: String,
    },
    /// Git operation failed.
    #[error("git error: {0}")]
    Git(#[from] git2::Error),
    /// Repository contains no commits that touch the requested spec dir.
    #[error("no commits found that touch specs/{feature}")]
    NoSpecHistory {
        /// Requested feature name.
        feature: String,
    },
    /// Requested task number not found in `tasks.md`.
    #[error("task '{task_number}' not found in specs/{feature}/tasks.md")]
    TaskNotFound {
        /// Feature whose tasks file was scanned.
        feature: String,
        /// Task number that was requested.
        task_number: String,
    },
    /// Subtask index is out of bounds for the located task.
    #[error(
        "subtask index {subtask_index} is out of range for task '{task_number}' (found {total})"
    )]
    SubtaskOutOfRange {
        /// Feature whose tasks file was scanned.
        feature: String,
        /// Task number whose subtasks were counted.
        task_number: String,
        /// Requested subtask index.
        subtask_index: usize,
        /// Number of subtasks present.
        total: usize,
    },
    /// Acceptance-criterion index is out of bounds.
    #[error(
        "criterion index {criterion_index} is out of range for specs/{feature}/spec.md (found {total})"
    )]
    CriterionOutOfRange {
        /// Feature whose spec was scanned.
        feature: String,
        /// Requested criterion index.
        criterion_index: usize,
        /// Number of acceptance criteria present.
        total: usize,
    },
    /// `set-status` was invoked with a `from` value that does not match disk.
    #[error("status mismatch in specs/{feature}/spec.md: expected '{expected}', found '{actual}'")]
    StatusMismatch {
        /// Feature whose spec was scanned.
        feature: String,
        /// Status the caller expected on disk.
        expected: String,
        /// Status actually present on disk.
        actual: String,
    },
    /// Frontmatter does not contain a `status:` field.
    #[error("frontmatter in specs/{feature}/spec.md has no `status:` field")]
    StatusFieldMissing {
        /// Feature whose spec was scanned.
        feature: String,
    },
    /// HTTP fetch returned a non-success status code.
    #[error("HTTP {status} fetching {url}")]
    HttpStatus {
        /// URL that returned the error.
        url: String,
        /// HTTP status code observed.
        status: u16,
    },
    /// Underlying `reqwest` failure (connect refused, TLS error, etc.).
    #[error("HTTP error on {url}: {source}")]
    Http {
        /// URL involved in the failed request.
        url: String,
        /// Underlying reqwest error.
        #[source]
        source: reqwest::Error,
    },
    /// sha256 sidecar did not match the computed hash of the downloaded archive.
    #[error("sha256 mismatch for {path}: sidecar declared {expected}, computed {actual}")]
    ChecksumMismatch {
        /// Local path of the archive whose sha didn't match.
        path: PathBuf,
        /// Hex digest declared in the sidecar.
        expected: String,
        /// Hex digest computed locally.
        actual: String,
    },
    /// sha256 sidecar payload didn't parse as `<hex>  <filename>` format.
    #[error("malformed sha256 sidecar from {url}: {reason}")]
    MalformedSidecar {
        /// URL the sidecar was fetched from.
        url: String,
        /// One-line description of what was malformed.
        reason: String,
    },
    /// Archive format could not be inferred from extension and no override given.
    #[error("unknown archive format for {path} (expected .tar.gz/.tgz/.zip)")]
    UnknownArchiveFormat {
        /// Local archive path whose format couldn't be determined.
        path: PathBuf,
    },
    /// Archive entry path escapes the destination directory (`..`, absolute).
    #[error("unsafe archive entry path: {entry}")]
    UnsafeArchivePath {
        /// Entry path as it appeared inside the archive.
        entry: String,
    },
    /// CLAUDE.md merge found a BEGIN marker without a matching END (or
    /// vice versa).
    #[error("malformed managed-block markers in {path}: {reason}")]
    MalformedMarkers {
        /// Path of the file whose markers were malformed.
        path: PathBuf,
        /// One-line description of the structural failure.
        reason: String,
    },
    /// Manifest entry referenced an unknown strategy. Valid values are
    /// `update`, `create`, and `skip-if-conflict`.
    #[error(
        "unknown manifest strategy '{strategy}' (expected 'update', 'create', or 'skip-if-conflict')"
    )]
    UnknownManifestStrategy {
        /// Strategy string as it appeared in the manifest entry.
        strategy: String,
    },
    /// `create-scenario` refused to overwrite an existing scenario file.
    #[error("scenario already exists: {path}")]
    ScenarioConflict {
        /// Path of the existing scenario file the primitive refused to overwrite.
        path: PathBuf,
    },
    /// Feature path supplied to a primitive does not exist.
    #[error("feature path does not exist: {path}")]
    FeaturePathNotFound {
        /// Caller-supplied feature path that did not resolve to a directory.
        path: PathBuf,
    },
    /// Slug component supplied by a caller failed validation (path separator,
    /// dot-prefix, or empty value).
    #[error("invalid slug '{slug}': {reason}")]
    InvalidSlug {
        /// Slug that was rejected.
        slug: String,
        /// One-line reason describing the rejection.
        reason: String,
    },
    /// Caller-supplied path failed traversal-safety validation.
    #[error("invalid path '{path}': {reason}")]
    InvalidPath {
        /// Path that was rejected.
        path: String,
        /// One-line reason describing the rejection.
        reason: String,
    },
    /// A required argument was omitted by the caller. Distinct from
    /// `InvalidSlug` / `InvalidPath` — the value was never supplied, not
    /// supplied-and-rejected.
    #[error("{primitive}: '{argument}' is required ({reason})")]
    MissingArgument {
        /// Primitive name (e.g., `append-task`).
        primitive: String,
        /// Argument name that was omitted.
        argument: String,
        /// One-line reason explaining why the argument is required in
        /// this context.
        reason: String,
    },
    /// `append-task` was called with a `parent-heading` argument that does
    /// not match any `## …` phase container in the target `tasks.md`.
    #[error(
        "append-task: parent-heading '{heading}' not found in tasks.md (available: {available})"
    )]
    ParentHeadingNotFound {
        /// Caller-supplied heading text that didn't match.
        heading: String,
        /// Comma-separated list of available phase headings (for the
        /// operator to choose from when retrying).
        available: String,
    },
    /// JSON parse failure (e.g., `merge-permissions` reading a malformed
    /// `.claude/settings.local.json`).
    #[error("JSON parse error in {path}: {source}")]
    Json {
        /// Path of the file whose JSON failed to parse.
        path: PathBuf,
        /// Underlying `serde_json` error.
        #[source]
        source: serde_json::Error,
    },
    /// JSON parsed but its shape doesn't match the primitive's expected
    /// schema (e.g., `permissions.allow` exists but is not an array).
    #[error("JSON schema mismatch in {path}: {reason}")]
    JsonSchema {
        /// Path of the file whose JSON shape was rejected.
        path: PathBuf,
        /// One-line description of the schema mismatch.
        reason: String,
    },
}

/// Convenience alias for primitive return values.
pub type Result<T> = std::result::Result<T, PrimitiveError>;

/// Split a markdown file's content into its frontmatter YAML block and the
/// body that follows. Returns an error if no `---` opening fence is present
/// or no closing fence is found.
pub(crate) fn split_frontmatter<'a>(content: &'a str, path: &Path) -> Result<(&'a str, &'a str)> {
    let after_open = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))
        .ok_or_else(|| PrimitiveError::MissingFrontmatter { path: path.into() })?;

    for fence in ["\n---\n", "\n---\r\n"] {
        if let Some(idx) = after_open.find(fence) {
            return Ok((&after_open[..idx], &after_open[idx + fence.len()..]));
        }
    }
    Err(PrimitiveError::MissingFrontmatter { path: path.into() })
}

/// Read a UTF-8 file, surfacing path context on failure.
pub(crate) fn read_text(path: &Path) -> Result<String> {
    std::fs::read_to_string(path).map_err(|source| PrimitiveError::Io {
        path: path.into(),
        source,
    })
}

/// Render a path as a repo-relative POSIX string, falling back to the
/// path's display form if it doesn't share a prefix with `repo`.
pub(crate) fn rel_path(path: &Path, repo: &Path) -> String {
    let display = path.strip_prefix(repo).unwrap_or(path);
    display.to_string_lossy().replace('\\', "/")
}

/// Atomically write `content` to `path` using `tempfile`'s create-then-rename
/// pattern. The tempfile is created in `path`'s parent directory so the rename
/// stays on the same filesystem (POSIX guarantee). A crash between creation
/// and persist leaves `path` unchanged; the orphaned tempfile is the only
/// recovery artifact.
pub(crate) fn write_atomic(path: &Path, content: &str) -> Result<()> {
    write_atomic_bytes(path, content.as_bytes())
}

/// Atomically write a byte slice to `path`. Same tempfile-plus-rename
/// pattern as [`write_atomic`]; used by primitives that produce binary
/// payloads (e.g., `fetch-archive` writing a downloaded tarball).
pub(crate) fn write_atomic_bytes(path: &Path, content: &[u8]) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|source| PrimitiveError::Io {
                path: parent.into(),
                source,
            })?;
        }
    }
    let parent = path.parent().unwrap_or_else(|| Path::new("."));
    let mut tmp = tempfile::NamedTempFile::new_in(parent).map_err(|source| PrimitiveError::Io {
        path: parent.into(),
        source,
    })?;
    tmp.as_file_mut()
        .write_all(content)
        .map_err(|source| PrimitiveError::Io {
            path: path.into(),
            source,
        })?;
    tmp.as_file_mut()
        .sync_all()
        .map_err(|source| PrimitiveError::Io {
            path: path.into(),
            source,
        })?;
    tmp.persist(path).map_err(|err| PrimitiveError::Io {
        path: path.into(),
        source: err.error,
    })?;
    Ok(())
}

/// Shared helpers for identifying and flipping markdown task-list checkbox
/// lines (`- [ ] ...` / `- [x] ...`). Used by both `mark-task` and
/// `mark-criterion`; the regex is `^(\s*-\s+)\[([ xX])\](\s.*)?$`, expressed
/// directly via byte inspection to avoid pulling in `regex` for this hot path.
pub(crate) mod checkbox {
    /// Return `(prefix_end, marker_index)` when `line` is a task-list
    /// checkbox line. `prefix_end` is the byte index of the `[`; `marker_index`
    /// is the byte index of the space/x/X marker character.
    pub(crate) fn find_checkbox_line(line: &str) -> Option<(usize, usize)> {
        let bytes = line.as_bytes();
        let mut idx = 0;
        while idx < bytes.len() && matches!(bytes[idx], b' ' | b'\t') {
            idx += 1;
        }
        if bytes.get(idx) != Some(&b'-') {
            return None;
        }
        idx += 1;
        let mut saw_space = false;
        while idx < bytes.len() && matches!(bytes[idx], b' ' | b'\t') {
            saw_space = true;
            idx += 1;
        }
        if !saw_space {
            return None;
        }
        if bytes.get(idx) != Some(&b'[') {
            return None;
        }
        let bracket_idx = idx;
        let marker_idx = idx + 1;
        if !matches!(bytes.get(marker_idx), Some(&b' ' | &b'x' | &b'X')) {
            return None;
        }
        if bytes.get(marker_idx + 1) != Some(&b']') {
            return None;
        }
        match bytes.get(marker_idx + 2) {
            Some(&b' ' | &b'\t' | &b'\n' | &b'\r') | None => Some((bracket_idx, marker_idx)),
            _ => None,
        }
    }

    /// Return `(previous_state, rewritten_line)` after flipping the marker at
    /// `marker_idx` (obtained from [`find_checkbox_line`]) to `desired`.
    pub(crate) fn flip_checkbox_at(line: &str, marker_idx: usize, desired: bool) -> (bool, String) {
        let previous = matches!(line.as_bytes()[marker_idx], b'x' | b'X');
        let mut out = String::with_capacity(line.len());
        out.push_str(&line[..marker_idx]);
        out.push(if desired { 'x' } else { ' ' });
        out.push_str(&line[marker_idx + 1..]);
        (previous, out)
    }
}

/// Reject caller-supplied paths that contain parent-directory components
/// (`..`) or absolute prefixes — the BE-INPUT-004 defense-in-depth check.
/// Primitives that accept paths from the host or LLM call this before any
/// filesystem operation to guarantee the resolved path stays inside the
/// repo root.
pub(crate) fn validate_no_traversal(path: &str) -> Result<()> {
    if path.is_empty() {
        return Err(PrimitiveError::InvalidPath {
            path: path.into(),
            reason: "path is empty".into(),
        });
    }
    let p = Path::new(path);
    if p.is_absolute() {
        return Err(PrimitiveError::InvalidPath {
            path: path.into(),
            reason: "absolute path not permitted".into(),
        });
    }
    for component in p.components() {
        if matches!(component, std::path::Component::ParentDir) {
            return Err(PrimitiveError::InvalidPath {
                path: path.into(),
                reason: "parent-directory component ('..') not permitted".into(),
            });
        }
    }
    Ok(())
}

/// Reject slugs that contain path separators, leading dots, or are empty.
/// Used by primitives that embed a caller-supplied slug into a destination
/// filename (e.g., `create-scenario` writes `scenarios/{slug}.md`).
pub(crate) fn validate_slug(slug: &str) -> Result<()> {
    if slug.is_empty() {
        return Err(PrimitiveError::InvalidSlug {
            slug: slug.into(),
            reason: "slug is empty".into(),
        });
    }
    if slug.contains('/') || slug.contains('\\') {
        return Err(PrimitiveError::InvalidSlug {
            slug: slug.into(),
            reason: "slug must not contain path separators".into(),
        });
    }
    if slug.starts_with('.') {
        return Err(PrimitiveError::InvalidSlug {
            slug: slug.into(),
            reason: "slug must not start with '.'".into(),
        });
    }
    Ok(())
}

/// Walk `content` line by line, yielding the numeric prefix of every ATX
/// heading at any of the given `levels` whose text begins with `N.`. Skips
/// headings inside fenced code blocks. Used by `tasks.md` primitives to
/// compute the next task number in both flat (`## N.`) and phased
/// (`### N.` under `## Phase X`) structures — passing `&[2, 3]` produces
/// the union across both shapes.
pub(crate) fn iter_task_numbers_at_levels<'a>(
    content: &'a str,
    levels: &'a [u8],
) -> impl Iterator<Item = u32> + 'a {
    let mut in_fence = false;
    content.lines().filter_map(move |line| {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            return None;
        }
        if in_fence {
            return None;
        }
        let (level, text) = parse_atx_heading(line)?;
        if !levels.contains(&level) {
            return None;
        }
        let dot = text.find('.')?;
        let num_part = &text[..dot];
        if num_part.is_empty() {
            return None;
        }
        num_part.parse::<u32>().ok()
    })
}

/// Phased vs flat structure of a `tasks.md` file.
///
/// A file is **phased** when it contains at least one `### N.` heading
/// outside of fenced blocks — meaning task entries live at level 3 under
/// `## …` phase containers (e.g., 023's `## Phase A — Refactor / ### 1.
/// Task`). Otherwise it is **flat** — task entries are `## N.` at level 2
/// (the original `tasks.md` shape).
///
/// Detection matches the [scenario][runtime-primitive-structural-bugs]
/// edge case "mixed structure → treat as phased": any `### N.` heading
/// anywhere in the file signals phased structure, even if `## N.` headings
/// are also present.
///
/// [runtime-primitive-structural-bugs]: <https://github.com/stonean/govern/blob/main/specs/022-deterministic-runtime/scenarios/runtime-primitive-structural-bugs.md>
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum TasksStructure {
    /// No `### N.` headings present; task entries are flat (`## N.`).
    Flat,
    /// At least one `### N.` heading present; task entries live under
    /// `## …` phase containers.
    Phased,
}

/// Detect a `tasks.md` file's structure. Used by `append-task` (to choose
/// flat-append vs phase-append) and `read-tasks` (to walk the appropriate
/// heading levels).
pub(crate) fn detect_tasks_structure(content: &str) -> TasksStructure {
    if iter_task_numbers_at_levels(content, &[3]).next().is_some() {
        TasksStructure::Phased
    } else {
        TasksStructure::Flat
    }
}

/// One `## …` phase container in a phased `tasks.md`. `start_line` and
/// `end_line` are 1-based line numbers from the file's `lines()` iterator;
/// `end_line` is the last content line that belongs to this phase (the
/// line before the next `## …` heading, or the last line of the file
/// when this is the final phase).
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct PhaseRange {
    /// Full heading text (without the leading `## ` prefix), e.g.,
    /// "Phase A — Refactor" or "Phase C — Follow-on scenarios".
    pub heading: String,
    /// 1-based line number of the `## …` heading line itself.
    pub start_line: usize,
    /// 1-based line number of the last content line that belongs to this
    /// phase (inclusive).
    pub end_line: usize,
}

/// Walk a phased `tasks.md` body and yield each `## …` phase container's
/// heading text plus the line range it covers. `## N.` headings (numeric
/// flat-task remnants in a mixed-structure file) are NOT treated as
/// phase containers — only `## …` headings with non-numeric text qualify.
/// Behavior on a non-phased file is informational; callers should gate
/// on [`detect_tasks_structure`] before consuming this iterator.
pub(crate) fn iter_phase_ranges(content: &str) -> Vec<PhaseRange> {
    let mut phases: Vec<PhaseRange> = Vec::new();
    let mut in_fence = false;
    let lines: Vec<&str> = content.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        if let Some((2, heading)) = parse_atx_heading(line) {
            // Skip numeric flat-task remnants: a heading whose text begins
            // with "N." (decimal digits, then dot) is a flat task, not a
            // phase container. Mixed files keep their phase set clean.
            if heading_starts_with_number(&heading) {
                continue;
            }
            // 1-based line numbers; close out the previous phase before
            // opening the next.
            let one_based = idx + 1;
            if let Some(prev) = phases.last_mut() {
                prev.end_line = one_based.saturating_sub(1);
            }
            phases.push(PhaseRange {
                heading,
                start_line: one_based,
                end_line: lines.len(), // closed below or left at EOF
            });
        }
    }
    phases
}

/// `true` when `heading` begins with `N.` (decimal digits, then a literal
/// dot). Used to filter numeric task headings from phase containers.
fn heading_starts_with_number(heading: &str) -> bool {
    let bytes = heading.as_bytes();
    let mut i = 0;
    while i < bytes.len() && bytes[i].is_ascii_digit() {
        i += 1;
    }
    i > 0 && i < bytes.len() && bytes[i] == b'.'
}

/// Parse an ATX heading line and return `(level, text)` when the line matches
/// `# heading` through `###### heading`. Trims trailing `#` runs in the closed
/// form (`## Foo ##`).
pub(crate) fn parse_atx_heading(line: &str) -> Option<(u8, String)> {
    let trimmed = line.trim_start();
    let bytes = trimmed.as_bytes();
    let mut level: u8 = 0;
    while (level as usize) < bytes.len() && bytes[level as usize] == b'#' && level < 6 {
        level += 1;
    }
    if level == 0 {
        return None;
    }
    let after = &trimmed[level as usize..];
    if !after.starts_with(' ') && !after.is_empty() {
        return None;
    }
    let heading = after.trim().trim_end_matches('#').trim().to_string();
    Some((level, heading))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn validate_slug_accepts_normal_slugs() {
        validate_slug("retry-on-timeout").unwrap();
        validate_slug("a").unwrap();
        validate_slug("ask-consolidation").unwrap();
    }

    #[test]
    fn validate_slug_rejects_empty() {
        assert!(matches!(
            validate_slug("").unwrap_err(),
            PrimitiveError::InvalidSlug { .. }
        ));
    }

    #[test]
    fn validate_slug_rejects_path_separators() {
        for bad in &["a/b", "a\\b", "../escape", "..\\escape"] {
            assert!(
                matches!(
                    validate_slug(bad).unwrap_err(),
                    PrimitiveError::InvalidSlug { .. }
                ),
                "expected rejection for {bad:?}"
            );
        }
    }

    #[test]
    fn validate_slug_rejects_dotfile_prefix() {
        for bad in &[".hidden", "..", "."] {
            assert!(
                matches!(
                    validate_slug(bad).unwrap_err(),
                    PrimitiveError::InvalidSlug { .. }
                ),
                "expected rejection for {bad:?}"
            );
        }
    }

    #[test]
    fn validate_no_traversal_accepts_normal_paths() {
        validate_no_traversal("specs/042-foo").unwrap();
        validate_no_traversal("a/b/c").unwrap();
        validate_no_traversal("specs/022-deterministic-runtime").unwrap();
    }

    #[test]
    fn validate_no_traversal_rejects_empty() {
        assert!(matches!(
            validate_no_traversal("").unwrap_err(),
            PrimitiveError::InvalidPath { .. }
        ));
    }

    #[test]
    fn validate_no_traversal_rejects_absolute_paths() {
        for bad in &["/etc/passwd", "/tmp/x"] {
            assert!(
                matches!(
                    validate_no_traversal(bad).unwrap_err(),
                    PrimitiveError::InvalidPath { .. }
                ),
                "expected rejection for {bad:?}"
            );
        }
    }

    #[test]
    fn validate_no_traversal_rejects_parent_components() {
        for bad in &["../foo", "specs/../target", "a/b/../c"] {
            assert!(
                matches!(
                    validate_no_traversal(bad).unwrap_err(),
                    PrimitiveError::InvalidPath { .. }
                ),
                "expected rejection for {bad:?}"
            );
        }
    }

    #[test]
    fn iter_numbered_headings_extracts_atx2_numbers() {
        let content = "# Title\n\n## 1. First\n\n## 2. Second\n\n## 3. Third\n\nNot a heading.\n";
        let nums: Vec<u32> = iter_task_numbers_at_levels(content, &[2]).collect();
        assert_eq!(nums, vec![1, 2, 3]);
    }

    #[test]
    fn iter_numbered_headings_skips_non_atx2() {
        let content =
            "# 99. Not counted\n\n## 1. Counted\n\n### 2. Not counted (level 3)\n\n## 2. Counted\n";
        let nums: Vec<u32> = iter_task_numbers_at_levels(content, &[2]).collect();
        assert_eq!(nums, vec![1, 2]);
    }

    #[test]
    fn iter_numbered_headings_skips_fenced_blocks() {
        let content = "## 1. Real\n\n```text\n## 99. Fake\n```\n\n## 2. Real\n";
        let nums: Vec<u32> = iter_task_numbers_at_levels(content, &[2]).collect();
        assert_eq!(nums, vec![1, 2]);
    }

    #[test]
    fn iter_numbered_headings_handles_non_numeric_headings() {
        let content = "## Setup\n\n## 1. First\n\n## Wrap-up\n\n## 7. Seventh\n";
        let nums: Vec<u32> = iter_task_numbers_at_levels(content, &[2]).collect();
        assert_eq!(nums, vec![1, 7]);
    }
}
