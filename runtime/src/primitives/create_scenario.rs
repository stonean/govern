//! `create-scenario` — write a new `scenarios/{slug}.md` file under a feature.
//!
//! Mirrors the scenario-creation phase of `/elaborate` (and, post-spec-023,
//! `/amend`'s scenario branch): writes a single file with `section` frontmatter
//! and prose body sections atomically via tempfile + rename, creating the
//! `scenarios/` subdirectory when absent. Refuses to overwrite an existing
//! scenario.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::primitives::{
    PrimitiveError, Result, rel_path, validate_no_traversal, validate_slug, write_atomic,
};
use crate::schema::primitives::{CreateScenarioArgs, CreateScenarioResult};

/// Execute the `create-scenario` primitive.
///
/// `args.feature_path` is resolved relative to `repo`. The scenario file lands
/// at `{repo}/{feature_path}/scenarios/{slug}.md`. The atomic write pattern
/// leaves the destination unchanged on a crash mid-write; the orphaned
/// tempfile is the only recovery artifact.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeaturePathNotFound`] when the resolved feature
/// directory does not exist, [`PrimitiveError::ScenarioConflict`] when a
/// file already lives at the destination path, or [`PrimitiveError::Io`]
/// for filesystem failures during the write.
pub fn run(args: &CreateScenarioArgs, repo: &Path) -> Result<CreateScenarioResult> {
    validate_no_traversal(&args.feature_path)?;
    validate_slug(&args.slug)?;
    let feature_dir = repo.join(&args.feature_path);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeaturePathNotFound {
            path: PathBuf::from(&args.feature_path),
        });
    }

    let scenarios_dir = feature_dir.join("scenarios");
    let dest = scenarios_dir.join(format!("{}.md", args.slug));
    if dest.exists() {
        return Err(PrimitiveError::ScenarioConflict { path: dest.clone() });
    }

    let body = render(args);
    write_atomic(&dest, &body)?;
    Ok(CreateScenarioResult {
        created: rel_path(&dest, repo),
    })
}

/// Render the scenario markdown body. The shape mirrors
/// `framework/templates/spec/scenario.md`: frontmatter, H1 from the slug,
/// Context / Behavior / (optional) Edge Cases / Open Questions / Resolved
/// Questions sections.
fn render(args: &CreateScenarioArgs) -> String {
    let title = title_from_slug(&args.slug);
    let mut out = String::new();
    out.push_str("---\n");
    let _ = writeln!(out, "section: \"{}\"", args.section);
    out.push_str("---\n\n");
    let _ = writeln!(out, "# {title}\n");
    out.push_str("## Context\n\n");
    out.push_str(args.context.trim_end());
    out.push_str("\n\n");
    out.push_str("## Behavior\n\n");
    out.push_str(args.behavior.trim_end());
    out.push_str("\n\n");
    if let Some(edge_cases) = &args.edge_cases {
        out.push_str("## Edge Cases\n\n");
        out.push_str(edge_cases.trim_end());
        out.push_str("\n\n");
    }
    out.push_str("## Open Questions\n\n");
    out.push_str("*None — captured during scenario authoring.*\n\n");
    out.push_str("## Resolved Questions\n\n");
    out.push_str("*None yet.*\n");
    out
}

/// Capitalize the slug's first character for the H1 heading. Hyphens are
/// preserved so the H1 mirrors the slug ("ask-consolidation" → "Ask-consolidation");
/// existing govern scenarios follow this pattern. Callers are free to edit
/// the H1 afterward for a more descriptive title.
fn title_from_slug(slug: &str) -> String {
    let mut chars = slug.chars();
    match chars.next() {
        Some(c) => c.to_uppercase().chain(chars).collect::<String>(),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn args(feature_path: &str, slug: &str, edge_cases: Option<&str>) -> CreateScenarioArgs {
        CreateScenarioArgs {
            feature_path: feature_path.into(),
            slug: slug.into(),
            section: "Follow-on scenarios".into(),
            context: "Upstream may time out.".into(),
            behavior: "Client retries up to three times.".into(),
            edge_cases: edge_cases.map(str::to_string),
        }
    }

    fn make_feature(tmp: &Path, feature_path: &str) {
        fs::create_dir_all(tmp.join(feature_path)).unwrap();
    }

    #[test]
    fn writes_scenario_with_full_sections() {
        let tmp = tempdir().unwrap();
        make_feature(tmp.path(), "specs/042-foo");
        let result = run(
            &args("specs/042-foo", "retry-on-timeout", Some("Network jitter.")),
            tmp.path(),
        )
        .unwrap();
        assert_eq!(
            result.created,
            "specs/042-foo/scenarios/retry-on-timeout.md"
        );
        let body = fs::read_to_string(
            tmp.path()
                .join("specs/042-foo/scenarios/retry-on-timeout.md"),
        )
        .unwrap();
        assert!(body.starts_with("---\nsection: \"Follow-on scenarios\"\n---\n"));
        assert!(body.contains("# Retry-on-timeout"));
        assert!(body.contains("## Context\n\nUpstream may time out."));
        assert!(body.contains("## Behavior\n\nClient retries up to three times."));
        assert!(body.contains("## Edge Cases\n\nNetwork jitter."));
        assert!(body.contains("## Open Questions"));
        assert!(body.contains("## Resolved Questions"));
    }

    #[test]
    fn omits_edge_cases_section_when_absent() {
        let tmp = tempdir().unwrap();
        make_feature(tmp.path(), "specs/042-foo");
        run(&args("specs/042-foo", "simple", None), tmp.path()).unwrap();
        let body =
            fs::read_to_string(tmp.path().join("specs/042-foo/scenarios/simple.md")).unwrap();
        assert!(!body.contains("## Edge Cases"));
        assert!(body.contains("## Behavior"));
    }

    #[test]
    fn creates_scenarios_subdirectory_when_absent() {
        let tmp = tempdir().unwrap();
        make_feature(tmp.path(), "specs/042-foo");
        assert!(!tmp.path().join("specs/042-foo/scenarios").exists());
        run(&args("specs/042-foo", "first", None), tmp.path()).unwrap();
        assert!(tmp.path().join("specs/042-foo/scenarios").is_dir());
    }

    #[test]
    fn refuses_when_slug_already_exists() {
        let tmp = tempdir().unwrap();
        make_feature(tmp.path(), "specs/042-foo");
        run(&args("specs/042-foo", "dupe", None), tmp.path()).unwrap();
        let err = run(&args("specs/042-foo", "dupe", None), tmp.path()).unwrap_err();
        match err {
            PrimitiveError::ScenarioConflict { path } => {
                assert!(path.ends_with("specs/042-foo/scenarios/dupe.md"));
            }
            other => panic!("expected ScenarioConflict, got {other:?}"),
        }
    }

    #[test]
    fn refuses_when_feature_path_is_missing() {
        let tmp = tempdir().unwrap();
        let err = run(&args("specs/999-nope", "x", None), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::FeaturePathNotFound { .. }));
    }

    #[test]
    fn dropping_named_tempfile_leaves_no_destination() {
        use std::io::Write;
        let tmp = tempdir().unwrap();
        make_feature(tmp.path(), "specs/042-foo");
        let scenarios_dir = tmp.path().join("specs/042-foo/scenarios");
        fs::create_dir_all(&scenarios_dir).unwrap();
        let dest = scenarios_dir.join("interrupted.md");
        {
            let mut tf = tempfile::NamedTempFile::new_in(&scenarios_dir).unwrap();
            tf.write_all(b"INTERRUPTED").unwrap();
        }
        assert!(!dest.exists());
    }

    #[test]
    fn title_from_slug_capitalizes_first_letter_preserving_hyphens() {
        assert_eq!(title_from_slug("ask-consolidation"), "Ask-consolidation");
        assert_eq!(title_from_slug("retry"), "Retry");
        assert_eq!(title_from_slug(""), "");
    }

    #[test]
    fn refuses_when_slug_contains_path_separator() {
        let tmp = tempdir().unwrap();
        make_feature(tmp.path(), "specs/042-foo");
        let err = run(&args("specs/042-foo", "../escape", None), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidSlug { .. }));
    }

    #[test]
    fn refuses_when_slug_starts_with_dot() {
        let tmp = tempdir().unwrap();
        make_feature(tmp.path(), "specs/042-foo");
        let err = run(&args("specs/042-foo", ".hidden", None), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidSlug { .. }));
    }

    #[test]
    fn refuses_when_feature_path_has_parent_component() {
        let tmp = tempdir().unwrap();
        make_feature(tmp.path(), "specs/042-foo");
        let err = run(&args("specs/../target", "x", None), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidPath { .. }));
    }
}
