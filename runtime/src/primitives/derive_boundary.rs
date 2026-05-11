//! `derive-boundary` — compute the runtime write boundary from
//! `git diff --name-only <first-commit-on-spec-dir>..HEAD` plus the spec dir.

use std::collections::BTreeSet;
use std::path::Path;

use git2::{Repository, Sort};

use crate::primitives::{PrimitiveError, Result};
use crate::schema::primitives::{DeriveBoundaryArgs, DeriveBoundaryResult};

/// Execute the `derive-boundary` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeatureNotFound`] when the feature directory
/// is absent, [`PrimitiveError::NoSpecHistory`] when no commit touches the
/// spec dir, and [`PrimitiveError::Git`] for any libgit2 failure.
pub fn run(args: &DeriveBoundaryArgs, repo: &Path) -> Result<DeriveBoundaryResult> {
    let feature_dir = repo.join("specs").join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            feature: args.feature.clone(),
        });
    }
    let repository = Repository::discover(repo)?;
    let spec_prefix = format!("specs/{}/", args.feature);

    let first_commit = first_commit_for_prefix(&repository, &spec_prefix)?.ok_or_else(|| {
        PrimitiveError::NoSpecHistory {
            feature: args.feature.clone(),
        }
    })?;
    let head_oid = repository.head()?.peel_to_commit()?.id();

    let first = repository.find_commit(first_commit)?;
    let first_tree = first.tree()?;
    let head_tree = repository.find_commit(head_oid)?.tree()?;
    let diff = repository.diff_tree_to_tree(Some(&first_tree), Some(&head_tree), None)?;

    let mut boundary: BTreeSet<String> = BTreeSet::new();
    // Always include the spec dir glob so the boundary covers files inside
    // the feature's own folder even when they aren't on disk yet.
    boundary.insert(format!("specs/{}/**", args.feature));

    diff.foreach(
        &mut |delta, _| {
            for path in [delta.old_file().path(), delta.new_file().path()]
                .into_iter()
                .flatten()
            {
                let s = path.to_string_lossy().replace('\\', "/");
                if s.starts_with(&spec_prefix) {
                    continue;
                }
                boundary.insert(s);
            }
            true
        },
        None,
        None,
        None,
    )?;

    Ok(DeriveBoundaryResult {
        boundary: boundary.into_iter().collect(),
        first_commit: first_commit.to_string(),
        current_head: head_oid.to_string(),
    })
}

fn first_commit_for_prefix(repo: &Repository, prefix: &str) -> Result<Option<git2::Oid>> {
    let mut walk = repo.revwalk()?;
    walk.push_head()?;
    walk.set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)?;

    for oid in walk {
        let oid = oid?;
        if commit_touches_prefix(repo, oid, prefix)? {
            return Ok(Some(oid));
        }
    }
    Ok(None)
}

fn commit_touches_prefix(repo: &Repository, oid: git2::Oid, prefix: &str) -> Result<bool> {
    let commit = repo.find_commit(oid)?;
    let tree = commit.tree()?;
    let parent_tree = if commit.parent_count() == 0 {
        None
    } else {
        Some(commit.parent(0)?.tree()?)
    };
    let diff = repo.diff_tree_to_tree(parent_tree.as_ref(), Some(&tree), None)?;
    let mut touched = false;
    diff.foreach(
        &mut |delta, _| {
            for path in [delta.old_file().path(), delta.new_file().path()]
                .into_iter()
                .flatten()
            {
                if path.to_string_lossy().starts_with(prefix) {
                    touched = true;
                }
            }
            true
        },
        None,
        None,
        None,
    )?;
    Ok(touched)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use git2::{IndexAddOption, Repository, Signature};
    use std::fs;
    use std::path::Path;

    fn commit_all(repo: &Repository, message: &str) -> git2::Oid {
        let mut index = repo.index().unwrap();
        index.add_all(["*"], IndexAddOption::DEFAULT, None).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = Signature::now("Test", "test@example.com").unwrap();
        let parent = repo
            .head()
            .ok()
            .and_then(|h| h.target())
            .and_then(|oid| repo.find_commit(oid).ok());
        let parents: Vec<&git2::Commit> = parent.as_ref().into_iter().collect();
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
            .unwrap()
    }

    fn write(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    #[test]
    fn boundary_covers_touched_files_outside_spec_dir() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        // Initial unrelated commit.
        write(&tmp.path().join("README.md"), "# repo\n");
        commit_all(&repo, "chore: init");
        // First spec-touching commit creates the feature.
        write(
            &tmp.path().join("specs/020-demo/spec.md"),
            "---\nstatus: planned\n---\n\n# 020\n",
        );
        commit_all(&repo, "feat(020): plan");
        // Subsequent commit touches files outside the spec dir.
        write(&tmp.path().join("runtime/src/main.rs"), "fn main() {}\n");
        write(&tmp.path().join("README.md"), "# repo v2\n");
        commit_all(&repo, "feat(020): runtime bootstrap");

        let result = run(
            &DeriveBoundaryArgs {
                feature: "020-demo".into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.first_commit.is_empty());
        assert!(!result.current_head.is_empty());
        let boundary: std::collections::HashSet<&str> =
            result.boundary.iter().map(String::as_str).collect();
        assert!(boundary.contains("specs/020-demo/**"));
        assert!(boundary.contains("runtime/src/main.rs"));
        assert!(boundary.contains("README.md"));
    }

    #[test]
    fn missing_spec_history_errors() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        write(&tmp.path().join("README.md"), "# repo\n");
        commit_all(&repo, "chore: init");
        write(
            &tmp.path().join("specs/030-orphan/spec.md"),
            "---\nstatus: planned\n---\n\n# 030\n",
        );
        // No commit yet — the feature dir exists on disk but never landed in
        // git history.
        let result = run(
            &DeriveBoundaryArgs {
                feature: "030-orphan".into(),
            },
            tmp.path(),
        );
        match result {
            Err(PrimitiveError::NoSpecHistory { feature }) => assert_eq!(feature, "030-orphan"),
            other => panic!("expected NoSpecHistory, got {other:?}"),
        }
    }
}
