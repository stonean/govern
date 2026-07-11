//! `create-feature` — scaffold the next `{specs-root}/{NNN-slug}/`
//! feature directory with a spec-template copy.
//!
//! The deterministic scaffold step of `/gov:specify` (spec 022, scenario
//! scaffolding-primitives): compute the next feature number, derive the
//! kebab-case slug from the title, create the directory, and copy the
//! spec template into it — atomic and mode-preserving. The LLM fills the
//! spec body afterwards via `writeSpecBody`.
//!
//! Template resolution mirrors `interpreter::payload::load_template`
//! (the `writeSpecBody` request builder): try the installed adopter
//! layout `{specs-root}/templates/spec.md` first, then the framework
//! source layout `framework/templates/spec/spec.md`. The copy goes
//! through [`crate::primitives::write_atomic_bytes`] plus
//! [`crate::primitives::apply_manifest::mirror_source_mode`] — the
//! atomic-write helper lands files at tempfile mode `0600`, so the
//! template's mode is re-applied explicitly (AGENTS.md gotcha,
//! 2026-06-30).

use std::path::Path;

use crate::primitives::apply_manifest::mirror_source_mode;
use crate::primitives::{
    PrimitiveError, Result, feature_number, list_feature_dirs, template_candidates,
    write_atomic_bytes,
};
use crate::schema::paths;
use crate::schema::primitives::{CreateFeatureArgs, CreateFeatureResult};

/// Execute the `create-feature` primitive against the given repo root.
///
/// # Errors
///
/// Returns [`PrimitiveError::InvalidArgument`] when `title` derives to an
/// empty slug, [`PrimitiveError::TemplateNotFound`] when no spec template
/// exists at either candidate location, or [`PrimitiveError::Io`] for
/// filesystem failures. An already-existing target directory is the
/// `created: false` **domain outcome** (no overwrite path), not an error.
pub fn run(args: &CreateFeatureArgs, repo: &Path) -> Result<CreateFeatureResult> {
    let slug = derive_slug(&args.title);
    if slug.is_empty() {
        return Err(PrimitiveError::InvalidArgument {
            primitive: "create-feature".into(),
            argument: "title".into(),
            reason: "title derives to an empty slug (no ASCII alphanumeric characters)".into(),
        });
    }

    let root = paths::Paths::load(repo).specs_root;
    let specs_dir = repo.join(&root);
    let number = next_feature_number(&specs_dir);
    let feature = format!("{number:03}-{slug}");
    let feature_dir = specs_dir.join(&feature);
    let rel_dir = format!("{root}/{feature}");

    // Refusal domain outcome: the derived directory already exists.
    // `next_feature_number` makes this unreachable for well-formed spec
    // roots (max + 1 exceeds every existing prefix), but a racing writer
    // or a hand-created directory must never be overwritten.
    if feature_dir.exists() {
        return Ok(CreateFeatureResult {
            created: false,
            feature,
            path: rel_dir,
            template: None,
        });
    }

    // Resolve the template before creating anything, so a missing
    // template leaves no half-scaffolded directory behind.
    let (template_rel, template_abs) = resolve_template(repo, &root)?;
    let template_bytes = std::fs::read(&template_abs).map_err(|source| PrimitiveError::Io {
        path: template_abs.clone(),
        source,
    })?;

    std::fs::create_dir_all(&feature_dir).map_err(|source| PrimitiveError::Io {
        path: feature_dir.clone(),
        source,
    })?;
    let dest = feature_dir.join("spec.md");
    write_atomic_bytes(&dest, &template_bytes)?;
    mirror_source_mode(&template_abs, &dest)?;

    Ok(CreateFeatureResult {
        created: true,
        feature,
        path: rel_dir,
        template: Some(template_rel),
    })
}

/// Derive the kebab-case directory slug from a feature title: every ASCII
/// alphanumeric character is lowercased and kept; every run of other
/// characters (spaces, punctuation, non-ASCII) collapses to a single
/// hyphen; leading and trailing hyphens are trimmed.
fn derive_slug(title: &str) -> String {
    let mut out = String::with_capacity(title.len());
    let mut pending_hyphen = false;
    for ch in title.chars() {
        if ch.is_ascii_alphanumeric() {
            if pending_hyphen && !out.is_empty() {
                out.push('-');
            }
            pending_hyphen = false;
            out.push(ch.to_ascii_lowercase());
        } else {
            pending_hyphen = true;
        }
    }
    out
}

/// Compute the next feature number: the max existing three-digit `NNN-`
/// prefix across feature directories, plus one. `1` when the spec root is
/// missing or holds no feature directories. Numbers past 999 render
/// four-digit (the `{:03}` pad only guarantees a minimum width).
fn next_feature_number(specs_dir: &Path) -> u32 {
    list_feature_dirs(specs_dir)
        .iter()
        .filter_map(|name| feature_number(name))
        .max()
        .unwrap_or(0)
        + 1
}

/// Resolve the spec template, mirroring `payload::load_template`'s
/// candidate order: `{specs-root}/templates/spec.md` (installed adopter
/// layout), then `framework/templates/spec/spec.md` (framework source
/// layout). Returns `(repo-relative path, absolute path)` of the first
/// candidate on disk.
fn resolve_template(repo: &Path, root: &str) -> Result<(String, std::path::PathBuf)> {
    let candidates = template_candidates(root, "spec.md");
    for rel in &candidates {
        let abs = repo.join(rel);
        if abs.is_file() {
            return Ok((rel.clone(), abs));
        }
    }
    Err(PrimitiveError::TemplateNotFound {
        tried: candidates.join(", "),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;
    use tempfile::tempdir;

    const TEMPLATE: &str = "---\nstatus: draft\ndependencies: []\n---\n\n# {Feature}\n";

    fn seed_with_installed_template(repo: &Path) {
        fs::create_dir_all(repo.join("specs/templates")).unwrap();
        fs::write(repo.join("specs/templates/spec.md"), TEMPLATE).unwrap();
    }

    fn args(title: &str) -> CreateFeatureArgs {
        CreateFeatureArgs {
            title: title.into(),
        }
    }

    #[test]
    fn creates_first_feature_as_001() {
        let tmp = tempdir().unwrap();
        seed_with_installed_template(tmp.path());
        let result = run(&args("Webhook Delivery"), tmp.path()).unwrap();
        assert!(result.created);
        assert_eq!(result.feature, "001-webhook-delivery");
        assert_eq!(result.path, "specs/001-webhook-delivery");
        assert_eq!(result.template.as_deref(), Some("specs/templates/spec.md"));
        let body =
            fs::read_to_string(tmp.path().join("specs/001-webhook-delivery/spec.md")).unwrap();
        assert_eq!(body, TEMPLATE, "spec.md is a byte copy of the template");
    }

    #[test]
    fn numbers_from_max_existing_prefix_plus_one() {
        let tmp = tempdir().unwrap();
        seed_with_installed_template(tmp.path());
        // Gap-tolerant: 003 and 007 exist → next is 008, not 004.
        for existing in ["003-a", "007-b"] {
            fs::create_dir_all(tmp.path().join("specs").join(existing)).unwrap();
        }
        let result = run(&args("next one"), tmp.path()).unwrap();
        assert_eq!(result.feature, "008-next-one");
    }

    #[test]
    fn non_feature_siblings_do_not_affect_numbering() {
        let tmp = tempdir().unwrap();
        seed_with_installed_template(tmp.path());
        fs::create_dir_all(tmp.path().join("specs/005-real")).unwrap();
        // `templates/` (already created), a stray file, and a dotdir must
        // not contribute prefixes.
        fs::write(tmp.path().join("specs/inbox.md"), "# Inbox\n").unwrap();
        fs::create_dir_all(tmp.path().join("specs/.cache")).unwrap();
        let result = run(&args("counted right"), tmp.path()).unwrap();
        assert_eq!(result.feature, "006-counted-right");
    }

    #[test]
    fn slug_derivation_collapses_and_trims() {
        assert_eq!(derive_slug("Webhook Delivery"), "webhook-delivery");
        assert_eq!(derive_slug("  Retry!!  Logic  "), "retry-logic");
        assert_eq!(derive_slug("API v2 (draft)"), "api-v2-draft");
        assert_eq!(derive_slug("already-kebab"), "already-kebab");
        assert_eq!(derive_slug("Café Menu"), "caf-menu");
        assert_eq!(derive_slug("!!!"), "");
    }

    #[test]
    fn rejects_title_with_no_alphanumerics() {
        let tmp = tempdir().unwrap();
        seed_with_installed_template(tmp.path());
        let err = run(&args("!!! ***"), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidArgument { .. }));
    }

    #[test]
    fn falls_back_to_framework_source_template() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("framework/templates/spec")).unwrap();
        fs::write(
            tmp.path().join("framework/templates/spec/spec.md"),
            TEMPLATE,
        )
        .unwrap();
        let result = run(&args("fallback"), tmp.path()).unwrap();
        assert!(result.created);
        assert_eq!(
            result.template.as_deref(),
            Some("framework/templates/spec/spec.md")
        );
    }

    #[test]
    fn installed_template_wins_over_framework_source() {
        let tmp = tempdir().unwrap();
        seed_with_installed_template(tmp.path());
        fs::create_dir_all(tmp.path().join("framework/templates/spec")).unwrap();
        fs::write(
            tmp.path().join("framework/templates/spec/spec.md"),
            "# other\n",
        )
        .unwrap();
        let result = run(&args("ordered"), tmp.path()).unwrap();
        assert_eq!(result.template.as_deref(), Some("specs/templates/spec.md"));
        let body = fs::read_to_string(tmp.path().join("specs/001-ordered/spec.md")).unwrap();
        assert_eq!(body, TEMPLATE);
    }

    #[test]
    fn missing_template_errors_without_creating_the_directory() {
        let tmp = tempdir().unwrap();
        let err = run(&args("no template"), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::TemplateNotFound { .. }));
        assert!(
            !tmp.path().join("specs/001-no-template").exists(),
            "a missing template must not leave a half-scaffolded directory"
        );
    }

    #[test]
    fn never_touches_existing_feature_directories() {
        let tmp = tempdir().unwrap();
        seed_with_installed_template(tmp.path());
        fs::create_dir_all(tmp.path().join("specs/001-existing")).unwrap();
        fs::write(
            tmp.path().join("specs/001-existing/notes.md"),
            "hands off\n",
        )
        .unwrap();
        let result = run(&args("existing"), tmp.path()).unwrap();
        // max(001) + 1 → a fresh 002 dir; the 001 dir is untouched.
        assert!(result.created);
        assert_eq!(result.feature, "002-existing");
        let notes = fs::read_to_string(tmp.path().join("specs/001-existing/notes.md")).unwrap();
        assert_eq!(notes, "hands off\n");
    }

    #[test]
    fn refusal_branch_reports_domain_outcome() {
        // Directly cover the `feature_dir.exists()` refusal: a dir whose
        // name will be derived next already exists as a *file*-bearing
        // path. Achieved by making the target path exist as a plain file
        // (exists() is true for files too).
        let tmp = tempdir().unwrap();
        seed_with_installed_template(tmp.path());
        fs::write(tmp.path().join("specs/001-taken"), "not a dir\n").unwrap();
        let result = run(&args("taken"), tmp.path()).unwrap();
        assert!(!result.created, "refusal is a domain outcome, not an error");
        assert_eq!(result.feature, "001-taken");
        assert_eq!(result.path, "specs/001-taken");
        assert!(result.template.is_none());
        let body = fs::read_to_string(tmp.path().join("specs/001-taken")).unwrap();
        assert_eq!(body, "not a dir\n", "existing path untouched");
    }

    #[cfg(unix)]
    #[test]
    fn copied_spec_mirrors_template_mode() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempdir().unwrap();
        seed_with_installed_template(tmp.path());
        let template = tmp.path().join("specs/templates/spec.md");
        let mut perms = fs::metadata(&template).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&template, perms).unwrap();

        run(&args("mode check"), tmp.path()).unwrap();
        let dest_mode = fs::metadata(tmp.path().join("specs/001-mode-check/spec.md"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(
            dest_mode, 0o644,
            "write_atomic_bytes lands 0600; the template mode must be mirrored"
        );
    }
}
