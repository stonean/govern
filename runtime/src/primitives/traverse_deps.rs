//! `traverse-deps` — verify spec dependencies exist as directories and that
//! their pipeline status is compatible.
//!
//! Compatibility rule: a dependency is compatible when its `status` is
//! `planned`, `in-progress`, or `done`. Pipeline status `draft` or
//! `clarified` blocks consumers because there is no committed plan to build
//! against; absent dependency directories block always.

use std::path::Path;

use crate::primitives::{PrimitiveError, Result, read_text, split_frontmatter};
use crate::schema::primitives::{
    DependencyEdge, Frontmatter, TraverseDepsArgs, TraverseDepsResult,
};

const COMPATIBLE_STATUSES: &[&str] = &["planned", "in-progress", "done"];

/// Execute the `traverse-deps` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeatureNotFound`] when the feature directory
/// is absent, [`PrimitiveError::Io`] / [`PrimitiveError::MissingFrontmatter`]
/// on read or parse failures of the feature's own `spec.md`, and
/// [`PrimitiveError::Yaml`] when its frontmatter is malformed. Missing or
/// malformed dependency specs are reported as findings, not errors.
pub fn run(args: &TraverseDepsArgs, repo: &Path) -> Result<TraverseDepsResult> {
    let feature_dir = repo.join("specs").join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            feature: args.feature.clone(),
        });
    }
    let spec_path = feature_dir.join("spec.md");
    let content = read_text(&spec_path)?;
    let (fm_text, _body) = split_frontmatter(&content, &spec_path)?;
    let frontmatter: Frontmatter =
        serde_yaml::from_str(fm_text).map_err(|source| PrimitiveError::Yaml {
            path: spec_path.clone(),
            source,
        })?;

    let mut edges: Vec<DependencyEdge> = Vec::with_capacity(frontmatter.dependencies.len());
    let mut overall = true;
    for dep_name in &frontmatter.dependencies {
        let dep_dir = repo.join("specs").join(dep_name);
        let dep_spec = dep_dir.join("spec.md");
        let exists = dep_dir.is_dir() && dep_spec.is_file();
        let status = if exists {
            read_status(&dep_spec).unwrap_or_default()
        } else {
            String::new()
        };
        let compatible = exists && COMPATIBLE_STATUSES.contains(&status.as_str());
        if !compatible {
            overall = false;
        }
        edges.push(DependencyEdge {
            feature: dep_name.clone(),
            exists,
            status,
            compatible,
        });
    }

    Ok(TraverseDepsResult {
        dependencies: edges,
        compatible: overall,
    })
}

fn read_status(spec_path: &Path) -> Result<String> {
    let content = read_text(spec_path)?;
    let (fm_text, _body) = split_frontmatter(&content, spec_path)?;
    let parsed: Frontmatter =
        serde_yaml::from_str(fm_text).map_err(|source| PrimitiveError::Yaml {
            path: spec_path.into(),
            source,
        })?;
    Ok(parsed.status)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::path::PathBuf;

    fn fixture_repo() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/primitives/sample-repo")
    }

    #[test]
    fn dependent_resolves_basic_edge() {
        let repo = fixture_repo();
        let result = run(
            &TraverseDepsArgs {
                feature: "002-dependent".into(),
            },
            &repo,
        )
        .unwrap();
        assert_eq!(result.dependencies.len(), 1);
        let edge = &result.dependencies[0];
        assert_eq!(edge.feature, "001-basic");
        assert!(edge.exists);
        assert_eq!(edge.status, "clarified");
        // 001-basic is `clarified`, which is not in the compatible set —
        // dependents need at least `planned` upstream, so the edge (and
        // overall result) reports incompatible.
        assert!(!edge.compatible);
        assert!(!result.compatible);
    }

    #[test]
    fn planned_dependency_is_compatible() {
        let tmp = tempfile::tempdir().unwrap();
        let upstream = tmp.path().join("specs").join("100-upstream");
        std::fs::create_dir_all(&upstream).unwrap();
        std::fs::write(
            upstream.join("spec.md"),
            "---\nstatus: planned\ndependencies: []\n---\n\n# upstream\n",
        )
        .unwrap();
        let downstream = tmp.path().join("specs").join("101-downstream");
        std::fs::create_dir_all(&downstream).unwrap();
        std::fs::write(
            downstream.join("spec.md"),
            "---\nstatus: planned\ndependencies: [100-upstream]\n---\n\n# downstream\n",
        )
        .unwrap();
        let result = run(
            &TraverseDepsArgs {
                feature: "101-downstream".into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert!(result.compatible);
        assert_eq!(result.dependencies.len(), 1);
        assert!(result.dependencies[0].compatible);
    }

    #[test]
    fn missing_dependency_is_incompatible() {
        let tmp = tempfile::tempdir().unwrap();
        let feature_dir = tmp.path().join("specs").join("003-missing-dep");
        std::fs::create_dir_all(&feature_dir).unwrap();
        std::fs::write(
            feature_dir.join("spec.md"),
            "---\nstatus: planned\ndependencies: [999-nope]\n---\n\n# x\n",
        )
        .unwrap();
        let result = run(
            &TraverseDepsArgs {
                feature: "003-missing-dep".into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.compatible);
        assert_eq!(result.dependencies.len(), 1);
        let edge = &result.dependencies[0];
        assert_eq!(edge.feature, "999-nope");
        assert!(!edge.exists);
        assert!(edge.status.is_empty());
        assert!(!edge.compatible);
    }

    #[test]
    fn empty_dependencies_is_compatible() {
        let repo = fixture_repo();
        let result = run(
            &TraverseDepsArgs {
                feature: "001-basic".into(),
            },
            &repo,
        )
        .unwrap();
        assert!(result.compatible);
        assert!(result.dependencies.is_empty());
    }
}
