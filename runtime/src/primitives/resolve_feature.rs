//! `resolve-feature` — resolve a user-supplied identifier to a feature
//! directory under the configured spec root.
//!
//! The deterministic core of `/gov:target`'s specs-dir scan (spec 022,
//! scenario scaffolding-primitives): every session's first command starts
//! by turning an identifier — exact directory name, feature number, or
//! partial slug — into a feature directory. Structured like
//! [`crate::primitives::read_spec`]: spec-root resolution via
//! [`crate::schema::paths::Paths::load`], repo-relative result paths.
//!
//! Matching order:
//!
//! 1. **Exact directory name** — `022-deterministic-runtime`.
//! 2. **Feature number** — an all-digit identifier (`7` or `007`) matches
//!    every feature whose three-digit `NNN-` prefix parses to the same
//!    number, so both forms resolve identically.
//! 3. **Partial slug substring** — case-insensitive substring over the
//!    directory names. Unique match resolves; multiple matches yield the
//!    `ambiguous` outcome with the sorted candidate list; zero yield
//!    `not-found`.
//!
//! Ambiguity and no-match are **domain outcomes** in the result, never
//! operational errors — choosing stays with the user through the host.

use std::path::Path;

use crate::primitives::{
    PrimitiveError, Result, is_feature_slug, read_text, split_frontmatter, validate_slug,
};
use crate::schema::paths;
use crate::schema::primitives::{
    ResolveFeatureArgs, ResolveFeatureOutcome, ResolveFeatureResult, ResolvedScenario,
};

/// Frontmatter shape used only to read `status` (best-effort).
#[derive(serde::Deserialize)]
struct StatusOnly {
    status: Option<String>,
}

/// Execute the `resolve-feature` primitive against the given repo root.
///
/// # Errors
///
/// Returns [`PrimitiveError::InvalidArgument`] when `identifier` is empty
/// or whitespace-only, or [`PrimitiveError::InvalidSlug`] when the
/// optional `scenario` slug carries path separators or a dot prefix.
/// Ambiguity and no-match are domain outcomes, not errors; a missing or
/// unreadable spec root simply yields `not-found`.
pub fn run(args: &ResolveFeatureArgs, repo: &Path) -> Result<ResolveFeatureResult> {
    let identifier = args.identifier.trim();
    if identifier.is_empty() {
        return Err(PrimitiveError::InvalidArgument {
            primitive: "resolve-feature".into(),
            argument: "identifier".into(),
            reason: "identifier is empty".into(),
        });
    }
    if let Some(slug) = &args.scenario {
        validate_slug(slug)?;
    }

    let root = paths::Paths::load(repo).specs_root;
    let features = list_features(&repo.join(&root));

    match match_identifier(&features, identifier) {
        Match::One(feature) => Ok(resolved(repo, &root, &feature, args.scenario.as_deref())),
        Match::Many(candidates) => Ok(ResolveFeatureResult {
            outcome: ResolveFeatureOutcome::Ambiguous,
            feature: None,
            path: None,
            status: None,
            candidates,
            scenario: None,
        }),
        Match::None => Ok(ResolveFeatureResult {
            outcome: ResolveFeatureOutcome::NotFound,
            feature: None,
            path: None,
            status: None,
            candidates: Vec::new(),
            scenario: None,
        }),
    }
}

/// Match classification for one identifier against the feature list.
enum Match {
    /// Exactly one feature matched.
    One(String),
    /// Multiple features matched a partial/numeric identifier (sorted).
    Many(Vec<String>),
    /// Nothing matched.
    None,
}

/// Apply the three-tier matching order documented at module level.
fn match_identifier(features: &[String], identifier: &str) -> Match {
    // 1. Exact directory name.
    if let Some(exact) = features.iter().find(|f| f.as_str() == identifier) {
        return Match::One(exact.clone());
    }
    // 2. Feature number: all-digit identifier compared against the parsed
    //    three-digit prefix, so `7` and `007` both match `007-foo`.
    if identifier.bytes().all(|b| b.is_ascii_digit()) {
        let Ok(number) = identifier.parse::<u32>() else {
            return Match::None; // longer than u32 — nothing can match
        };
        let matches: Vec<String> = features
            .iter()
            .filter(|f| feature_number(f) == Some(number))
            .cloned()
            .collect();
        return classify(matches);
    }
    // 3. Case-insensitive partial slug substring.
    let needle = identifier.to_lowercase();
    let matches: Vec<String> = features
        .iter()
        .filter(|f| f.to_lowercase().contains(&needle))
        .cloned()
        .collect();
    classify(matches)
}

/// Fold a match list into the `Match` classification. The list arrives
/// sorted (feature enumeration sorts), so `Many` carries the sorted
/// candidate list directly.
fn classify(mut matches: Vec<String>) -> Match {
    match matches.len() {
        0 => Match::None,
        1 => Match::One(matches.remove(0)),
        _ => Match::Many(matches),
    }
}

/// Parse a feature directory's three-digit `NNN-` prefix. `None` for
/// names that don't match the convention (callers pre-filter with
/// [`is_feature_slug`], so this is belt-and-suspenders).
fn feature_number(name: &str) -> Option<u32> {
    name.get(..3)?.parse::<u32>().ok()
}

/// List feature directories (`NNN-slug`) under the spec root, sorted by
/// name. A missing or unreadable spec root yields an empty list — the
/// caller reports `not-found` rather than an operational error, since a
/// repo without a spec root has no features by definition.
fn list_features(specs_dir: &Path) -> Vec<String> {
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

/// Build the `resolved` result: repo-relative path, best-effort status,
/// and the optional scenario detail.
fn resolved(
    repo: &Path,
    root: &str,
    feature: &str,
    scenario_slug: Option<&str>,
) -> ResolveFeatureResult {
    let feature_dir = repo.join(root).join(feature);
    let status = read_status(&feature_dir);
    let scenario = scenario_slug.map(|slug| scenario_detail(&feature_dir, root, feature, slug));
    ResolveFeatureResult {
        outcome: ResolveFeatureOutcome::Resolved,
        feature: Some(feature.to_string()),
        path: Some(format!("{root}/{feature}")),
        status,
        candidates: Vec::new(),
        scenario,
    }
}

/// Best-effort read of the spec's frontmatter `status`. A missing spec
/// file, missing frontmatter, or malformed YAML degrades to `None` —
/// resolution is the first step of a session, and a broken spec must
/// still be targetable so the user can go fix it.
fn read_status(feature_dir: &Path) -> Option<String> {
    let spec_path = feature_dir.join("spec.md");
    let content = read_text(&spec_path).ok()?;
    let (fm_text, _body) = split_frontmatter(&content, &spec_path).ok()?;
    serde_norway::from_str::<StatusOnly>(fm_text).ok()?.status
}

/// Scenario frontmatter shape — `section` is the post-017 field;
/// `spec-ref` is the pre-017 legacy field. Mirrors
/// `crate::primitives::dashboard`'s reader so the two surfaces agree.
#[derive(serde::Deserialize)]
struct ScenarioFrontmatter {
    #[serde(default)]
    section: Option<String>,
    #[serde(default, rename = "spec-ref")]
    spec_ref: Option<String>,
}

/// Build the scenario detail for a resolved feature: existence plus the
/// `section` frontmatter (best-effort empty on absent/unreadable files,
/// matching `dashboard`'s scenario-detail degradation).
fn scenario_detail(feature_dir: &Path, root: &str, feature: &str, slug: &str) -> ResolvedScenario {
    let scenario_path = feature_dir.join("scenarios").join(format!("{slug}.md"));
    let exists = scenario_path.is_file();
    let section = if exists {
        read_section(&scenario_path).unwrap_or_default()
    } else {
        String::new()
    };
    ResolvedScenario {
        slug: slug.to_string(),
        path: format!("{root}/{feature}/scenarios/{slug}.md"),
        exists,
        section,
    }
}

/// Best-effort read of a scenario's `section` (or legacy `spec-ref`)
/// frontmatter field.
fn read_section(path: &Path) -> Option<String> {
    let content = read_text(path).ok()?;
    let (fm_text, _body) = split_frontmatter(&content, path).ok()?;
    let fm = serde_norway::from_str::<ScenarioFrontmatter>(fm_text).ok()?;
    fm.section.or(fm.spec_ref)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn write(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    fn seed(repo: &Path) {
        write(
            &repo.join("specs/007-webhooks/spec.md"),
            "---\nstatus: done\ndependencies: []\n---\n\n# Webhooks\n",
        );
        write(
            &repo.join("specs/022-deterministic-runtime/spec.md"),
            "---\nstatus: in-progress\ndependencies: []\n---\n\n# Runtime\n",
        );
        write(
            &repo.join("specs/022-deterministic-runtime/scenarios/scaffolding-primitives.md"),
            "---\nsection: \"Follow-on scenarios\"\n---\n\n# Scaffolding-primitives\n",
        );
        write(
            &repo.join("specs/023-command-runtime/spec.md"),
            "---\nstatus: done\ndependencies: []\n---\n\n# Commands\n",
        );
        // Non-feature siblings that must never match.
        fs::create_dir_all(repo.join("specs/templates")).unwrap();
        write(&repo.join("specs/inbox.md"), "# Inbox\n");
    }

    fn args(identifier: &str, scenario: Option<&str>) -> ResolveFeatureArgs {
        ResolveFeatureArgs {
            identifier: identifier.into(),
            scenario: scenario.map(Into::into),
        }
    }

    #[test]
    fn resolves_exact_directory_name() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        let result = run(&args("022-deterministic-runtime", None), tmp.path()).unwrap();
        assert_eq!(result.outcome, ResolveFeatureOutcome::Resolved);
        assert_eq!(result.feature.as_deref(), Some("022-deterministic-runtime"));
        assert_eq!(
            result.path.as_deref(),
            Some("specs/022-deterministic-runtime")
        );
        assert_eq!(result.status.as_deref(), Some("in-progress"));
        assert!(result.candidates.is_empty());
        assert!(result.scenario.is_none());
    }

    #[test]
    fn resolves_bare_and_zero_padded_numbers_identically() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        for ident in ["7", "007"] {
            let result = run(&args(ident, None), tmp.path()).unwrap();
            assert_eq!(
                result.outcome,
                ResolveFeatureOutcome::Resolved,
                "identifier {ident:?}"
            );
            assert_eq!(result.feature.as_deref(), Some("007-webhooks"));
            assert_eq!(result.status.as_deref(), Some("done"));
        }
    }

    #[test]
    fn resolves_unique_partial_slug_case_insensitively() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        let result = run(&args("DETERMINISTIC", None), tmp.path()).unwrap();
        assert_eq!(result.outcome, ResolveFeatureOutcome::Resolved);
        assert_eq!(result.feature.as_deref(), Some("022-deterministic-runtime"));
    }

    #[test]
    fn ambiguous_partial_returns_sorted_candidates_as_domain_outcome() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        // "runtime" matches 022- and 023-.
        let result = run(&args("runtime", None), tmp.path()).unwrap();
        assert_eq!(result.outcome, ResolveFeatureOutcome::Ambiguous);
        assert_eq!(
            result.candidates,
            vec![
                "022-deterministic-runtime".to_string(),
                "023-command-runtime".to_string()
            ]
        );
        assert!(result.feature.is_none());
        assert!(result.status.is_none());
    }

    #[test]
    fn no_match_returns_not_found_domain_outcome() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        let result = run(&args("no-such-feature", None), tmp.path()).unwrap();
        assert_eq!(result.outcome, ResolveFeatureOutcome::NotFound);
        assert!(result.candidates.is_empty());

        let by_number = run(&args("404", None), tmp.path()).unwrap();
        assert_eq!(by_number.outcome, ResolveFeatureOutcome::NotFound);
    }

    #[test]
    fn missing_specs_root_is_not_found_not_an_error() {
        let tmp = tempdir().unwrap();
        let result = run(&args("anything", None), tmp.path()).unwrap();
        assert_eq!(result.outcome, ResolveFeatureOutcome::NotFound);
    }

    #[test]
    fn scenario_detail_reports_existence_and_section() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        let result = run(&args("022", Some("scaffolding-primitives")), tmp.path()).unwrap();
        let scenario = result.scenario.expect("scenario detail present");
        assert!(scenario.exists);
        assert_eq!(scenario.section, "Follow-on scenarios");
        assert_eq!(
            scenario.path,
            "specs/022-deterministic-runtime/scenarios/scaffolding-primitives.md"
        );
        assert_eq!(scenario.slug, "scaffolding-primitives");
    }

    #[test]
    fn scenario_detail_reports_missing_file_with_empty_section() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        let result = run(&args("022", Some("not-yet-written")), tmp.path()).unwrap();
        let scenario = result.scenario.expect("scenario detail present");
        assert!(!scenario.exists);
        assert_eq!(scenario.section, "");
        assert_eq!(
            scenario.path,
            "specs/022-deterministic-runtime/scenarios/not-yet-written.md"
        );
    }

    #[test]
    fn scenario_detail_falls_back_to_legacy_spec_ref() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        write(
            &tmp.path()
                .join("specs/007-webhooks/scenarios/legacy-shape.md"),
            "---\nspec-ref: \"Delivery guarantees\"\n---\n\n# Legacy-shape\n",
        );
        let result = run(&args("007", Some("legacy-shape")), tmp.path()).unwrap();
        let scenario = result.scenario.expect("scenario detail present");
        assert!(scenario.exists);
        assert_eq!(scenario.section, "Delivery guarantees");
    }

    #[test]
    fn unreadable_spec_degrades_status_to_none() {
        let tmp = tempdir().unwrap();
        // Feature dir with a spec.md that has no frontmatter fences.
        write(
            &tmp.path().join("specs/050-broken/spec.md"),
            "# Broken — no frontmatter\n",
        );
        let result = run(&args("050", None), tmp.path()).unwrap();
        assert_eq!(result.outcome, ResolveFeatureOutcome::Resolved);
        assert_eq!(result.feature.as_deref(), Some("050-broken"));
        assert!(result.status.is_none(), "broken spec still resolves");
    }

    #[test]
    fn rejects_empty_identifier() {
        let tmp = tempdir().unwrap();
        for ident in ["", "   "] {
            let err = run(&args(ident, None), tmp.path()).unwrap_err();
            assert!(
                matches!(err, PrimitiveError::InvalidArgument { .. }),
                "expected InvalidArgument for {ident:?}"
            );
        }
    }

    #[test]
    fn rejects_scenario_slug_with_path_separator() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        let err = run(&args("022", Some("../escape")), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidSlug { .. }));
    }

    #[test]
    fn resolves_against_fixture_repo() {
        // The shared fixture repo exercises the default `specs` root.
        let repo =
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/primitives/sample-repo");
        let result = run(&args("1", None), &repo).unwrap();
        assert_eq!(result.outcome, ResolveFeatureOutcome::Resolved);
        assert_eq!(result.feature.as_deref(), Some("001-basic"));
        assert_eq!(result.status.as_deref(), Some("clarified"));
    }
}
