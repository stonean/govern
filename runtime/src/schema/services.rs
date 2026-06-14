//! `[services]` registry schema from `.govern.toml`.
//!
//! Cross-service references (spec 030) resolve a linked spec's lifecycle
//! status by matching a reference link's repository URL against a registered
//! service's `repo`, then reading the linked spec from that service's local
//! `path`. This module is the pure shape plus parser; file IO and outcome
//! classification live in the `resolve-references` primitive.
//!
//! Schema is canonical in
//! `specs/030-cross-service-references/data-model.md`.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// One `[services.<alias>]` entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct ServiceEntry {
    /// Canonical repository URL — the identity matched against a reference
    /// link's href to decide registration.
    pub repo: String,
    /// Local checkout location (relative to the repo root or absolute) read
    /// for status resolution.
    pub path: String,
    /// Optional human/agent-facing note on the service's purpose.
    /// Informational only — no runtime behavior depends on it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// The `[services]` table: alias → entry. Empty when the table is absent.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Services(pub BTreeMap<String, ServiceEntry>);

/// Wrapper for extracting just the `[services]` table from `.govern.toml`.
/// Unknown top-level tables are accepted and ignored.
#[derive(Debug, Default, Deserialize)]
struct ServicesConfig {
    #[serde(default)]
    services: BTreeMap<String, ServiceEntry>,
}

impl Services {
    /// Parse the `[services]` table from `.govern.toml` contents. An absent
    /// table or an empty document yields an empty registry — never an error.
    ///
    /// # Errors
    ///
    /// Returns the underlying [`toml::de::Error`] when the document is not
    /// valid TOML or a `[services]` entry is missing a required field.
    pub fn from_toml_str(content: &str) -> std::result::Result<Self, toml::de::Error> {
        let parsed: ServicesConfig = toml::from_str(content)?;
        Ok(Self(parsed.services))
    }

    /// True when no services are registered.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Aliases that share a `repo`, grouped by repo. A duplicate repo makes
    /// link→service matching ambiguous, so callers surface it as a finding.
    /// Returns one `(repo, aliases)` group per repo used by two or more
    /// aliases; output is sorted for determinism.
    #[must_use]
    pub fn duplicate_repos(&self) -> Vec<(String, Vec<String>)> {
        let mut by_repo: BTreeMap<&str, Vec<&str>> = BTreeMap::new();
        for (alias, entry) in &self.0 {
            by_repo
                .entry(entry.repo.as_str())
                .or_default()
                .push(alias.as_str());
        }
        by_repo
            .into_iter()
            .filter(|(_, aliases)| aliases.len() > 1)
            .map(|(repo, aliases)| {
                (
                    repo.to_string(),
                    aliases.into_iter().map(String::from).collect(),
                )
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn parses_present_entries() {
        let toml = r#"
[services.api]
repo = "https://github.com/acme/api"
path = "../api"
description = "owns all data models; the system of record"

[services.frontend]
repo = "https://github.com/acme/frontend"
path = "../frontend"
"#;
        let services = Services::from_toml_str(toml).unwrap();
        assert_eq!(services.0.len(), 2);

        let api = services.0.get("api").expect("api entry");
        assert_eq!(api.repo, "https://github.com/acme/api");
        assert_eq!(api.path, "../api");
        assert_eq!(
            api.description.as_deref(),
            Some("owns all data models; the system of record")
        );

        let frontend = services.0.get("frontend").expect("frontend entry");
        assert_eq!(frontend.path, "../frontend");
        assert!(frontend.description.is_none());
    }

    #[test]
    fn absent_table_is_empty() {
        // A `.govern.toml` with other tables but no `[services]`.
        let toml = "[review]\ntech-stack-verified = true\n";
        let services = Services::from_toml_str(toml).unwrap();
        assert!(services.is_empty());
        assert!(services.duplicate_repos().is_empty());
    }

    #[test]
    fn empty_document_is_empty() {
        let services = Services::from_toml_str("").unwrap();
        assert!(services.is_empty());
    }

    #[test]
    fn duplicate_repos_detected() {
        let toml = r#"
[services.api]
repo = "https://github.com/acme/api"
path = "../api"

[services.backend]
repo = "https://github.com/acme/api"
path = "../backend"

[services.frontend]
repo = "https://github.com/acme/frontend"
path = "../frontend"
"#;
        let services = Services::from_toml_str(toml).unwrap();
        let dups = services.duplicate_repos();
        assert_eq!(dups.len(), 1, "exactly one repo is shared");
        let (repo, aliases) = &dups[0];
        assert_eq!(repo, "https://github.com/acme/api");
        assert_eq!(aliases, &vec!["api".to_string(), "backend".to_string()]);
    }

    #[test]
    fn distinct_repos_have_no_duplicates() {
        let toml = r#"
[services.api]
repo = "https://github.com/acme/api"
path = "../api"

[services.frontend]
repo = "https://github.com/acme/frontend"
path = "../frontend"
"#;
        let services = Services::from_toml_str(toml).unwrap();
        assert!(services.duplicate_repos().is_empty());
    }

    #[test]
    fn missing_required_field_is_error() {
        // `path` omitted — a malformed entry surfaces as a parse error.
        let toml = "[services.api]\nrepo = \"https://github.com/acme/api\"\n";
        assert!(Services::from_toml_str(toml).is_err());
    }
}
