//! `check-stuck` — count commits on `tasks.md` since the spec entered
//! `in-progress`, surfacing cycles where the same task is touched repeatedly.
//!
//! History reading is branch-shape tolerant (scenario
//! `primitive-robustness-hardening`): the `in-progress` transition commit
//! is found even when it landed on a merged side branch, transitions are
//! detected by comparing each commit's status against its *own parents'*
//! blobs (never against walk-order neighbors, which fabricates phantom
//! transitions across topologically adjacent commits of different
//! branches), and the commit count uses `since..HEAD` reachability rather
//! than a first-parent walk (which never reaches a side-branch `since`).
//! On a repo with no merge commits the results are identical to the
//! pre-hardening behavior.

use std::collections::HashMap;
use std::path::Path;

use git2::{Repository, Sort};

use crate::primitives::{PrimitiveError, Result, SkipScanner, frontmatter_status};
use crate::schema::paths;
use crate::schema::primitives::{CheckStuckArgs, CheckStuckResult};

/// Execute the `check-stuck` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeatureNotFound`] when the feature directory
/// is absent and [`PrimitiveError::Git`] for any libgit2 failure (repo not
/// found, walk failure, tree lookup, etc.).
pub fn run(args: &CheckStuckArgs, repo: &Path) -> Result<CheckStuckResult> {
    super::validate_no_traversal(&args.feature)?;
    let layout = paths::Paths::load(repo);
    let feature_dir = repo.join(&layout.specs_root).join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            root: layout.specs_root.clone(),
            feature: args.feature.clone(),
        });
    }
    let repository = Repository::discover(repo)?;
    let spec_rel = format!("{}/{}/spec.md", layout.specs_root, args.feature);
    let tasks_rel = format!("{}/{}/tasks.md", layout.specs_root, args.feature);

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
/// or `None` when no incomplete subtask exists. Fenced code blocks and HTML
/// comments are skipped so example `- [ ]` lines inside `` ``` `` blocks or
/// `<!-- … -->` guidance comments do not match.
fn first_incomplete_subtask_index(content: &str) -> Option<usize> {
    let mut skip = SkipScanner::default();
    for (idx, line) in content.lines().enumerate() {
        if skip.skip(line) {
            continue;
        }
        // Match `- [ ]` exactly (space inside the brackets, not [x]/[X]).
        // Allow leading whitespace before the `-` for nested list items.
        if line.trim_start().starts_with("- [ ]") {
            return Some(idx);
        }
    }
    None
}

/// Walk every commit reachable from HEAD looking for the most recent
/// commit whose `spec.md` transitions `status` *into* `in-progress`.
/// Returns the sha of that commit, or `None` when no such transition
/// exists.
///
/// Branch-shape tolerance: the walk covers all parents (not just the
/// first-parent chain), so a transition commit that landed on a merged
/// side branch is still visited. A commit is a transition exactly when
/// its own status is `in-progress` and *none of its own parents'* is —
/// comparing against walk-order neighbors (the pre-hardening behavior)
/// fabricates phantom transitions when the topological order interleaves
/// commits from different branches. A merge that simply carries an
/// already-in-progress parent forward is not a transition; the real
/// transition lives in that parent's history and is found there.
///
/// With `TOPOLOGICAL | REVERSE` sorting parents are always visited
/// before children, so "most recent" is the last transition seen.
///
/// The status-parse memoization is keyed by the spec's tree-entry **blob
/// OID**, not the commit OID: the `spec.md` blob is byte-identical across
/// nearly every commit (status changes only at transitions), so a
/// blob-keyed cache collapses the parse to one per distinct spec version
/// rather than one per commit — turning tens of thousands of redundant
/// frontmatter parses on a big-history repo into a handful.
///
/// Shared with `compute-review-scope`, which uses the same commit as its
/// default `diff-base`.
pub(crate) fn find_in_progress_commit(repo: &Repository, spec_rel: &str) -> Result<Option<String>> {
    let mut walk = repo.revwalk()?;
    walk.push_head()?;
    walk.set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)?;

    let mut newest_in_progress: Option<String> = None;
    let mut status_by_blob: HashMap<git2::Oid, Option<String>> = HashMap::new();
    for oid in walk {
        let oid = oid?;
        let status = status_of_commit(repo, &mut status_by_blob, oid, spec_rel)?;
        if status.as_deref() != Some("in-progress") {
            continue;
        }
        let commit = repo.find_commit(oid)?;
        let mut parent_in_progress = false;
        for parent in commit.parent_ids() {
            if status_of_commit(repo, &mut status_by_blob, parent, spec_rel)?.as_deref()
                == Some("in-progress")
            {
                parent_in_progress = true;
                break;
            }
        }
        if !parent_in_progress {
            newest_in_progress = Some(oid.to_string());
        }
    }
    Ok(newest_in_progress)
}

/// Read (with memoization) the spec's `status:` value at a commit.
///
/// `cache` is keyed by the spec's tree-entry **blob OID** rather than the
/// commit OID: identical spec content shares a blob, so this collapses the
/// parse to one per distinct spec version across the whole walk (see
/// [`find_in_progress_commit`]). A commit whose tree has no `spec.md`
/// (blob absent) has no status and is not cached — that case is cheap and
/// carries no reusable parse.
fn status_of_commit(
    repo: &Repository,
    cache: &mut HashMap<git2::Oid, Option<String>>,
    oid: git2::Oid,
    spec_rel: &str,
) -> Result<Option<String>> {
    let commit = repo.find_commit(oid)?;
    let Some(blob_id) = tree_entry_id(&commit.tree()?, spec_rel) else {
        return Ok(None);
    };
    if let Some(cached) = cache.get(&blob_id) {
        return Ok(cached.clone());
    }
    // Frontmatter reading reuses the shared [`frontmatter_status`] helper —
    // CRLF-aware splitting plus real YAML parsing; the hand-rolled
    // predecessor missed `\r\n---\r\n` close fences, so a CRLF checkout's
    // transitions were invisible.
    let blob = repo.find_blob(blob_id)?;
    let status = std::str::from_utf8(blob.content())
        .ok()
        .and_then(|content| frontmatter_status(content, Path::new(spec_rel)));
    cache.insert(blob_id, status.clone());
    Ok(status)
}

/// Count commits touching `path_rel` in the `since..HEAD` reachability
/// set, plus the `since` commit itself when it touches the file
/// (inclusive per the data-model semantics: the count begins at the
/// transition). Reachability-based selection is branch-shape tolerant: a
/// `since` on a merged side branch is still an ancestor cutoff, and side
/// branch commits merged after the transition are counted exactly once
/// (see [`commit_touches`] for the merge-commit TREESAME rule). The
/// count is a property of the commit *set*, so walk order never affects
/// the result — deterministic without relying on TIME sorting.
fn count_commits_touching(
    repo: &Repository,
    path_rel: &str,
    since_sha: Option<&str>,
) -> Result<u32> {
    let Some(since) = since_sha else {
        return Ok(0);
    };
    let since_oid = git2::Oid::from_str(since)?;
    let mut walk = repo.revwalk()?;
    walk.push_head()?;
    walk.hide(since_oid)?;
    let mut count: u32 = 0;
    for oid in walk {
        if commit_touches(repo, oid?, path_rel)? {
            count = count.saturating_add(1);
        }
    }
    if commit_touches(repo, since_oid, path_rel)? {
        count = count.saturating_add(1);
    }
    Ok(count)
}

/// Whether a commit changed `path_rel`. For a root commit: the path
/// exists in its tree. For a single-parent commit: the blob differs from
/// the parent's (add/remove/modify all differ). For a merge commit: the
/// blob differs from *every* parent — git's TREESAME rule. A merge that
/// merely integrates a side branch's edit is TREESAME to that side
/// parent and does not count; the side branch's own commits are in the
/// walked set and carry the count, so nothing is double-counted. A merge
/// whose conflict resolution produced content unlike any parent counts
/// once, as it should.
fn commit_touches(repo: &Repository, oid: git2::Oid, path_rel: &str) -> Result<bool> {
    let commit = repo.find_commit(oid)?;
    let own = tree_entry_id(&commit.tree()?, path_rel);
    if commit.parent_count() == 0 {
        return Ok(own.is_some());
    }
    for parent in commit.parents() {
        if tree_entry_id(&parent.tree()?, path_rel) == own {
            return Ok(false);
        }
    }
    Ok(true)
}

/// The object id at `path_rel` in `tree`, or `None` when absent.
fn tree_entry_id(tree: &git2::Tree<'_>, path_rel: &str) -> Option<git2::Oid> {
    tree.get_path(Path::new(path_rel)).ok().map(|e| e.id())
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

    /// Stage the full workdir and create a commit with explicit parents.
    /// `update_head: false` leaves HEAD where it is (side-branch commit);
    /// `true` advances HEAD (libgit2 requires the current tip to be the
    /// first parent, which merge fixtures satisfy by listing it first).
    fn commit_with_parents(
        repo: &Repository,
        message: &str,
        parents: &[git2::Oid],
        update_head: bool,
    ) -> git2::Oid {
        let mut index = repo.index().unwrap();
        index.add_all(["*"], IndexAddOption::DEFAULT, None).unwrap();
        index.write().unwrap();
        let tree_id = index.write_tree().unwrap();
        let tree = repo.find_tree(tree_id).unwrap();
        let sig = Signature::now("Test", "test@example.com").unwrap();
        let parent_commits: Vec<git2::Commit> = parents
            .iter()
            .map(|oid| repo.find_commit(*oid).unwrap())
            .collect();
        let parent_refs: Vec<&git2::Commit> = parent_commits.iter().collect();
        let refname = if update_head { Some("HEAD") } else { None };
        repo.commit(refname, &sig, &sig, message, &tree, &parent_refs)
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

    /// CRLF variant of [`spec`] — close fence is `\r\n---\r\n`, invisible
    /// to a splitter that only knows the LF form.
    fn spec_crlf(status: &str) -> String {
        format!("---\r\nstatus: {status}\r\ndependencies: []\r\n---\r\n\r\n# X\r\n")
    }

    #[test]
    fn counts_commits_under_configured_specs_root() {
        // Spec 040: the git-relative spec/tasks paths track `[paths] specs-root`,
        // so stuck detection works when the spec root is renamed.
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        write(
            &tmp.path().join(".govern.toml"),
            "[paths]\nspecs-root = \"governance\"\n",
        );
        let spec_path = tmp.path().join("governance/010-demo/spec.md");
        let tasks_path = tmp.path().join("governance/010-demo/tasks.md");
        write(&spec_path, &spec("planned"));
        write(&tasks_path, "# Tasks\n\n## 1. Bootstrap\n\n- [ ] start\n");
        commit_all(&repo, "feat(010): plan");

        write(&spec_path, &spec("in-progress"));
        commit_all(&repo, "chore(010): begin");

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
        assert_eq!(result.commit_count, 3, "counts commits under governance/");
        assert!(result.stuck);
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

        // Reopen: done → in-progress via /gov:amend's back-edge. The new
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

    /// Branch-shape tolerance (scenario primitive-robustness-hardening):
    /// the `in-progress` transition commit landed on a side branch that
    /// was merged into main. The transition must still be found (the
    /// side-branch commit is `since`), and the count must cover the
    /// post-merge tasks.md commits — a first-parent-only walk from HEAD
    /// never reaches a side-branch `since` and would count back to root.
    #[test]
    fn transition_on_merged_side_branch_is_found() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        let spec_path = tmp.path().join("specs/010-demo/spec.md");
        let tasks_path = tmp.path().join("specs/010-demo/tasks.md");

        write(&spec_path, &spec("planned"));
        write(&tasks_path, "# Tasks\n\n## 1. Bootstrap\n\n- [ ] start\n");
        let m1 = commit_all(&repo, "feat: plan");

        // Side branch off m1 flips the spec to in-progress. HEAD stays
        // at m1.
        write(&spec_path, &spec("in-progress"));
        let s1 = commit_with_parents(&repo, "side: begin", &[m1], false);

        // Main advances independently (spec restored to planned, an
        // unrelated file added).
        write(&spec_path, &spec("planned"));
        write(&tmp.path().join("README.md"), "readme\n");
        let m2 = commit_all(&repo, "main: unrelated");

        // Merge the side branch: the merged tree carries in-progress.
        write(&spec_path, &spec("in-progress"));
        commit_with_parents(&repo, "merge side", &[m2, s1], true);

        // Two tasks.md commits after the merge; the first incomplete
        // subtask stays at the same line index (genuinely stuck).
        for i in 1..=2 {
            write(
                &tasks_path,
                &format!("# Tasks v{i}\n\n## 1. Bootstrap\n\n- [ ] still\n"),
            );
            commit_all(&repo, &format!("wip: pass {i}"));
        }

        let result = run(
            &CheckStuckArgs {
                feature: "010-demo".into(),
                threshold: 2,
            },
            tmp.path(),
        )
        .unwrap();
        assert_eq!(
            result.since_sha,
            s1.to_string(),
            "since must be the side-branch transition commit"
        );
        assert_eq!(
            result.commit_count, 2,
            "only the two post-merge tasks.md commits count"
        );
        assert!(result.stuck);
    }

    /// Phantom-transition regression: a side branch whose spec never left
    /// `planned` merges into a main line that has been `in-progress` for
    /// several commits. Under walk-order neighbor tracking the planned
    /// side commit interleaves before the merge and fabricates a
    /// transition at the merge commit; per-parent comparison keeps the
    /// real (older) transition.
    #[test]
    fn merged_planned_branch_does_not_fabricate_transition() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        let spec_path = tmp.path().join("specs/010-demo/spec.md");
        let tasks_path = tmp.path().join("specs/010-demo/tasks.md");
        let base_tasks = "# Tasks\n\n## 1. Bootstrap\n\n- [ ] start\n";

        write(&spec_path, &spec("planned"));
        write(&tasks_path, base_tasks);
        let m1 = commit_all(&repo, "feat: plan");

        write(&spec_path, &spec("in-progress"));
        let m2 = commit_all(&repo, "chore: begin"); // the real transition

        write(
            &tasks_path,
            "# Tasks v1\n\n## 1. Bootstrap\n\n- [ ] still\n",
        );
        commit_all(&repo, "wip: pass 1");
        write(
            &tasks_path,
            "# Tasks v2\n\n## 1. Bootstrap\n\n- [ ] still\n",
        );
        let m4 = commit_all(&repo, "wip: pass 2");

        // Side branch off m1: spec still planned, tasks untouched, one
        // unrelated file. Topologically unordered w.r.t. m2..m4.
        write(&spec_path, &spec("planned"));
        write(&tasks_path, base_tasks);
        write(&tmp.path().join("side.md"), "side\n");
        let s1 = commit_with_parents(&repo, "side: unrelated", &[m1], false);

        // Merge keeps main's in-progress spec and tasks.
        write(&spec_path, &spec("in-progress"));
        write(
            &tasks_path,
            "# Tasks v2\n\n## 1. Bootstrap\n\n- [ ] still\n",
        );
        commit_with_parents(&repo, "merge side", &[m4, s1], true);

        write(
            &tasks_path,
            "# Tasks v3\n\n## 1. Bootstrap\n\n- [ ] still\n",
        );
        commit_all(&repo, "wip: pass 3");

        let result = run(
            &CheckStuckArgs {
                feature: "010-demo".into(),
                threshold: 99,
            },
            tmp.path(),
        )
        .unwrap();
        assert_eq!(
            result.since_sha,
            m2.to_string(),
            "since must be the real main-line transition, not the merge"
        );
        // Three tasks.md commits since m2 (side commit restored tasks →
        // TREESAME; merge TREESAME to first parent).
        assert_eq!(result.commit_count, 3);
        assert!(!result.stuck);
    }

    /// CRLF close fence: a spec checked out with CRLF line endings has a
    /// `\r\n---\r\n` close fence, which the hand-rolled splitter missed
    /// (it only knew `\n---\n`), making every transition invisible. The
    /// shared CRLF-aware helper reads it.
    #[test]
    fn crlf_spec_frontmatter_transition_is_detected() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        let spec_path = tmp.path().join("specs/010-demo/spec.md");
        let tasks_path = tmp.path().join("specs/010-demo/tasks.md");

        write(&spec_path, &spec_crlf("planned"));
        write(&tasks_path, "# Tasks\n\n## 1. Bootstrap\n\n- [ ] start\n");
        commit_all(&repo, "feat: plan");

        write(&spec_path, &spec_crlf("in-progress"));
        commit_all(&repo, "chore: begin");

        write(
            &tasks_path,
            "# Tasks v1\n\n## 1. Bootstrap\n\n- [ ] still\n",
        );
        commit_all(&repo, "wip: pass 1");

        let result = run(
            &CheckStuckArgs {
                feature: "010-demo".into(),
                threshold: 1,
            },
            tmp.path(),
        )
        .unwrap();
        assert!(
            !result.since_sha.is_empty(),
            "CRLF close fence must not hide the transition"
        );
        assert_eq!(result.commit_count, 1);
        assert!(result.stuck);
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
