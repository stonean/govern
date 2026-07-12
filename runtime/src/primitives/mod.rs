//! Deterministic primitive operations.
//!
//! Each primitive has a pure-Rust `run` function (no stdout/stderr I/O — the
//! caller wraps the result into a JSON envelope), a `clap`-derive args struct
//! from [`crate::schema::primitives`], and a unit test against a fixture file
//! under `runtime/tests/fixtures/primitives/`.

#![allow(clippy::module_name_repetitions)]

use std::io::Write;
use std::path::{Path, PathBuf};

pub mod append_inbox;
pub mod append_task;
pub mod apply_manifest;
pub mod check_artifacts;
pub mod check_rule_ids;
pub mod check_stuck;
pub mod compute_review_scope;
pub mod create_feature;
pub mod create_scenario;
pub mod dashboard;
pub mod derive_boundary;
pub mod discover_rule_files;
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
pub mod migrate_session_file;
pub mod process_waivers;
pub mod prune_tasks;
pub mod read_spec;
pub mod read_tasks;
pub mod resolve_anchor;
pub mod resolve_feature;
pub mod resolve_references;
pub mod run_generator;
pub mod set_status;
pub mod substitute_templates;
pub mod traverse_deps;
pub mod validate_frontmatter;
pub mod write_review;
pub mod write_session;

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
        source: serde_norway::Error,
    },
    /// File has no leading `---` frontmatter block.
    #[error("frontmatter missing in {path} (no leading `---` block)")]
    MissingFrontmatter {
        /// Path of the offending file.
        path: PathBuf,
    },
    /// Feature directory does not exist under the configured spec-root.
    #[error("feature directory not found: {root}/{feature}")]
    FeatureNotFound {
        /// Configured spec-root directory name (default `specs`; spec 040).
        root: String,
        /// Requested feature name.
        feature: String,
    },
    /// Git operation failed.
    #[error("git error: {0}")]
    Git(#[from] git2::Error),
    /// Repository contains no commits that touch the requested spec dir.
    #[error("no commits found that touch {root}/{feature}")]
    NoSpecHistory {
        /// Configured spec-root directory name (default `specs`; spec 040).
        root: String,
        /// Requested feature name.
        feature: String,
    },
    /// Requested task number not found in `tasks.md`.
    #[error("task '{task_number}' not found in {root}/{feature}/tasks.md")]
    TaskNotFound {
        /// Configured spec-root directory name (default `specs`; spec 040).
        root: String,
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
        "criterion index {criterion_index} is out of range for {root}/{feature}/spec.md (found {total})"
    )]
    CriterionOutOfRange {
        /// Configured spec-root directory name (default `specs`; spec 040).
        root: String,
        /// Feature whose spec was scanned.
        feature: String,
        /// Requested criterion index.
        criterion_index: usize,
        /// Number of acceptance criteria present.
        total: usize,
    },
    /// `set-status` was invoked with a `from` value that does not match disk.
    #[error("status mismatch in {root}/{feature}/spec.md: expected '{expected}', found '{actual}'")]
    StatusMismatch {
        /// Configured spec-root directory name (default `specs`; spec 040).
        root: String,
        /// Feature whose spec was scanned.
        feature: String,
        /// Status the caller expected on disk.
        expected: String,
        /// Status actually present on disk.
        actual: String,
    },
    /// Frontmatter does not contain a `status:` field.
    #[error("frontmatter in {root}/{feature}/spec.md has no `status:` field")]
    StatusFieldMissing {
        /// Configured spec-root directory name (default `specs`; spec 040).
        root: String,
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
    /// `create-feature` found no spec template at any candidate location,
    /// so there is nothing to copy into the new feature directory. Raised
    /// before the directory is created, so a missing template leaves no
    /// half-scaffolded feature behind.
    #[error("spec template not found (tried {tried})")]
    TemplateNotFound {
        /// Comma-separated repo-relative candidate paths that were tried.
        tried: String,
    },
    /// Feature path supplied to a primitive does not exist.
    #[error("feature path does not exist: {path}")]
    FeaturePathNotFound {
        /// Caller-supplied feature path that did not resolve to a directory.
        path: PathBuf,
    },
    /// Slug supplied by a caller failed the slug-grammar allowlist
    /// (`^[a-z0-9]+(?:-[a-z0-9]+)*$`, BE-INPUT-002) — empty, or holding a
    /// character outside lowercase-alphanumeric-plus-single-hyphen (path
    /// separators, dots, whitespace, control characters, uppercase).
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
    /// A supplied argument value failed validation (e.g., embedded
    /// newlines in a single-line field). Distinct from
    /// [`PrimitiveError::MissingArgument`] — the value was supplied but
    /// rejected.
    #[error("{primitive}: invalid '{argument}': {reason}")]
    InvalidArgument {
        /// Primitive name (e.g., `append-task`).
        primitive: String,
        /// Argument name carrying the rejected value.
        argument: String,
        /// One-line reason describing the rejection.
        reason: String,
    },
    /// `set-status` was invoked with a `from` or `to` value outside the
    /// constitution's lifecycle set. Transition-edge legality stays with
    /// procedures; the primitive guards set membership only.
    #[error("set-status: '{argument}' value '{value}' is not one of {allowed}")]
    InvalidStatus {
        /// Argument name (`from` or `to`) carrying the invalid value.
        argument: String,
        /// The rejected status value.
        value: String,
        /// Pipe-joined allowed lifecycle set.
        allowed: String,
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
    /// TOML parse failure (e.g., `dashboard` reading a malformed
    /// `.govern.toml`).
    #[error("TOML parse error in {path}: {source}")]
    Toml {
        /// Path of the file whose TOML failed to parse.
        path: PathBuf,
        /// Underlying TOML error.
        #[source]
        source: toml::de::Error,
    },
    /// Spec directory missing its `spec.md` file. `dashboard` raises this
    /// when an `NNN-feature` directory under the configured spec-root lacks
    /// the expected `spec.md` — the directory naming convention promises one.
    #[error("missing spec.md in {root}/{feature}")]
    MissingSpecFile {
        /// Configured spec-root directory name (default `specs`; spec 040).
        root: String,
        /// Feature directory name that lacks a `spec.md`.
        feature: String,
    },
    /// Feature directory exists but has no `tasks.md`. `prune-tasks` raises
    /// this so the command can direct the user to run the plan phase.
    #[error("tasks.md not found: {root}/{feature}/tasks.md")]
    TasksFileMissing {
        /// Configured spec-root directory name (default `specs`; spec 040).
        root: String,
        /// Feature whose tasks file is missing.
        feature: String,
    },
    /// `prune-tasks --reset` found a `tasks.md` with no `# …` heading, so it
    /// cannot preserve the feature identity for the reset. Writes nothing.
    #[error("malformed tasks.md at {path}: {reason}")]
    MalformedTasks {
        /// Path of the offending tasks file.
        path: PathBuf,
        /// One-line description of the structural problem.
        reason: String,
    },
    /// `[rules] surfaces` named a member outside `{backend, frontend}`.
    /// `discover-rule-files` fails fast rather than silently ignoring it.
    #[error(
        "invalid [rules] surfaces member \"{value}\" — accepted members are \"backend\" and \"frontend\" (use [] for cross-only; -cross.md files always apply)"
    )]
    InvalidSurfacesMember {
        /// The offending member string.
        value: String,
    },
    /// `[rules] surfaces` was set to something other than a list of strings.
    #[error("[rules] surfaces must be a list of strings, got {got}")]
    InvalidSurfacesType {
        /// Human-readable description of the actual type found.
        got: String,
    },
}

/// Convenience alias for primitive return values.
pub type Result<T> = std::result::Result<T, PrimitiveError>;

/// Split a markdown file's content into its frontmatter YAML block and the
/// body that follows. Returns an error if no `---` opening fence is present
/// or no closing fence is found.
pub(crate) fn split_frontmatter<'a>(content: &'a str, path: &Path) -> Result<(&'a str, &'a str)> {
    let (fm_text, body, _fm_offset) = split_frontmatter_with_offset(content, path)?;
    Ok((fm_text, body))
}

/// Like [`split_frontmatter`], but also returns the byte offset of the
/// frontmatter text within `content` — the length of the opener fence that
/// actually matched (4 for `---\n`, 5 for `---\r\n`). Callers that splice
/// edits back into the full file (e.g. `set-status`) need the real offset;
/// hardcoding the LF opener corrupts CRLF checkouts by one byte.
pub(crate) fn split_frontmatter_with_offset<'a>(
    content: &'a str,
    path: &Path,
) -> Result<(&'a str, &'a str, usize)> {
    let (after_open, fm_offset) = ["---\n", "---\r\n"]
        .iter()
        .find_map(|opener| {
            content
                .strip_prefix(opener)
                .map(|rest| (rest, opener.len()))
        })
        .ok_or_else(|| PrimitiveError::MissingFrontmatter { path: path.into() })?;

    // Empty frontmatter (`---\n---\n`): the closing fence is the very next
    // line, so there is no preceding newline for the `\n---` search below
    // to find. Present-but-empty frontmatter is a validation concern
    // (missing required fields), not a missing-frontmatter halt.
    for fence in ["---\n", "---\r\n"] {
        if let Some(body) = after_open.strip_prefix(fence) {
            return Ok(("", body, fm_offset));
        }
    }

    for fence in ["\n---\n", "\n---\r\n"] {
        if let Some(idx) = after_open.find(fence) {
            return Ok((
                &after_open[..idx],
                &after_open[idx + fence.len()..],
                fm_offset,
            ));
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

/// Read the frontmatter `status` value from a markdown document's content,
/// collapsing every unreadability (missing/unclosed frontmatter, invalid
/// YAML, frontmatter that doesn't parse as spec
/// [`Frontmatter`](crate::schema::primitives::Frontmatter), missing or
/// non-string `status`) to `None`.
///
/// The shared READ-only status reader: `traverse-deps` (dependency status),
/// `resolve-references` (linked-spec status, with its own membership policy
/// on top), and `check-stuck` (spec blobs from git history) all consume it.
/// The write path (`set-status`) keeps its span-preserving
/// `locate_status_field` instead — it must splice the value in place, not
/// just read it.
pub(crate) fn frontmatter_status(content: &str, path: &Path) -> Option<String> {
    let (fm_text, _body) = split_frontmatter(content, path).ok()?;
    serde_norway::from_str::<crate::schema::primitives::Frontmatter>(fm_text)
        .ok()
        .map(|fm| fm.status)
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
    // Capture the destination's existing mode (Unix) so an in-place rewrite
    // preserves it. `NamedTempFile` is created 0600 and `persist` renames it
    // over the target, so without this every rewrite would narrow an existing
    // 0644 file to owner-only. New files keep the tempfile default; a
    // primitive that writes an *executable* re-applies its mode after this
    // returns (see `apply-manifest`'s `mirror_source_mode`).
    #[cfg(unix)]
    let prior_mode = {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .ok()
            .map(|meta| meta.permissions().mode())
    };
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|source| PrimitiveError::Io {
            path: parent.into(),
            source,
        })?;
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
    #[cfg(unix)]
    if let Some(mode) = prior_mode {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(mode)).map_err(
            |source| PrimitiveError::Io {
                path: path.into(),
                source,
            },
        )?;
    }
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

    /// Parse a checkbox line into `(checked, text)`, where `text` is the
    /// trimmed content after the `]`. Recognition delegates to
    /// [`find_checkbox_line`], so the read side (`read-spec` criteria,
    /// `read-tasks` subtasks) and the mark side (`mark-task`,
    /// `mark-criterion`) share one grammar — the read/mark index contract
    /// requires that a checkbox counted by a reader is addressable by the
    /// matching marker, and vice versa.
    pub(crate) fn parse_checkbox_line(line: &str) -> Option<(bool, String)> {
        let (_bracket, marker_idx) = find_checkbox_line(line)?;
        let checked = matches!(line.as_bytes()[marker_idx], b'x' | b'X');
        let text = line[marker_idx + 2..].trim().to_string();
        Some((checked, text))
    }
}

/// Resolve a caller-supplied path argument against the repo root, accepting
/// absolute paths as-is.
///
/// This is the accept-absolute-paths counterpart to
/// [`validate_no_traversal`]: primitives whose path arguments are
/// operator/machine-local (fixture specs in temp dirs, generator scripts,
/// downloaded archives, sibling-service checkouts) resolve through this
/// helper; primitives that must stay inside the repo root
/// (`merge-managed-block`, `merge-permissions`, …) call
/// [`validate_no_traversal`] first and never accept absolute input.
/// `enforce-manifest` keeps its own stricter `resolve_contained_dir`
/// (absolute allowed only under the repo root) because its cleanup loop is
/// destructive.
pub(crate) fn resolve_path(repo: &Path, path_arg: &str) -> PathBuf {
    let candidate = Path::new(path_arg);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo.join(candidate)
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
    // `has_root()` in addition to `is_absolute()`: on Windows a `/`- or
    // `\`-rooted path without a drive letter is not "absolute", yet it
    // still escapes the repo (it resolves against the drive root). The
    // prefix check additionally rejects drive-relative forms (`C:foo`)
    // and UNC prefixes, which carry no root but name another location.
    let has_prefix = p
        .components()
        .next()
        .is_some_and(|c| matches!(c, std::path::Component::Prefix(_)));
    if p.is_absolute() || p.has_root() || has_prefix {
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

/// Validate a caller-supplied slug against the framework slug grammar
/// `^[a-z0-9]+(?:-[a-z0-9]+)*$`: one or more lowercase-alphanumeric
/// segments joined by single hyphens — exactly the alphabet
/// `create_feature::derive_slug` emits. This is an allowlist
/// (BE-INPUT-002): every slug reaches a written filename
/// (`scenarios/{slug}.md`) and a rendered heading, so anything outside the
/// grammar — uppercase, `_`, `.`, path separators, whitespace, newlines,
/// or other control characters — is rejected before it can inject a path
/// segment or forge markdown structure.
/// Reject a text value carrying an embedded newline or carriage return.
/// Such a value, interpolated verbatim into a markdown or YAML artifact,
/// would inject document structure (a phantom heading, a new frontmatter
/// key); `primitive`/`argument` name the offending field. Shared by every
/// primitive that splices caller-supplied text into a file it writes.
pub(crate) fn validate_single_line(primitive: &str, argument: &str, value: &str) -> Result<()> {
    if value.contains('\n') || value.contains('\r') {
        return Err(PrimitiveError::InvalidArgument {
            primitive: primitive.into(),
            argument: argument.into(),
            reason: "embedded newlines would inject document structure; \
                     supply single-line text"
                .into(),
        });
    }
    Ok(())
}

pub(crate) fn validate_slug(slug: &str) -> Result<()> {
    if slug.is_empty() {
        return Err(PrimitiveError::InvalidSlug {
            slug: slug.into(),
            reason: "slug is empty".into(),
        });
    }
    // Allowlist, segment by segment: each `-`-delimited segment must be
    // non-empty (rejecting a leading/trailing hyphen and a `--` run) and
    // hold only lowercase ASCII letters and digits.
    for segment in slug.split('-') {
        if segment.is_empty()
            || !segment
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit())
        {
            return Err(PrimitiveError::InvalidSlug {
                slug: slug.into(),
                reason: "slug must match ^[a-z0-9]+(?:-[a-z0-9]+)*$ \
                         (lowercase letters, digits, single hyphens)"
                    .into(),
            });
        }
    }
    Ok(())
}

/// Per-line skip state shared by the `tasks.md` / spec structural walkers.
/// Content inside a fenced code block (` ``` `) or an HTML comment
/// (`<!-- … -->`) is not markdown structure — it must not yield headings,
/// task numbers, phase containers, or checkboxes. Feed every line of a
/// document in order; [`skip`](SkipScanner::skip) reports the lines to ignore.
///
/// This exists because `tasks.md`'s own template guidance comment embeds
/// example `## N.` task headings; without comment-awareness the tasks
/// parsers mis-read a reset (template-state) file as containing phantom
/// tasks, splitting the runtime/markdown two-paths guarantee.
#[derive(Default)]
pub(crate) struct SkipScanner {
    in_fence: bool,
    in_comment: bool,
}

impl SkipScanner {
    /// Advance over `line` (document order) and report whether its content
    /// must be skipped. Fence and multi-line-comment delimiter lines are
    /// themselves skipped, matching the pre-existing fenced-block handling.
    /// A comment that opens and closes on the same line is inline — its
    /// surrounding content is real markdown and the line is not skipped.
    pub(crate) fn skip(&mut self, line: &str) -> bool {
        if self.in_fence {
            if line.trim_start().starts_with("```") {
                self.in_fence = false;
            }
            return true;
        }
        if self.in_comment {
            if line.contains("-->") {
                self.in_comment = false;
            }
            return true;
        }
        if line.trim_start().starts_with("```") {
            self.in_fence = true;
            return true;
        }
        if let Some(open) = line.find("<!--") {
            if line[open + 4..].contains("-->") {
                return false;
            }
            self.in_comment = true;
            return true;
        }
        false
    }
}

/// Walk `content` line by line, yielding the numeric prefix of every ATX
/// heading at any of the given `levels` whose text begins with `N.`. Skips
/// headings inside fenced code blocks and HTML comments. Used by `tasks.md`
/// primitives to
/// compute the next task number in both flat (`## N.`) and phased
/// (`### N.` under `## Phase X`) structures — passing `&[2, 3]` produces
/// the union across both shapes.
pub(crate) fn iter_task_numbers_at_levels<'a>(
    content: &'a str,
    levels: &'a [u8],
) -> impl Iterator<Item = u32> + 'a {
    let mut skip = SkipScanner::default();
    content.lines().filter_map(move |line| {
        if skip.skip(line) {
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
    let mut skip = SkipScanner::default();
    let lines: Vec<&str> = content.lines().collect();
    for (idx, line) in lines.iter().enumerate() {
        if skip.skip(line) {
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

/// Yield the body lines inside the section with heading `heading`, in
/// document order. The section ends at the next ATX heading whose level
/// is `<=` the matched heading's level (a sibling or shallower heading),
/// or at EOF. Lines INSIDE the section — including blank lines and any
/// nested deeper-level headings — are yielded as-is so consumers can
/// apply their own filters. When the heading appears more than once,
/// lines from every matching section are yielded in document order.
///
/// Shared between `read_spec::parse_open_questions` (returns
/// `Vec<OpenQuestion>`) and `dashboard::{count_open_questions,
/// context_summary}` (return a `u32` count and a `String` summary
/// respectively). The iteration semantics are the single source of
/// truth for "lines inside section X"; consumers diverge only in how
/// they fold the yielded lines into their result shape.
pub(crate) fn section_lines<'a>(body: &'a str, heading: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut in_section = false;
    let mut section_level: u8 = 0;
    for line in body.lines() {
        if let Some((level, h)) = parse_atx_heading(line) {
            if in_section && level <= section_level {
                in_section = false;
            }
            if h == heading {
                in_section = true;
                section_level = level;
                continue;
            }
        }
        if in_section {
            out.push(line);
        }
    }
    out
}

/// Comment/fence-aware variant of [`section_lines`]: returns the 0-based
/// indices (into `lines`) of the content lines inside the section named
/// `heading`, applying [`SkipScanner`] semantics to the whole document.
/// Lines inside fenced code blocks or HTML comments are neither yielded
/// nor treated as section-boundary headings, so a template guidance
/// comment that embeds example checkboxes or headings contributes
/// nothing. As with [`section_lines`], every matching section's lines are
/// yielded in document order when the heading repeats.
///
/// This is the single source of truth for "structural lines inside
/// section X": `read-spec`'s acceptance-criteria walk and
/// `mark-criterion`'s checkbox addressing both consume it, which keeps
/// their criterion indexes in lockstep (the two-paths guarantee).
pub(crate) fn section_line_indices(lines: &[&str], heading: &str) -> Vec<usize> {
    let mut out = Vec::new();
    let mut skip = SkipScanner::default();
    let mut in_section = false;
    let mut section_level: u8 = 0;
    for (idx, line) in lines.iter().enumerate() {
        if skip.skip(line) {
            continue;
        }
        if let Some((level, h)) = parse_atx_heading(line) {
            if in_section && level <= section_level {
                in_section = false;
            }
            if h == heading {
                in_section = true;
                section_level = level;
                continue;
            }
        }
        if in_section {
            out.push(idx);
        }
    }
    out
}

/// `true` when `name` matches the `NNN-feature` convention: three ASCII
/// digits, a literal hyphen, and at least one trailing character. Used
/// by primitives that walk `specs/` and need to distinguish feature
/// directories from sibling artifacts (`templates/`, `inbox.md`, ad-hoc
/// notes, dotfiles).
pub(crate) fn is_feature_slug(name: &str) -> bool {
    let bytes = name.as_bytes();
    bytes.len() >= 5
        && bytes[0].is_ascii_digit()
        && bytes[1].is_ascii_digit()
        && bytes[2].is_ascii_digit()
        && bytes[3] == b'-'
}

/// Parse a feature directory's three-digit `NNN-` prefix into its numeric
/// value. `None` when the first three bytes aren't a parseable number
/// (callers typically pre-filter with [`is_feature_slug`], so this is
/// belt-and-suspenders). Shared by `resolve-feature` (numeric-identifier
/// match) and `create-feature` (next-number computation).
pub(crate) fn feature_number(name: &str) -> Option<u32> {
    name.get(..3)?.parse::<u32>().ok()
}

/// List feature directories (`NNN-slug`) under the spec root, sorted by
/// name. Best-effort: a missing or unreadable spec root yields an empty
/// list — a repo without a spec root has no features by definition, and
/// the primitives that consume this (`resolve-feature`, `create-feature`,
/// `dashboard`, and `interpreter::payload`'s inbox router) all report the
/// empty case as "no features" rather than an operational error.
pub(crate) fn list_feature_dirs(specs_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(specs_dir) else {
        return Vec::new();
    };
    let mut features: Vec<String> = entries
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_dir())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| is_feature_slug(name))
        .collect();
    features.sort();
    features
}

/// List the scenario markdown files directly under `scenarios_dir`, sorted
/// by filename. Matches the `.md` extension CASE-INSENSITIVELY so `FOO.MD`
/// and `foo.md` are both scenarios — the two consuming surfaces
/// (`dashboard` counts, `check-artifacts` derives slugs) must agree on the
/// same set. Subdirectories and non-markdown files are excluded; an absent
/// or unreadable directory yields an empty list.
pub(crate) fn list_scenario_files(scenarios_dir: &Path) -> Vec<String> {
    let Ok(entries) = std::fs::read_dir(scenarios_dir) else {
        return Vec::new();
    };
    let mut files: Vec<String> = entries
        .filter_map(std::result::Result::ok)
        .filter(|e| e.path().is_file())
        .filter_map(|e| e.file_name().into_string().ok())
        .filter(|name| {
            Path::new(name)
                .extension()
                .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        })
        .collect();
    files.sort();
    files
}

/// Scenario frontmatter shape: `section` is the post-017 field; `spec-ref`
/// is the pre-017 legacy field still present on older scenarios. The single
/// shared definition for `resolve-feature`'s scenario detail and
/// `dashboard`'s session-target detail, so both surfaces fold
/// `section.or(spec-ref)` identically.
#[derive(serde::Deserialize)]
pub(crate) struct ScenarioFrontmatter {
    /// Post-017 section label the scenario belongs to.
    #[serde(default)]
    pub(crate) section: Option<String>,
    /// Pre-017 legacy field, read only as a fallback for `section`.
    #[serde(default, rename = "spec-ref")]
    pub(crate) spec_ref: Option<String>,
}

/// Best-effort read of a scenario file's `section` (or legacy `spec-ref`)
/// frontmatter field. `None` when the file is unreadable, has no
/// frontmatter, or the frontmatter fails to parse — every consumer degrades
/// an unreadable scenario to a detail-less entry rather than an error.
pub(crate) fn read_scenario_section(path: &Path) -> Option<String> {
    let content = read_text(path).ok()?;
    let (fm_text, _body) = split_frontmatter(&content, path).ok()?;
    let fm = serde_norway::from_str::<ScenarioFrontmatter>(fm_text).ok()?;
    fm.section.or(fm.spec_ref)
}

/// The two candidate locations for a spec-pipeline template file, in
/// resolution order: the installed adopter layout
/// `{specs-root}/templates/{file}` (what `/gov:init` scaffolds and the
/// command prose names) first, then the framework source layout
/// `framework/templates/spec/{file}` (the govern repo itself). Shared by
/// `create-feature`'s template copy and the `writeSpecBody` request builder
/// (`interpreter::payload::load_template`); each caller keeps its own
/// missing-template policy.
pub(crate) fn template_candidates(specs_root: &str, file: &str) -> [String; 2] {
    [
        format!("{specs_root}/templates/{file}"),
        format!("framework/templates/spec/{file}"),
    ]
}

/// Parse the `## Affected Files` markdown table in a plan body and return
/// the first-column path entries in document order. Tolerates rows with
/// backtick-wrapped paths and skips the header separator row. Shared by the
/// writeCode plan reader (`interpreter::payload`) and `compute-review-scope`
/// so both readers agree on the one canonical plan format (a table; see spec
/// 022 task 47).
pub(crate) fn parse_affected_files(plan_content: &str) -> Vec<String> {
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

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[cfg(unix)]
    #[test]
    fn write_atomic_preserves_existing_file_mode() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("spec.md");
        std::fs::write(&path, "old").unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        write_atomic(&path, "new").unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(
            mode, 0o644,
            "in-place rewrite must not narrow the file mode"
        );
        assert_eq!(std::fs::read_to_string(&path).unwrap(), "new");
    }

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
    fn validate_slug_rejects_newlines_and_control_chars() {
        // The denylist that BE-INPUT-002 replaced admitted these into a
        // written filename and a rendered heading; the allowlist rejects
        // every character outside `[a-z0-9-]`.
        for bad in &["a\nb", "a\rb", "a\tb", "a b", "a\u{7f}b", "a\0b"] {
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
    fn validate_slug_rejects_non_grammar_shapes() {
        // Uppercase, underscores, dots, and leading/trailing/repeated
        // hyphens all fall outside ^[a-z0-9]+(?:-[a-z0-9]+)*$.
        for bad in &["Upper", "a_b", "-lead", "trail-", "a--b", "a.b", "café"] {
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
    fn validate_slug_accepts_grammar_conformant_slugs() {
        for good in &["a", "022", "retry-on-timeout", "spec-042-foo", "x1-y2-z3"] {
            validate_slug(good).unwrap();
        }
    }

    #[test]
    fn feature_number_parses_nnn_prefix() {
        assert_eq!(feature_number("022-deterministic-runtime"), Some(22));
        assert_eq!(feature_number("007-webhooks"), Some(7));
        assert_eq!(feature_number("abc-nope"), None);
        assert_eq!(feature_number("22"), None); // too short for a 3-byte slice
    }

    #[test]
    fn template_candidates_orders_adopter_layout_first() {
        assert_eq!(
            template_candidates("specs", "spec.md"),
            [
                "specs/templates/spec.md".to_string(),
                "framework/templates/spec/spec.md".to_string(),
            ]
        );
        // Honors a configured spec root in the first candidate only.
        assert_eq!(
            template_candidates("governance", "plan.md")[0],
            "governance/templates/plan.md"
        );
    }

    #[test]
    fn list_scenario_files_matches_md_case_insensitively() {
        let tmp = tempfile::tempdir().unwrap();
        let dir = tmp.path().join("scenarios");
        std::fs::create_dir_all(&dir).unwrap();
        for name in ["alpha.md", "BETA.MD", "notes.txt", "README"] {
            std::fs::write(dir.join(name), "x").unwrap();
        }
        std::fs::create_dir_all(dir.join("nested.md")).unwrap(); // a dir, excluded
        assert_eq!(
            list_scenario_files(&dir),
            vec!["BETA.MD".to_string(), "alpha.md".to_string()]
        );
        // Absent directory degrades to empty.
        assert!(list_scenario_files(&tmp.path().join("missing")).is_empty());
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
    fn resolve_path_joins_relative_and_passes_absolute_through() {
        let repo = Path::new("/repo");
        assert_eq!(
            resolve_path(repo, "specs/001-basic/spec.md"),
            Path::new("/repo/specs/001-basic/spec.md")
        );
        assert_eq!(
            resolve_path(repo, "/tmp/x/spec.md"),
            Path::new("/tmp/x/spec.md")
        );
    }

    #[test]
    fn frontmatter_status_reads_string_status() {
        let content = "---\nstatus: planned\ndependencies: []\n---\n\n# X\n";
        assert_eq!(
            frontmatter_status(content, Path::new("spec.md")).as_deref(),
            Some("planned")
        );
    }

    #[test]
    fn frontmatter_status_collapses_unreadable_shapes_to_none() {
        for content in &[
            "# No frontmatter\n",                           // missing block
            "---\nstatus: [unterminated\n---\n# X\n",       // invalid YAML
            "---\ndependencies: []\n---\n# X\n",            // status missing
            "---\nstatus: [a, b]\ndependencies: []\n---\n", // non-string status
        ] {
            assert_eq!(
                frontmatter_status(content, Path::new("spec.md")),
                None,
                "expected None for {content:?}"
            );
        }
    }

    #[test]
    fn parse_checkbox_line_matches_mark_side_grammar() {
        // Accepted: the exact grammar find_checkbox_line recognizes.
        assert_eq!(
            checkbox::parse_checkbox_line("- [ ] pending item"),
            Some((false, "pending item".to_string()))
        );
        assert_eq!(
            checkbox::parse_checkbox_line("  - [x] done item"),
            Some((true, "done item".to_string()))
        );
        assert_eq!(
            checkbox::parse_checkbox_line("- [X] done upper"),
            Some((true, "done upper".to_string()))
        );
        // Bare checkbox (nothing after `]`) is a checkbox with empty text —
        // the mark side can flip it, so the read side must count it.
        assert_eq!(
            checkbox::parse_checkbox_line("- [ ]"),
            Some((false, String::new()))
        );
        // Rejected: divergent-grammar shapes the mark side cannot address.
        for not_a_checkbox in &[
            "- [-] partial",
            "- [x]no-space",
            "* [ ] star bullet",
            "- foo",
        ] {
            assert_eq!(
                checkbox::parse_checkbox_line(not_a_checkbox),
                None,
                "expected rejection for {not_a_checkbox:?}"
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
        let content = "## Setup\n\n## 1. First\n\n## 7. Seventh\n";
        let nums: Vec<u32> = iter_task_numbers_at_levels(content, &[2]).collect();
        assert_eq!(nums, vec![1, 7]);
    }

    #[test]
    fn section_lines_yields_section_body_until_sibling_heading() {
        let body = "## A\n\nline A1\nline A2\n\n## B\n\nline B1\n";
        let a = section_lines(body, "A");
        assert_eq!(a, vec!["", "line A1", "line A2", ""]);
        let b = section_lines(body, "B");
        assert_eq!(b, vec!["", "line B1"]);
    }

    #[test]
    fn section_lines_yields_nothing_for_absent_heading() {
        let body = "## Other\n\nx\n";
        assert!(section_lines(body, "Missing").is_empty());
    }

    #[test]
    fn section_lines_keeps_deeper_nested_headings_as_body_content() {
        // A `### nested` heading INSIDE `## A` is body content, not a
        // section boundary — section ends only at <= same-level heading.
        let body = "## A\n\n### nested\n\nx\n\n## B\n";
        let a = section_lines(body, "A");
        assert_eq!(a, vec!["", "### nested", "", "x", ""]);
    }

    #[test]
    fn section_lines_handles_repeated_heading() {
        // When the same heading appears more than once, lines from every
        // matching section are yielded in document order.
        let body = "## A\n\nfirst\n\n## B\n\nx\n\n## A\n\nsecond\n";
        let a = section_lines(body, "A");
        assert_eq!(a, vec!["", "first", "", "", "second"]);
    }

    #[test]
    fn section_line_indices_skips_comment_and_fence_content() {
        let body = "## A\n\n<!--\n- [ ] fake\n-->\n- [ ] real\n```\n- [ ] fenced\n```\n\n## B\n";
        let lines: Vec<&str> = body.lines().collect();
        // Comment and fence content (delimiter lines included) is skipped;
        // only the blank lines and the real checkbox line survive.
        assert_eq!(section_line_indices(&lines, "A"), vec![1, 5, 9]);
    }

    #[test]
    fn section_line_indices_ignores_headings_inside_comments() {
        // A sibling heading inside a comment must not close the section.
        let body = "## A\n\n<!--\n## B\n-->\n- [ ] still in A\n";
        let lines: Vec<&str> = body.lines().collect();
        assert_eq!(section_line_indices(&lines, "A"), vec![1, 5]);
    }

    #[test]
    fn section_line_indices_keeps_inline_comment_lines() {
        // A comment that opens and closes on the same line is inline — the
        // line is real content (documented SkipScanner delimiter behavior).
        let body = "## A\n- [ ] real <!-- note -->\n";
        let lines: Vec<&str> = body.lines().collect();
        assert_eq!(section_line_indices(&lines, "A"), vec![1]);
    }

    #[test]
    fn split_frontmatter_offset_matches_lf_opener() {
        let content = "---\nstatus: x\n---\nbody\n";
        let (fm, body, offset) =
            split_frontmatter_with_offset(content, Path::new("spec.md")).unwrap();
        assert_eq!(fm, "status: x");
        assert_eq!(body, "body\n");
        assert_eq!(offset, "---\n".len());
    }

    #[test]
    fn split_frontmatter_offset_matches_crlf_opener() {
        let content = "---\r\nstatus: draft\r\n---\r\n\r\nbody\r\n";
        let (fm, body, offset) =
            split_frontmatter_with_offset(content, Path::new("spec.md")).unwrap();
        assert_eq!(fm, "status: draft\r");
        assert_eq!(body, "\r\nbody\r\n");
        assert_eq!(offset, "---\r\n".len());
    }

    #[test]
    fn split_frontmatter_accepts_empty_block() {
        let (fm, body, offset) =
            split_frontmatter_with_offset("---\n---\nbody\n", Path::new("spec.md")).unwrap();
        assert_eq!(fm, "");
        assert_eq!(body, "body\n");
        assert_eq!(offset, "---\n".len());

        let (fm, body, offset) =
            split_frontmatter_with_offset("---\r\n---\r\n", Path::new("spec.md")).unwrap();
        assert_eq!(fm, "");
        assert_eq!(body, "");
        assert_eq!(offset, "---\r\n".len());
    }

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
    fn is_feature_slug_accepts_canonical_form() {
        for slug in &["022-deterministic-runtime", "000-blocker", "999-foo"] {
            assert!(is_feature_slug(slug), "expected acceptance for {slug:?}");
        }
    }

    #[test]
    fn is_feature_slug_rejects_non_pattern() {
        for bad in &[
            "templates",
            "inbox.md",
            ".hidden",
            "022",
            "abc-something",
            "22-too-short",
            "0220-too-long-prefix", // first 3 chars are digits but 4th isn't '-'
        ] {
            assert!(!is_feature_slug(bad), "expected rejection for {bad:?}");
        }
    }

    #[test]
    fn task_walkers_ignore_html_comment_headings() {
        // The tasks.md template guidance comment embeds `## 1.` example
        // headings; they must not be counted as tasks or flip structure
        // detection.
        let content = "# T\n\nIntro.\n\n<!-- Example:\n## 1. Not a task\n\n- [ ] not a subtask\n### 2. Also not\n-->\n\n## 1. Real task\n\n- [ ] real\n";
        let nums: Vec<u32> = iter_task_numbers_at_levels(content, &[2, 3]).collect();
        assert_eq!(
            nums,
            vec![1],
            "only the real `## 1.` outside the comment counts"
        );
        assert_eq!(detect_tasks_structure(content), TasksStructure::Flat);

        // A pure-comment example (no real tasks) yields nothing.
        let only_comment = "# T\n\n<!-- \n## 1. X\n### 2. Y\n-->\n";
        assert!(
            iter_task_numbers_at_levels(only_comment, &[2, 3])
                .next()
                .is_none()
        );

        // An inline self-closing comment on a heading line does not hide it.
        let inline = "## 3. Real <!-- note -->\n\n- [ ] x\n";
        let inline_nums: Vec<u32> = iter_task_numbers_at_levels(inline, &[2, 3]).collect();
        assert_eq!(inline_nums, vec![3]);
    }
}
