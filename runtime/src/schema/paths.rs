//! `[paths]` block schema from `.govern.toml` — the configurable spec-root
//! directory name (spec 040).
//!
//! The top-level directory that holds every govern artifact defaults to
//! `specs`, but an adopter may rename it via `.govern.toml`:
//!
//! ```toml
//! [paths]
//! specs-root = "governance"
//! ```
//!
//! to avoid collisions with sibling-framework directories (the motivating
//! case is `RSpec`'s `spec/`, one character away from `specs/`). This module is
//! the single source of truth the runtime uses to resolve that name: every
//! primitive that today hardcodes `repo.join("specs")` calls [`specs_dir`]
//! instead, and the default keeps an adopter who never sets the key on exactly
//! today's behavior (`specs`). Per [§runtime-boundary], `.govern.toml` is the
//! git-tracked source of truth — the runtime reads the resolved name, it does
//! not own it.
//!
//! Resolution is best-effort, mirroring [`crate::host::Host::load`]: a missing
//! file, an absent block/key, an empty value, a malformed value, or an
//! unparseable document all fall back to the default `specs` (the malformed
//! cases log a one-line warning to stderr) so that path resolution never fails
//! because of an unrelated or operator config error. Hard rejection of a
//! malformed value at configuration time is the `/govern` markdown path's job;
//! [`validate_specs_root`] is the shared predicate both layers agree on.
//!
//! [§runtime-boundary]: `framework/constitution.md` §runtime-boundary

use std::path::{Path, PathBuf};

use serde::Deserialize;

/// Default spec-root directory name when `.govern.toml` does not configure
/// one. Matches the framework's historical hardcoded value, so an adopter who
/// never sets `[paths] specs-root` sees byte-for-byte identical behavior.
pub const DEFAULT_SPECS_ROOT: &str = "specs";

/// Validate a configured spec-root directory name for well-formedness.
///
/// A well-formed name is a single directory-name segment using only the
/// conservative charset `[A-Za-z0-9_-]` (letters, digits, hyphen, underscore)
/// and is non-empty. This is deliberately stricter than "no separators / no
/// `..`": the runtime uses the name only as a literal path component (safe at
/// any charset), but the bash generators interpolate it **unescaped** into
/// `grep -E` / awk regexes, where a `.`, `+`, `*`, `(`, … would act as a
/// regex metacharacter (over-matching, or a syntax error that silently drops a
/// spec). Restricting the charset keeps both sides safe with one rule and also
/// rejects a lone `.` (which would resolve the spec-root to the repo root).
/// See spec 040's review. The predicate is shared so the runtime's
/// best-effort resolver ([`Paths::load`]) and the `/govern` configuration
/// prompt apply the same rule.
///
/// # Errors
///
/// Returns a human-readable reason (suitable for a stderr warning or a
/// configuration-time rejection message) when `name` is malformed.
pub fn validate_specs_root(name: &str) -> Result<(), String> {
    if name.is_empty() {
        return Err("spec-root name must not be empty".to_owned());
    }
    // A path separator is the most common mistake — give it a specific message
    // before the general charset check.
    if name.contains('/') || name.contains('\\') {
        return Err(format!(
            "spec-root name must not contain a path separator: {name:?}"
        ));
    }
    if let Some(bad) = name
        .chars()
        .find(|c| !matches!(c, 'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_'))
    {
        return Err(format!(
            "spec-root name must use only letters, digits, '-', or '_': {name:?} (offending character {bad:?})"
        ));
    }
    Ok(())
}

/// Resolved `[paths]` configuration — currently just the spec-root directory
/// name. Loaded once per primitive invocation via [`Paths::load`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    /// The spec-root directory name (e.g., `specs` by default, or a
    /// configured value like `governance`). Always well-formed: a malformed
    /// configured value is rejected during load and replaced by the default.
    pub specs_root: String,
}

impl Paths {
    /// Resolve `[paths]` for `base` by reading `<base>/.govern.toml`.
    ///
    /// `base` is the directory whose `.govern.toml` governs the layout — the
    /// repo root in the common case, or a cross-service checkout root for
    /// `resolve-references` (each service may configure its own spec-root).
    ///
    /// Best-effort and infallible: a missing file, absent `[paths]` table,
    /// absent or empty `specs-root`, malformed value, or unparseable document
    /// all yield the default `specs`. The two malformed cases (bad value, bad
    /// document) log a one-line warning to stderr so the operator sees the
    /// fallback rather than silently getting `specs`.
    #[must_use]
    pub fn load(base: &Path) -> Self {
        let specs_root =
            Self::load_configured(base).unwrap_or_else(|| DEFAULT_SPECS_ROOT.to_owned());
        Self { specs_root }
    }

    /// Read and validate `[paths] specs-root` from `<base>/.govern.toml`.
    /// Returns `None` (→ caller uses the default) when the file is missing,
    /// the key is absent or empty, the value is malformed, or the document
    /// does not parse. Malformed value/document log to stderr.
    fn load_configured(base: &Path) -> Option<String> {
        let toml_path = base.join(".govern.toml");
        let content = std::fs::read_to_string(&toml_path).ok()?;
        let parsed: PathsFile = match toml::from_str(&content) {
            Ok(parsed) => parsed,
            Err(err) => {
                eprintln!(
                    "gvrn: failed to parse {} for [paths] block: {err}; using default spec-root {DEFAULT_SPECS_ROOT:?}",
                    toml_path.display()
                );
                return None;
            }
        };
        let raw = parsed.paths?.specs_root?;
        let name = raw.trim();
        if name.is_empty() {
            // Empty/whitespace value is treated as unset → default, no warning.
            return None;
        }
        match validate_specs_root(name) {
            Ok(()) => Some(name.to_owned()),
            Err(reason) => {
                eprintln!(
                    "gvrn: invalid [paths] specs-root {name:?} in {}: {reason}; using default {DEFAULT_SPECS_ROOT:?}",
                    toml_path.display()
                );
                None
            }
        }
    }
}

/// Resolve the absolute spec-root directory under `base`.
///
/// This is the single replacement for the historical `base.join("specs")`:
/// every runtime primitive that joins a bare feature name (or enumerates the
/// tree) under the spec root calls `specs_dir(base).join(feature)` instead, so
/// the configured root is honored uniformly. With no configuration it returns
/// `base/specs`, preserving today's behavior exactly.
#[must_use]
pub fn specs_dir(base: &Path) -> PathBuf {
    base.join(Paths::load(base).specs_root)
}

#[derive(Deserialize, Default)]
struct PathsFile {
    #[serde(default)]
    paths: Option<PathsBlock>,
}

#[derive(Deserialize, Default)]
struct PathsBlock {
    #[serde(default, rename = "specs-root")]
    specs_root: Option<String>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use tempfile::TempDir;

    fn tmp_repo() -> TempDir {
        tempfile::Builder::new()
            .prefix("govern-paths-fixture")
            .tempdir()
            .unwrap()
    }

    fn write_toml(dir: &Path, body: &str) {
        std::fs::write(dir.join(".govern.toml"), body).unwrap();
    }

    // --- validate_specs_root -------------------------------------------------

    #[test]
    fn validate_accepts_well_formed_names() {
        for name in [
            "specs",
            "governance",
            "gov-specs",
            "design",
            "specs_v2",
            "s",
        ] {
            assert!(
                validate_specs_root(name).is_ok(),
                "{name:?} should be accepted"
            );
        }
    }

    #[test]
    fn validate_rejects_empty() {
        assert!(validate_specs_root("").is_err());
    }

    #[test]
    fn validate_rejects_leading_slash() {
        assert!(validate_specs_root("/abs").is_err());
        assert!(validate_specs_root("/").is_err());
    }

    #[test]
    fn validate_rejects_path_separator() {
        assert!(validate_specs_root("a/b").is_err());
        assert!(validate_specs_root("nested/specs").is_err());
        assert!(validate_specs_root("a\\b").is_err());
    }

    #[test]
    fn validate_rejects_dot_dot() {
        assert!(validate_specs_root("..").is_err());
        assert!(validate_specs_root("../escape").is_err());
        assert!(validate_specs_root("a..b").is_err());
    }

    #[test]
    fn validate_rejects_regex_metachars_and_dot() {
        // Characters outside [A-Za-z0-9_-] are rejected so they cannot act as
        // regex metacharacters when the bash generators interpolate the name
        // (spec 040 review). A lone `.` (repo-root) is rejected too.
        for name in [
            ".", "v1.0", "a.b", "spec+s", "spec(s", "spec*s", "a b", "a[b",
        ] {
            assert!(
                validate_specs_root(name).is_err(),
                "{name:?} should be rejected"
            );
        }
    }

    // --- Paths::load / specs_dir: default fallbacks --------------------------

    #[test]
    fn missing_file_defaults_to_specs() {
        let repo = tmp_repo();
        assert_eq!(Paths::load(repo.path()).specs_root, "specs");
        assert_eq!(specs_dir(repo.path()), repo.path().join("specs"));
    }

    #[test]
    fn empty_file_defaults_to_specs() {
        let repo = tmp_repo();
        write_toml(repo.path(), "# empty\n");
        assert_eq!(Paths::load(repo.path()).specs_root, "specs");
    }

    #[test]
    fn paths_block_absent_defaults_to_specs() {
        let repo = tmp_repo();
        write_toml(repo.path(), "[review]\ntech-stack-verified = true\n");
        assert_eq!(Paths::load(repo.path()).specs_root, "specs");
    }

    #[test]
    fn specs_root_key_absent_defaults_to_specs() {
        let repo = tmp_repo();
        write_toml(repo.path(), "[paths]\nother = \"x\"\n");
        assert_eq!(Paths::load(repo.path()).specs_root, "specs");
    }

    #[test]
    fn empty_value_defaults_to_specs() {
        let repo = tmp_repo();
        write_toml(repo.path(), "[paths]\nspecs-root = \"\"\n");
        assert_eq!(Paths::load(repo.path()).specs_root, "specs");
    }

    #[test]
    fn whitespace_only_value_defaults_to_specs() {
        let repo = tmp_repo();
        write_toml(repo.path(), "[paths]\nspecs-root = \"   \"\n");
        assert_eq!(Paths::load(repo.path()).specs_root, "specs");
    }

    // --- Paths::load / specs_dir: configured override ------------------------

    #[test]
    fn configured_value_overrides_default() {
        let repo = tmp_repo();
        write_toml(repo.path(), "[paths]\nspecs-root = \"governance\"\n");
        assert_eq!(Paths::load(repo.path()).specs_root, "governance");
        assert_eq!(specs_dir(repo.path()), repo.path().join("governance"));
    }

    #[test]
    fn configured_value_is_trimmed() {
        let repo = tmp_repo();
        write_toml(repo.path(), "[paths]\nspecs-root = \"  governance  \"\n");
        assert_eq!(Paths::load(repo.path()).specs_root, "governance");
    }

    #[test]
    fn coexists_with_other_tables() {
        let repo = tmp_repo();
        write_toml(
            repo.path(),
            "[host]\nproject = \"anvil\"\n\n[paths]\nspecs-root = \"design\"\n\n[review]\ntech-stack-verified = true\n",
        );
        assert_eq!(Paths::load(repo.path()).specs_root, "design");
    }

    // --- Paths::load: malformed config falls back to default -----------------

    #[test]
    fn malformed_value_falls_back_to_default() {
        let repo = tmp_repo();
        write_toml(repo.path(), "[paths]\nspecs-root = \"../escape\"\n");
        assert_eq!(Paths::load(repo.path()).specs_root, "specs");
    }

    #[test]
    fn separator_value_falls_back_to_default() {
        let repo = tmp_repo();
        write_toml(repo.path(), "[paths]\nspecs-root = \"nested/specs\"\n");
        assert_eq!(Paths::load(repo.path()).specs_root, "specs");
    }

    #[test]
    fn malformed_document_falls_back_to_default() {
        let repo = tmp_repo();
        write_toml(repo.path(), "[paths\nbroken");
        assert_eq!(Paths::load(repo.path()).specs_root, "specs");
    }
}
