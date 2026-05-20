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

    // Second condition (per scenario check-stuck-tasks-md-advancement):
    // `stuck` only fires when the first incomplete subtask in tasks.md has
    // NOT advanced across the walked commit window. Position-based equality:
    // the linear line index of the first `- [ ]` group at `since-sha` is
    // compared against the same index at HEAD. Vacuous-false when either
    // index is unavailable (no tasks.md at since-sha, or no incomplete
    // subtasks remain at HEAD).
    let first_incomplete_unchanged = match since.as_deref() {
        Some(s) => first_incomplete_index_unchanged(&repository, &tasks_rel, s)?,
        None => false,
    };
    let stuck = count >= args.threshold && first_incomplete_unchanged;

    Ok(CheckStuckResult {
        commit_count: count,
        stuck,
        since_sha: since.unwrap_or_default(),
        threshold: args.threshold,
    })
}

/// Compare the linear line-index of the first `- [ ]` group in `tasks_rel`
/// at the commit `since_sha` vs. at HEAD. Returns `true` when both indices
/// exist and match (the first incomplete subtask hasn't advanced). Returns
/// `false` when either index is unavailable (no tasks.md at since, or all
/// subtasks complete at HEAD) — vacuous-false matches the scenario's edge
/// cases (no first-incomplete subtask at baseline / completion is the
/// opposite of stuck).
fn first_incomplete_index_unchanged(
    repo: &Repository,
    tasks_rel: &str,
    since_sha: &str,
) -> Result<bool> {
    let head_content = read_blob_at_head(repo, tasks_rel)?;
    let since_content = read_blob_at_commit(repo, since_sha, tasks_rel)?;
    let head_idx = first_incomplete_subtask_index(head_content.as_deref().unwrap_or(""));
    let since_idx = first_incomplete_subtask_index(since_content.as_deref().unwrap_or(""));
    Ok(match (head_idx, since_idx) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    })
}

/// Read the blob at `path_rel` from HEAD's tree, returning its UTF-8 content.
fn read_blob_at_head(repo: &Repository, path_rel: &str) -> Result<Option<String>> {
    let head = repo.head()?.peel_to_commit()?;
    read_blob_from_tree(repo, &head.tree()?, path_rel)
}

/// Read the blob at `path_rel` from the tree of the commit named `sha`.
fn read_blob_at_commit(repo: &Repository, sha: &str, path_rel: &str) -> Result<Option<String>> {
    let oid = git2::Oid::from_str(sha)?;
    let commit = repo.find_commit(oid)?;
    read_blob_from_tree(repo, &commit.tree()?, path_rel)
}

fn read_blob_from_tree(
    repo: &Repository,
    tree: &git2::Tree<'_>,
    path_rel: &str,
) -> Result<Option<String>> {
    let Some(entry) = tree.get_path(Path::new(path_rel)).ok() else {
        return Ok(None);
    };
    let blob = repo.find_blob(entry.id())?;
    Ok(std::str::from_utf8(blob.content()).ok().map(str::to_string))
}

/// Return the 0-based line index of the first `- [ ]` group in `content`,
/// or `None` when no incomplete subtask exists. Fenced code blocks are
/// skipped so example `- [ ]` lines inside `` ``` `` blocks do not match.
fn first_incomplete_subtask_index(content: &str) -> Option<usize> {
    let mut in_fence = false;
    for (idx, line) in content.lines().enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with("```") {
            in_fence = !in_fence;
            continue;
        }
        if in_fence {
            continue;
        }
        // Match `- [ ]` exactly (space inside the brackets, not [x]/[X]).
        // Allow leading whitespace before the `-` for nested list items.
        if trimmed.starts_with("- [ ]") {
            return Some(idx);
        }
    }
    None
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
        let status = read_blob_from_tree(repo, &tree, spec_rel)?
            .as_deref()
            .and_then(extract_status)
            .map(str::to_string);
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

    /// Reopen regression test: a spec that went
    /// `planned → in-progress → done → in-progress` must measure
    /// `commit_count` from the SECOND `in-progress` transition, not the
    /// first. The original 022 implementation captured the first
    /// transition, causing 023's first commit attempt on the living-specs
    /// task to fire `stuck: true` with `commit-count: 8` because the
    /// initial implementation window's commits still counted toward the
    /// reopen's count.
    #[test]
    fn reopen_measures_from_most_recent_in_progress_transition() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        let spec_path = tmp.path().join("specs/010-demo/spec.md");
        let tasks_path = tmp.path().join("specs/010-demo/tasks.md");

        // Original implementation window: planned → in-progress, plus
        // several tasks.md commits during the work.
        write(&spec_path, &spec("planned"));
        write(&tasks_path, "# Tasks\n\n## 1. Bootstrap\n");
        commit_all(&repo, "feat: plan");

        write(&spec_path, &spec("in-progress"));
        commit_all(&repo, "chore: begin original implementation"); // first transition

        for i in 1..=5 {
            write(&tasks_path, &format!("# Tasks v{i}\n\n## 1. Bootstrap\n"));
            commit_all(&repo, &format!("wip: pass {i}"));
        }

        // Close out: in-progress → done.
        write(&spec_path, &spec("done"));
        commit_all(&repo, "feat: ship");

        // Reopen: done → in-progress via /gov:ask's back-edge. The new
        // task starts fresh.
        write(&spec_path, &spec("in-progress"));
        commit_all(&repo, "chore: reopen for follow-on"); // second transition (this is `since`)

        // One tasks.md commit during the reopen window.
        write(
            &tasks_path,
            "# Tasks v6\n\n## 1. Bootstrap\n\n## 2. Follow-on\n",
        );
        commit_all(&repo, "wip: start follow-on");

        let result = run(
            &CheckStuckArgs {
                feature: "010-demo".into(),
                threshold: 3,
            },
            tmp.path(),
        )
        .unwrap();
        // Count must be 1 (only the reopen-window tasks.md commit), not 6
        // (the original-window commits plus the reopen commit). And not
        // stuck — threshold is 3.
        assert_eq!(
            result.commit_count, 1,
            "expected count from most-recent in-progress, got {} (likely measured from first transition)",
            result.commit_count
        );
        assert!(!result.stuck);
        assert!(!result.since_sha.is_empty());

        // Sanity: walking by the captured since_sha should land at the
        // reopen commit, not the original begin commit.
        let repo2 = Repository::open(tmp.path()).unwrap();
        let oid = git2::Oid::from_str(&result.since_sha).unwrap();
        let commit = repo2.find_commit(oid).unwrap();
        assert_eq!(
            commit.message().unwrap(),
            "chore: reopen for follow-on",
            "since_sha should point at the reopen commit, not the original begin"
        );
    }

    /// Counterpart to the reopen test: a spec that has NEVER been
    /// `done` (no reopen has occurred) must still produce the correct
    /// count from its single `in-progress` transition. The fix to the
    /// reopen case must not break this routine path.
    #[test]
    fn first_in_progress_works_when_never_reopened() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        let spec_path = tmp.path().join("specs/010-demo/spec.md");
        let tasks_path = tmp.path().join("specs/010-demo/tasks.md");
        write(&spec_path, &spec("planned"));
        write(&tasks_path, "# Tasks\n");
        commit_all(&repo, "feat: plan");
        write(&spec_path, &spec("in-progress"));
        commit_all(&repo, "chore: begin");
        for i in 1..=2 {
            write(&tasks_path, &format!("# Tasks v{i}\n"));
            commit_all(&repo, &format!("wip: pass {i}"));
        }
        let result = run(
            &CheckStuckArgs {
                feature: "010-demo".into(),
                threshold: 3,
            },
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.commit_count, 2);
        assert!(!result.stuck);
    }

    /// Mechanical sweep commits between the most-recent `in-progress`
    /// transition and HEAD touch `spec.md` but do NOT change the
    /// `status:` line. They must not register as new in-progress
    /// transitions (which would skip past them to the older, wrong
    /// transition).
    #[test]
    fn mechanical_sweeps_do_not_disturb_since_sha() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        let spec_path = tmp.path().join("specs/010-demo/spec.md");
        let tasks_path = tmp.path().join("specs/010-demo/tasks.md");

        write(&spec_path, &spec("planned"));
        write(&tasks_path, "# Tasks\n");
        commit_all(&repo, "feat: plan");
        write(&spec_path, &spec("in-progress"));
        commit_all(&repo, "chore: begin");

        // Mechanical sweep on spec.md (e.g., rename pass) — same status
        // value, different body content.
        let mut sweeped = spec("in-progress");
        sweeped.push_str("\nMechanical sweep added this line.\n");
        write(&spec_path, &sweeped);
        commit_all(&repo, "chore: rename sweep across specs");

        // One legitimate tasks.md commit after the sweep.
        write(&tasks_path, "# Tasks v2\n");
        commit_all(&repo, "wip: pass 1");

        let result = run(
            &CheckStuckArgs {
                feature: "010-demo".into(),
                threshold: 3,
            },
            tmp.path(),
        )
        .unwrap();
        // Count is just the tasks.md commit (1). The sweep commit doesn't
        // touch tasks.md so it doesn't count toward stuck.
        assert_eq!(result.commit_count, 1);
        assert!(!result.stuck);
    }

    /// The false-positive case the `check-stuck-tasks-md-advancement`
    /// scenario closes: threshold-count commits land on `tasks.md`, BUT
    /// each one flips a different subtask checkbox. Under the old
    /// implementation (count >= threshold alone), `stuck: true` would fire
    /// despite real progress. With the second condition (first-incomplete
    /// index unchanged), the check correctly reports `stuck: false`.
    #[test]
    fn stuck_false_when_checkboxes_flipped_across_threshold_commits() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        let spec_path = tmp.path().join("specs/010-demo/spec.md");
        let tasks_path = tmp.path().join("specs/010-demo/tasks.md");

        write(&spec_path, &spec("planned"));
        write(
            &tasks_path,
            "# Tasks\n\n## 1. Bootstrap\n\n- [ ] subtask A\n- [ ] subtask B\n- [ ] subtask C\n",
        );
        commit_all(&repo, "feat: plan");

        write(&spec_path, &spec("in-progress"));
        commit_all(&repo, "chore: begin");

        // Three commits, each flipping a subsequent checkbox. The first
        // incomplete subtask's index advances every commit (line 4 → 5 → 6).
        write(
            &tasks_path,
            "# Tasks\n\n## 1. Bootstrap\n\n- [x] subtask A\n- [ ] subtask B\n- [ ] subtask C\n",
        );
        commit_all(&repo, "wip: flip A");
        write(
            &tasks_path,
            "# Tasks\n\n## 1. Bootstrap\n\n- [x] subtask A\n- [x] subtask B\n- [ ] subtask C\n",
        );
        commit_all(&repo, "wip: flip B");
        write(
            &tasks_path,
            "# Tasks\n\n## 1. Bootstrap\n\n- [x] subtask A\n- [x] subtask B\n- [x] subtask C\n",
        );
        commit_all(&repo, "wip: flip C");

        // Add one more commit so count clearly exceeds threshold, again
        // with a different first-incomplete state — re-introduce one.
        write(
            &tasks_path,
            "# Tasks\n\n## 1. Bootstrap\n\n- [x] subtask A\n- [x] subtask B\n- [x] subtask C\n\n## 2. Follow-on\n\n- [ ] D\n",
        );
        commit_all(&repo, "wip: add follow-on task");

        let result = run(
            &CheckStuckArgs {
                feature: "010-demo".into(),
                threshold: 3,
            },
            tmp.path(),
        )
        .unwrap();
        // 4 commits touched tasks.md → count exceeds threshold.
        assert_eq!(result.commit_count, 4);
        // But subtasks advanced across the window: the first incomplete
        // subtask moved from line 4 (subtask A) at since-sha to a later
        // index (line 9, the new Follow-on D) at HEAD. NOT stuck.
        assert!(
            !result.stuck,
            "stuck must not fire when first-incomplete index has advanced"
        );
    }
}
