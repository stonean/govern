//! Host config loader — resolves the `{cli-config-dir}` and `{project}`
//! template variables used to locate slash-command files at `gvrn exec`
//! time. See spec 022 scenario `commands-dir-parameterization`.
//!
//! The runtime resolves command files at two callsites
//! (`main::run_exec` and `interpreter::payload::locate_command_file`),
//! both of which used to bake in Claude Code's config-dir name and
//! this repo's slash-command namespace. This module reads `project` from
//! `.govern.toml`'s `[host]` block (team-shared — the slash-command
//! namespace is identical for every contributor) and `cli-config-dir` from
//! the gitignored, per-contributor `.govern.session.toml` (teammates may
//! each use a different agent, so the config-dir name must never be
//! committed). For adopters predating that relocation, `cli-config-dir`
//! falls back to the legacy `.govern.toml` `[host]` value, then to defaults
//! that preserve the framework repo's behavior (`.claude` and the repo
//! directory basename).
//!
//! The two callsites resolve the installed command file via
//! [`Host::command_file_candidates`], which covers both flat-namespaced
//! layouts: `claude-style`'s `commands/` (Claude, Auggie) and `opencode`'s
//! singular `command/`.

use std::path::Path;

use serde::Deserialize;

use crate::primitives::write_session::SESSION_FILE;

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
    /// Load `Host` for `repo`: `project` from `.govern.toml`'s `[host]`
    /// block (team-shared), `cli-config-dir` from the per-contributor
    /// `.govern.session.toml` with a legacy `.govern.toml` `[host]` fallback.
    /// Returns defaults (`.claude` / repo directory basename) for any value
    /// not found in those sources.
    ///
    /// A malformed `.govern.toml` or `.govern.session.toml` is treated as
    /// absent (the `.govern.toml` case logs a warning to stderr) — command
    /// resolution should not fail because of an unrelated config error.
    /// Mismatches between the resolved values and the on-disk layout surface
    /// as the existing "command file not found" error at lookup time.
    #[must_use]
    pub fn load(repo: &Path) -> Self {
        let defaults = Self::defaults(repo);
        let host_block = Self::load_host_block(repo);
        // `project` is shared across the team — it names the slash-command
        // namespace and is identical for every contributor — so it stays in
        // the committed `.govern.toml` `[host]` block.
        let project = host_block
            .as_ref()
            .and_then(|b| b.project.clone())
            .unwrap_or(defaults.project);
        // `cli-config-dir` is per-contributor: teammates on one project may
        // each use a different agent (`.claude` / `.augment` / `.opencode` /
        // `.agents`), so it must NOT live in committed config. Prefer the
        // gitignored `.govern.session.toml`; fall back to the legacy
        // `.govern.toml` `[host]` value for adopters predating the
        // relocation; then the default.
        let cli_config_dir = Self::load_session_cli_config_dir(repo)
            .or_else(|| host_block.and_then(|b| b.cli_config_dir))
            .unwrap_or(defaults.cli_config_dir);
        Self {
            cli_config_dir,
            project,
        }
    }

    /// Read the `[host]` block from `<repo>/.govern.toml`. Returns `None`
    /// when the file is missing, has no `[host]` block, or fails to parse
    /// (a parse error logs to stderr and yields `None` — command resolution
    /// should not fail because of an unrelated config error).
    fn load_host_block(repo: &Path) -> Option<HostBlock> {
        let toml_path = repo.join(".govern.toml");
        let content = std::fs::read_to_string(&toml_path).ok()?;
        match toml::from_str::<HostFile>(&content) {
            Ok(parsed) => parsed.host,
            Err(err) => {
                eprintln!(
                    "gvrn: failed to parse {} for [host] block: {err}; using defaults",
                    toml_path.display()
                );
                None
            }
        }
    }

    /// Read the per-contributor `cli-config-dir` from the gitignored
    /// `<repo>/.govern.session.toml`. Best-effort: a missing or malformed
    /// session file yields `None` so resolution falls through to the legacy
    /// `.govern.toml` value and then the default.
    fn load_session_cli_config_dir(repo: &Path) -> Option<String> {
        let session_path = repo.join(SESSION_FILE);
        let content = std::fs::read_to_string(&session_path).ok()?;
        toml::from_str::<SessionHost>(&content).ok()?.cli_config_dir
    }

    /// Repo-relative paths where an installed slash-command file named
    /// `command_name` may live, in resolution order. Covers the two
    /// flat-namespaced command layouts the runtime knows about:
    ///
    /// - `claude-style` (Claude Code, Auggie) — `{dir}/commands/{project}/<name>.md`
    /// - `opencode` — `{dir}/command/{project}/<name>.md` (singular `command/`)
    ///
    /// Each adopter installs into exactly one of these (selected by the
    /// agent's registry `layout`), and the directory names are agent-specific
    /// via `cli_config_dir` (`.claude` / `.augment` vs `.opencode`), so the
    /// two candidates never both exist — trying both lets the runtime resolve
    /// any supported layout without knowing which agent wrote the file. The
    /// plural form is tried first, so existing claude-style adopters resolve
    /// exactly as before.
    #[must_use]
    pub fn command_file_candidates(&self, command_name: &str) -> Vec<String> {
        ["commands", "command"]
            .iter()
            .map(|subdir| {
                format!(
                    "{}/{subdir}/{}/{command_name}.md",
                    self.cli_config_dir, self.project
                )
            })
            .collect()
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

/// The per-contributor slice of `.govern.session.toml` this module reads —
/// the flat top-level `cli-config-dir` key. Other session keys (`feature`,
/// `path`, `set-at`, …) are ignored here; serde drops unknown fields.
#[derive(Deserialize)]
struct SessionHost {
    #[serde(default, rename = "cli-config-dir")]
    cli_config_dir: Option<String>,
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
    fn session_cli_config_dir_overrides_legacy_govern_toml() {
        // Per-contributor session file wins for `cli-config-dir`; `project`
        // still comes from the committed `.govern.toml`. This is the team
        // case: the committed config may say `.claude` (or nothing), but a
        // contributor using OpenCode resolves their own `.opencode`.
        let repo = tmp_repo("anvil-fixture");
        std::fs::write(
            repo.path().join(".govern.toml"),
            "[host]\ncli-config-dir = \".claude\"\nproject = \"anvil\"\n",
        )
        .unwrap();
        std::fs::write(
            repo.path().join(".govern.session.toml"),
            "feature = \"001-x\"\npath = \"specs/001-x\"\nset-at = \"2026-06-20T00:00:00Z\"\ncli-config-dir = \".opencode\"\n",
        )
        .unwrap();
        let host = Host::load(repo.path());
        assert_eq!(host.cli_config_dir, ".opencode");
        assert_eq!(host.project, "anvil");
    }

    #[test]
    fn session_cli_config_dir_used_when_no_legacy_block() {
        let repo = tmp_repo("anvil-fixture");
        std::fs::write(
            repo.path().join(".govern.session.toml"),
            "cli-config-dir = \".opencode\"\n",
        )
        .unwrap();
        let host = Host::load(repo.path());
        assert_eq!(host.cli_config_dir, ".opencode");
    }

    #[test]
    fn malformed_session_falls_back_to_legacy_then_default() {
        let repo = tmp_repo("anvil-fixture");
        std::fs::write(
            repo.path().join(".govern.toml"),
            "[host]\ncli-config-dir = \".augment\"\n",
        )
        .unwrap();
        std::fs::write(
            repo.path().join(".govern.session.toml"),
            "cli-config-dir = [broken\n",
        )
        .unwrap();
        let host = Host::load(repo.path());
        assert_eq!(host.cli_config_dir, ".augment");
    }

    #[test]
    fn command_file_candidates_cover_both_layouts_plural_first() {
        let host = Host {
            cli_config_dir: ".opencode".to_owned(),
            project: "anvil".to_owned(),
        };
        assert_eq!(
            host.command_file_candidates("specify"),
            vec![
                ".opencode/commands/anvil/specify.md".to_owned(),
                ".opencode/command/anvil/specify.md".to_owned(),
            ],
            "plural (claude-style) tried first, then singular (opencode)"
        );
    }

    #[test]
    fn malformed_toml_falls_back_to_defaults() {
        let repo = tmp_repo("govern-fixture");
        std::fs::write(repo.path().join(".govern.toml"), "[host\nbroken").unwrap();
        let host = Host::load(repo.path());
        assert_eq!(host.cli_config_dir, ".claude");
    }
}
