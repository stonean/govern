//! `check-stuck` — count commits on `tasks.md` since the spec entered
//! `in-progress`, surfacing cycles where the same task is touched repeatedly.

use std::path::Path;

use git2::{Repository, Sort};

use crate::primitives::{PrimitiveError, Result};
use crate::schema::primitives::{CheckStuckArgs, CheckStuckResult};

/// Execute the `check-stuck` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeatureNotFound`] when the feature directory
/// is absent and [`PrimitiveError::Git`] for any libgit2 failure (repo not
/// found, walk failure, tree lookup, etc.).
pub fn run(args: &CheckStuckArgs, repo: &Path) -> Result<CheckStuckResult> {
    let feature_dir = repo.join("specs").join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            feature: args.feature.clone(),
        });
    }
    let repository = Repository::discover(repo)?;
    let spec_rel = format!("specs/{}/spec.md", args.feature);
    let tasks_rel = format!("specs/{}/tasks.md", args.feature);

    let since = find_in_progress_commit(&repository, &spec_rel)?;
    let count = count_commits_touching(&repository, &tasks_rel, since.as_deref())?;
    let stuck = count >= args.threshold;

    Ok(CheckStuckResult {
        commit_count: count,
        stuck,
        since_sha: since.unwrap_or_default(),
        threshold: args.threshold,
    })
}

/// Walk commits oldest-first looking for the most recent commit whose
/// `spec.md` content transitions `status` *into* `in-progress`. Returns the
/// sha of that commit, or `None` when no such transition exists.
fn find_in_progress_commit(repo: &Repository, spec_rel: &str) -> Result<Option<String>> {
    let mut walk = repo.revwalk()?;
    walk.push_head()?;
    walk.set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)?;

    let mut newest_in_progress: Option<String> = None;
    let mut previous_status: Option<String> = None;
    for oid in walk {
        let oid = oid?;
        let commit = repo.find_commit(oid)?;
        let tree = commit.tree()?;
        let status = match tree.get_path(Path::new(spec_rel)).ok() {
            Some(entry) => {
                let blob = repo.find_blob(entry.id())?;
                extract_status(std::str::from_utf8(blob.content()).unwrap_or(""))
                    .map(str::to_string)
            }
            None => None,
        };
        if previous_status.as_deref() != Some("in-progress")
            && status.as_deref() == Some("in-progress")
        {
            newest_in_progress = Some(oid.to_string());
        }
        previous_status = status;
    }
    Ok(newest_in_progress)
}

fn count_commits_touching(
    repo: &Repository,
    path_rel: &str,
    since_sha: Option<&str>,
) -> Result<u32> {
    let Some(since) = since_sha else {
        return Ok(0);
    };
    // Walk the first-parent chain from HEAD until we reach `since`.
    // TIME-sorted revwalk is unstable when commits share timestamps; this
    // linear traversal is deterministic for the linear-history case.
    let mut current = Some(repo.head()?.peel_to_commit()?);
    let mut count: u32 = 0;
    while let Some(commit) = current.take() {
        let oid = commit.id();
        let sha = oid.to_string();
        let touched = commit_touches(repo, oid, path_rel)?;
        if sha == since {
            // Inclusive of the in-progress commit per the data-model
            // semantics: the count begins at the transition.
            if touched {
                count = count.saturating_add(1);
            }
            break;
        }
        if touched {
            count = count.saturating_add(1);
        }
        if commit.parent_count() > 0 {
            current = Some(commit.parent(0)?);
        }
    }
    Ok(count)
}

fn commit_touches(repo: &Repository, oid: git2::Oid, path_rel: &str) -> Result<bool> {
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
            let old_match = delta
                .old_file()
                .path()
                .is_some_and(|p| p == Path::new(path_rel));
            let new_match = delta
                .new_file()
                .path()
                .is_some_and(|p| p == Path::new(path_rel));
            if old_match || new_match {
                touched = true;
            }
            true
        },
        None,
        None,
        None,
    )?;
    Ok(touched)
}

fn extract_status(content: &str) -> Option<&str> {
    let after_open = content
        .strip_prefix("---\n")
        .or_else(|| content.strip_prefix("---\r\n"))?;
    let end = after_open.find("\n---\n")?;
    let fm = &after_open[..end];
    for line in fm.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("status:") {
            return Some(rest.trim().trim_matches('"'));
        }
    }
    None
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

    fn spec(status: &str) -> String {
        format!(
            "---\nstatus: {status}\ndependencies: []\n---\n\n# X\n\n## Acceptance Criteria\n\n- [ ] one\n"
        )
    }

    #[test]
    fn counts_commits_since_in_progress() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        let spec_path = tmp.path().join("specs/010-demo/spec.md");
        let tasks_path = tmp.path().join("specs/010-demo/tasks.md");
        write(&spec_path, &spec("planned"));
        write(&tasks_path, "# Tasks\n\n## 1. Bootstrap\n\n- [ ] start\n");
        commit_all(&repo, "feat(010): plan");

        // Advance to in-progress.
        write(&spec_path, &spec("in-progress"));
        commit_all(&repo, "chore(010): begin");

        // Touch tasks.md three times.
        for i in 1..=3 {
            write(
                &tasks_path,
                &format!("# Tasks v{i}\n\n## 1. Bootstrap\n\n- [ ] still\n"),
            );
            commit_all(&repo, &format!("wip(010): pass {i}"));
        }

        let result = run(
            &CheckStuckArgs {
                feature: "010-demo".into(),
                threshold: 3,
            },
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.threshold, 3);
        assert_eq!(result.commit_count, 3);
        assert!(result.stuck);
        assert!(!result.since_sha.is_empty());
    }

    #[test]
    fn no_in_progress_yields_zero() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        let spec_path = tmp.path().join("specs/010-demo/spec.md");
        let tasks_path = tmp.path().join("specs/010-demo/tasks.md");
        write(&spec_path, &spec("planned"));
        write(&tasks_path, "# Tasks\n");
        commit_all(&repo, "feat(010): plan");
        write(&tasks_path, "# Tasks v2\n");
        commit_all(&repo, "wip");
        let result = run(
            &CheckStuckArgs {
                feature: "010-demo".into(),
                threshold: 3,
            },
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.commit_count, 0);
        assert!(!result.stuck);
        assert!(result.since_sha.is_empty());
    }
}
