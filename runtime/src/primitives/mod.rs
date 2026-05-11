//! Deterministic primitive operations.
//!
//! Each primitive has a pure-Rust `run` function (no stdout/stderr I/O — the
//! caller wraps the result into a JSON envelope), a `clap`-derive args struct
//! from [`crate::schema::primitives`], and a unit test against a fixture file
//! under `runtime/tests/fixtures/primitives/`.

#![allow(clippy::module_name_repetitions)]

use std::path::{Path, PathBuf};

pub mod check_rule_ids;
pub mod check_stuck;
pub mod derive_boundary;
pub mod read_spec;
pub mod read_tasks;
pub mod resolve_anchor;
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
