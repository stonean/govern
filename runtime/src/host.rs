//! Host config loader — resolves the `{cli-config-dir}` and `{project}`
//! template variables used to locate slash-command files at `gvrn exec`
//! time. See spec 022 scenario `commands-dir-parameterization`.
//!
//! The runtime resolves command files at two callsites
//! (`main::run_exec` and `interpreter::payload::locate_command_file`),
//! both of which used to bake in Claude Code's config-dir name and
//! this repo's slash-command namespace. This module reads the host's
//! values from `.govern.toml`'s `[host]` block and falls back to
//! defaults that preserve the framework repo's behavior (`.claude`
//! and the repo directory basename) when the block is absent.

use std::path::Path;

use serde::Deserialize;

/// Default `cli-config-dir` when `.govern.toml`'s `[host]` block is
/// missing the key. Matches the framework repo's own layout, so this
/// repo's behavior is unchanged when no `[host]` block is declared.
const DEFAULT_CLI_CONFIG_DIR: &str = ".claude";

/// Last-resort `project` fallback when the repo path has no
/// extractable file-name component (UTF-8-invalid name, root path,
/// trailing `..`). The normal fallback is the repo's directory
/// basename; this constant only fires on the degenerate path shape.
const FALLBACK_PROJECT: &str = "gov";

/// Resolved host config — the values both command-resolution callsites
/// need at lookup time. `cli_config_dir` is the host's per-user
/// config-dir name (e.g., `.claude` for Claude Code, `.augment` for
/// Auggie); `project` is the slash-command namespace under that dir
/// (e.g., `gov` in this repo, `anvil` for the Anvil adopter).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Host {
    /// Host's per-user config-dir name (e.g., `.claude`, `.augment`).
    pub cli_config_dir: String,
    /// Slash-command namespace under the config dir (e.g., `gov`).
    pub project: String,
}

impl Host {
    /// Load `Host` from `<repo>/.govern.toml`'s `[host]` block. Returns
    /// defaults (`.claude` / repo directory basename) when the file is
    /// missing, when the block is absent, or when individual fields
    /// are missing within the block.
    ///
    /// A malformed `.govern.toml` (TOML parse error) logs a warning to
    /// stderr and returns defaults — command resolution should not
    /// fail because of an unrelated config error. Mismatches between
    /// the resolved values and the on-disk layout surface as the
    /// existing "command file not found" error at lookup time.
    #[must_use]
    pub fn load(repo: &Path) -> Self {
        let defaults = Self::defaults(repo);
        let toml_path = repo.join(".govern.toml");
        let Ok(content) = std::fs::read_to_string(&toml_path) else {
            return defaults;
        };
        let parsed: HostFile = match toml::from_str(&content) {
            Ok(v) => v,
            Err(err) => {
                eprintln!(
                    "gvrn: failed to parse {} for [host] block: {err}; using defaults",
                    toml_path.display()
                );
                return defaults;
            }
        };
        let Some(block) = parsed.host else {
            return defaults;
        };
        Self {
            cli_config_dir: block.cli_config_dir.unwrap_or(defaults.cli_config_dir),
            project: block.project.unwrap_or(defaults.project),
        }
    }

    fn defaults(repo: &Path) -> Self {
        let project = repo
            .file_name()
            .and_then(|s| s.to_str())
            .map_or_else(|| FALLBACK_PROJECT.to_owned(), str::to_owned);
        Self {
            cli_config_dir: DEFAULT_CLI_CONFIG_DIR.to_owned(),
            project,
        }
    }
}

#[derive(Deserialize, Default)]
struct HostFile {
    #[serde(default)]
    host: Option<HostBlock>,
}

#[derive(Deserialize, Default)]
struct HostBlock {
    #[serde(default, rename = "cli-config-dir")]
    cli_config_dir: Option<String>,
    #[serde(default)]
    project: Option<String>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]
    use super::*;
    use tempfile::TempDir;

    fn tmp_repo(name: &str) -> TempDir {
        tempfile::Builder::new().prefix(name).tempdir().unwrap()
    }

    #[test]
    fn missing_file_returns_defaults() {
        let repo = tmp_repo("govern-fixture");
        let host = Host::load(repo.path());
        assert_eq!(host.cli_config_dir, ".claude");
        assert_eq!(
            host.project,
            repo.path().file_name().unwrap().to_str().unwrap()
        );
    }

    #[test]
    fn empty_file_returns_defaults() {
        let repo = tmp_repo("govern-fixture");
        std::fs::write(repo.path().join(".govern.toml"), "# empty\n").unwrap();
        let host = Host::load(repo.path());
        assert_eq!(host.cli_config_dir, ".claude");
    }

    #[test]
    fn host_block_absent_returns_defaults() {
        let repo = tmp_repo("govern-fixture");
        std::fs::write(
            repo.path().join(".govern.toml"),
            "[pins]\n\"foo\" = \"v1\"\n",
        )
        .unwrap();
        let host = Host::load(repo.path());
        assert_eq!(host.cli_config_dir, ".claude");
    }

    #[test]
    fn host_block_full_overrides_defaults() {
        let repo = tmp_repo("govern-fixture");
        std::fs::write(
            repo.path().join(".govern.toml"),
            "[host]\ncli-config-dir = \".augment\"\nproject = \"anvil\"\n",
        )
        .unwrap();
        let host = Host::load(repo.path());
        assert_eq!(host.cli_config_dir, ".augment");
        assert_eq!(host.project, "anvil");
    }

    #[test]
    fn host_block_partial_uses_defaults_for_missing() {
        let repo = tmp_repo("anvil-fixture");
        std::fs::write(
            repo.path().join(".govern.toml"),
            "[host]\nproject = \"anvil\"\n",
        )
        .unwrap();
        let host = Host::load(repo.path());
        assert_eq!(host.cli_config_dir, ".claude");
        assert_eq!(host.project, "anvil");
    }

    #[test]
    fn malformed_toml_falls_back_to_defaults() {
        let repo = tmp_repo("govern-fixture");
        std::fs::write(repo.path().join(".govern.toml"), "[host\nbroken").unwrap();
        let host = Host::load(repo.path());
        assert_eq!(host.cli_config_dir, ".claude");
    }
}
