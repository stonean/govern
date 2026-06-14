//! `resolve-references` — classify each cross-service reference (spec 030).
//!
//! Reads the consumer feature's derived `references:` index, resolves each
//! entry's service through the `.govern.toml` `[services]` registry, and
//! reads the linked spec's live `status` from the registered local checkout.
//! Every reference is classified into the closed [`ReferenceOutcome`] enum by
//! deterministic predicates — no prose is read for intent. Reuses
//! `read_text` / `split_frontmatter` and the allowed-status set that
//! `validate-frontmatter` enforces.
//!
//! The index stores `{service, spec}` pairs (the raw URL is not persisted),
//! so URL-level distinctions collapse here: a malformed URL that yielded no
//! valid spec was never harvested, and a resolved-but-absent target surfaces
//! as `broken`; a scenario link was normalized to its feature slug at harvest
//! time. Schema is canonical in
//! `specs/030-cross-service-references/data-model.md`.

use std::path::{Path, PathBuf};

use serde::Deserialize;
use serde_norway::Value as YamlValue;

use crate::primitives::{PrimitiveError, Result, read_text, rel_path, split_frontmatter};
use crate::schema::primitives::{
    ReferenceOutcome, ResolutionRecord, ResolveReferencesArgs, ResolveReferencesResult,
};
use crate::schema::services::Services;

/// Lifecycle statuses a linked spec may carry. Mirrors the set
/// `validate-frontmatter` enforces and constitution §text-first-artifacts.
const ALLOWED_STATUSES: &[&str] = &["draft", "clarified", "planned", "in-progress", "done"];

/// The subset of the consumer spec's frontmatter this primitive reads: the
/// derived `references:` index. Other fields are ignored.
#[derive(Debug, Default, Deserialize)]
struct ConsumerFrontmatter {
    #[serde(default)]
    references: Vec<IndexEntry>,
}

/// One `references:` index entry. `service` is null for an `unregistered`
/// reference (the harvester matched no registry repo).
#[derive(Debug, Deserialize)]
struct IndexEntry {
    #[serde(default)]
    service: Option<String>,
    spec: String,
}

/// Execute the `resolve-references` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::Io`] when the consumer spec cannot be read,
/// [`PrimitiveError::MissingFrontmatter`] when it has no `---` block,
/// [`PrimitiveError::Yaml`] when its frontmatter is not valid YAML, or
/// [`PrimitiveError::Toml`] when `.govern.toml` is present but malformed.
/// Per-reference resolution failures are outcomes, not errors.
pub fn run(args: &ResolveReferencesArgs, repo: &Path) -> Result<ResolveReferencesResult> {
    let spec_path = repo.join("specs").join(&args.feature).join("spec.md");
    let content = read_text(&spec_path)?;
    let (fm_text, _body) = split_frontmatter(&content, &spec_path)?;
    let frontmatter: ConsumerFrontmatter =
        serde_norway::from_str(fm_text).map_err(|source| PrimitiveError::Yaml {
            path: spec_path.clone(),
            source,
        })?;

    let services = load_services(repo)?;

    let references = frontmatter
        .references
        .iter()
        .map(|entry| {
            let (outcome, status) = classify(repo, &services, entry);
            ResolutionRecord {
                service: entry.service.clone(),
                spec: entry.spec.clone(),
                outcome,
                status,
            }
        })
        .collect();

    Ok(ResolveReferencesResult {
        references,
        path: rel_path(&spec_path, repo),
    })
}

/// Read `.govern.toml` `[services]` from the repo root. An absent file is an
/// empty registry; a malformed one is an operational error.
fn load_services(repo: &Path) -> Result<Services> {
    let toml_path = repo.join(".govern.toml");
    if !toml_path.exists() {
        return Ok(Services::default());
    }
    let content = read_text(&toml_path)?;
    Services::from_toml_str(&content).map_err(|source| PrimitiveError::Toml {
        path: toml_path,
        source,
    })
}

/// Classify one reference against the registry and the local checkout.
fn classify(
    repo: &Path,
    services: &Services,
    entry: &IndexEntry,
) -> (ReferenceOutcome, Option<String>) {
    // Null service (or an alias no longer in the registry) → unregistered:
    // a plain navigational link, status not attempted.
    let Some(alias) = entry.service.as_deref() else {
        return (ReferenceOutcome::Unregistered, None);
    };
    let Some(service) = services.0.get(alias) else {
        return (ReferenceOutcome::Unregistered, None);
    };

    // Resolve the local checkout. A missing/unusable path can prove nothing,
    // so it is `not-checked-out`, never `broken`.
    let checkout = resolve_checkout(repo, &service.path);
    if !checkout.is_dir() {
        return (ReferenceOutcome::NotCheckedOut, None);
    }

    // Reachable checkout: the target spec either resolves or is provably broken.
    let target = checkout.join("specs").join(&entry.spec).join("spec.md");
    if !target.is_file() {
        return (ReferenceOutcome::Broken, None);
    }

    match read_target_status(&target) {
        Some(status) => (ReferenceOutcome::Ok, Some(status)),
        None => (ReferenceOutcome::StatusUnreadable, None),
    }
}

/// Resolve a service's `path` (relative to the repo root or absolute) to a
/// concrete checkout directory. `..` is permitted — a sibling checkout is the
/// normal case (`path = "../api"`), and this is machine-local config, not an
/// LLM-supplied path.
fn resolve_checkout(repo: &Path, path: &str) -> PathBuf {
    let p = Path::new(path);
    if p.is_absolute() {
        p.to_path_buf()
    } else {
        repo.join(p)
    }
}

/// Read a reachable target spec's `status`, returning `Some(status)` only when
/// the frontmatter parses and the value is in the allowed set. Any
/// unreadability (no frontmatter, malformed YAML, missing/non-string/out-of-set
/// `status`) collapses to `None` → `status-unreadable`.
fn read_target_status(target: &Path) -> Option<String> {
    let content = std::fs::read_to_string(target).ok()?;
    let (fm_text, _body) = split_frontmatter(&content, target).ok()?;
    let parsed: YamlValue = serde_norway::from_str(fm_text).ok()?;
    let YamlValue::Mapping(map) = parsed else {
        return None;
    };
    let status = map.get("status")?.as_str()?;
    if ALLOWED_STATUSES.contains(&status) {
        Some(status.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::path::Path;

    /// Write `content` to `path`, creating parent directories.
    fn write_file(path: &Path, content: &str) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(path, content).unwrap();
    }

    /// Seed a consumer repo: `.govern.toml` from `toml`, and a consumer
    /// `specs/001-consumer/spec.md` whose frontmatter carries `references`.
    fn seed_consumer(repo: &Path, toml: &str, references_block: &str) {
        write_file(&repo.join(".govern.toml"), toml);
        let spec = format!(
            "---\nstatus: in-progress\ndependencies: []\n{references_block}---\n\n# Consumer\n"
        );
        write_file(&repo.join("specs/001-consumer/spec.md"), &spec);
    }

    /// Write a fake registered checkout's spec at
    /// `<repo>/<checkout>/specs/<slug>/spec.md` with the given raw content.
    fn write_checkout_spec(repo: &Path, checkout: &str, slug: &str, content: &str) {
        write_file(
            &repo.join(checkout).join("specs").join(slug).join("spec.md"),
            content,
        );
    }

    fn run_consumer(repo: &Path) -> ResolveReferencesResult {
        run(
            &ResolveReferencesArgs {
                feature: "001-consumer".into(),
            },
            repo,
        )
        .unwrap()
    }

    const API_TOML: &str =
        "[services.api]\nrepo = \"https://github.com/acme/api\"\npath = \"checkouts/api\"\n";

    #[test]
    fn ok_when_registered_reachable_and_status_in_set() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        seed_consumer(
            repo,
            API_TOML,
            "references:\n  - service: api\n    spec: 003-user\n",
        );
        write_checkout_spec(
            repo,
            "checkouts/api",
            "003-user",
            "---\nstatus: clarified\n---\n# U\n",
        );

        let result = run_consumer(repo);
        assert_eq!(result.references.len(), 1);
        let rec = &result.references[0];
        assert_eq!(rec.service.as_deref(), Some("api"));
        assert_eq!(rec.spec, "003-user");
        assert_eq!(rec.outcome, ReferenceOutcome::Ok);
        assert_eq!(rec.status.as_deref(), Some("clarified"));
        assert_eq!(result.path, "specs/001-consumer/spec.md");
    }

    #[test]
    fn unregistered_when_service_is_null() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        // Null service in the index → unregistered, status not attempted.
        seed_consumer(
            repo,
            API_TOML,
            "references:\n  - service: null\n    spec: 004-orders\n",
        );

        let rec = &run_consumer(repo).references[0];
        assert_eq!(rec.service, None);
        assert_eq!(rec.outcome, ReferenceOutcome::Unregistered);
        assert_eq!(rec.status, None);
    }

    #[test]
    fn unregistered_when_alias_absent_from_registry() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        // Index names `ghost`, but the registry only knows `api`.
        seed_consumer(
            repo,
            API_TOML,
            "references:\n  - service: ghost\n    spec: 003-user\n",
        );

        let rec = &run_consumer(repo).references[0];
        assert_eq!(rec.outcome, ReferenceOutcome::Unregistered);
    }

    #[test]
    fn not_checked_out_when_path_missing() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        // Registered, but no checkout directory exists at `checkouts/api`.
        seed_consumer(
            repo,
            API_TOML,
            "references:\n  - service: api\n    spec: 003-user\n",
        );

        let rec = &run_consumer(repo).references[0];
        assert_eq!(rec.outcome, ReferenceOutcome::NotCheckedOut);
        assert_eq!(rec.status, None);
    }

    #[test]
    fn broken_when_checkout_reachable_but_target_absent() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        seed_consumer(
            repo,
            API_TOML,
            "references:\n  - service: api\n    spec: 003-user\n",
        );
        // Checkout exists (some other spec present) but 003-user does not —
        // renamed / deleted / mistyped upstream, or a malformed URL's slug.
        write_checkout_spec(
            repo,
            "checkouts/api",
            "009-other",
            "---\nstatus: done\n---\n# O\n",
        );

        let rec = &run_consumer(repo).references[0];
        assert_eq!(rec.outcome, ReferenceOutcome::Broken);
        assert_eq!(rec.status, None);
    }

    #[test]
    fn status_unreadable_when_no_frontmatter() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        seed_consumer(
            repo,
            API_TOML,
            "references:\n  - service: api\n    spec: 003-user\n",
        );
        write_checkout_spec(
            repo,
            "checkouts/api",
            "003-user",
            "# User\n\nNo frontmatter here.\n",
        );

        let rec = &run_consumer(repo).references[0];
        assert_eq!(rec.outcome, ReferenceOutcome::StatusUnreadable);
    }

    #[test]
    fn status_unreadable_when_yaml_malformed() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        seed_consumer(
            repo,
            API_TOML,
            "references:\n  - service: api\n    spec: 003-user\n",
        );
        // Unbalanced bracket → YAML parse failure inside the frontmatter.
        write_checkout_spec(
            repo,
            "checkouts/api",
            "003-user",
            "---\nstatus: [unterminated\n---\n# U\n",
        );

        let rec = &run_consumer(repo).references[0];
        assert_eq!(rec.outcome, ReferenceOutcome::StatusUnreadable);
    }

    #[test]
    fn status_unreadable_when_status_out_of_set() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        seed_consumer(
            repo,
            API_TOML,
            "references:\n  - service: api\n    spec: 003-user\n",
        );
        write_checkout_spec(
            repo,
            "checkouts/api",
            "003-user",
            "---\nstatus: wibble\n---\n# U\n",
        );

        let rec = &run_consumer(repo).references[0];
        assert_eq!(rec.outcome, ReferenceOutcome::StatusUnreadable);
    }

    #[test]
    fn self_reference_resolves_like_any_registered_service() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        // The consumer's own repo is registered with `path = "."`, and the
        // reference points back at one of its own specs. No special-casing:
        // it resolves `ok` like any registered service.
        let toml = "[services.self]\nrepo = \"https://github.com/acme/consumer\"\npath = \".\"\n";
        seed_consumer(
            repo,
            toml,
            "references:\n  - service: self\n    spec: 002-local\n",
        );
        write_file(
            &repo.join("specs/002-local/spec.md"),
            "---\nstatus: planned\ndependencies: []\n---\n# Local\n",
        );

        let rec = &run_consumer(repo).references[0];
        assert_eq!(rec.service.as_deref(), Some("self"));
        assert_eq!(rec.outcome, ReferenceOutcome::Ok);
        assert_eq!(rec.status.as_deref(), Some("planned"));
    }

    #[test]
    fn empty_index_yields_no_records() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        // No `references:` field at all.
        seed_consumer(repo, API_TOML, "");

        let result = run_consumer(repo);
        assert!(result.references.is_empty());
    }

    #[test]
    fn absent_govern_toml_makes_named_service_unregistered() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path();
        // No `.govern.toml` → empty registry → a named alias resolves to
        // unregistered rather than erroring.
        let spec = "---\nstatus: draft\ndependencies: []\nreferences:\n  - service: api\n    spec: 003-user\n---\n\n# Consumer\n";
        write_file(&repo.join("specs/001-consumer/spec.md"), spec);

        let rec = &run_consumer(repo).references[0];
        assert_eq!(rec.outcome, ReferenceOutcome::Unregistered);
    }
}
