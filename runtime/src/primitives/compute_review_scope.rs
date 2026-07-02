//! `compute-review-scope` — deterministic scope resolution for `/gov:review`.
//!
//! Resolves three things the review command needs, from git history:
//!
//! - **diff-base** — the commit the spec advanced to `in-progress` at (shared
//!   with `check-stuck`), or a caller-supplied `--since` override.
//! - **scope** — the larger of the plan's `Affected Files` set and the set of
//!   files modified since `diff-base` (review walks the bigger surface).
//! - **captured-issues** — lines added to `{specs-root}/inbox.md` in the window
//!   (`diff-base..HEAD`), the incidental issues logged during the work.
//!
//! Read-only.
//!
//! Defined by
//! `specs/022-deterministic-runtime/scenarios/review-runtime-acceleration.md`.

use std::collections::BTreeSet;
use std::path::Path;

use git2::{DiffLineType, DiffOptions, Oid, Repository};

use crate::primitives::check_stuck::find_in_progress_commit;
use crate::primitives::{PrimitiveError, Result, section_lines};
use crate::schema::paths;
use crate::schema::primitives::{ComputeReviewScopeArgs, ComputeReviewScopeResult};

/// Execute the `compute-review-scope` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeatureNotFound`] when the feature directory is
/// absent, or [`PrimitiveError::Git`] for any libgit2 failure (repo discovery,
/// revparse of `--since`, tree/diff lookup).
pub fn run(args: &ComputeReviewScopeArgs, repo: &Path) -> Result<ComputeReviewScopeResult> {
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

    let diff_base = match &args.since {
        Some(reference) => repository
            .revparse_single(reference)?
            .peel_to_commit()?
            .id()
            .to_string(),
        None => find_in_progress_commit(&repository, &spec_rel)?.unwrap_or_default(),
    };

    let inbox_rel = format!("{}/inbox.md", layout.specs_root);
    let (modified_since, captured_issues) = if diff_base.is_empty() {
        (Vec::new(), Vec::new())
    } else {
        diff_since(&repository, &diff_base, &inbox_rel)?
    };

    let plan_affected = read_plan_affected(&feature_dir);

    // "Whichever set is larger" — review walks the bigger surface. On a tie,
    // prefer the git-derived modified-since set (authoritative for what the
    // work actually touched).
    let scope = if plan_affected.len() > modified_since.len() {
        plan_affected.clone()
    } else {
        modified_since.clone()
    };

    Ok(ComputeReviewScopeResult {
        diff_base,
        scope,
        modified_since,
        plan_affected,
        captured_issues,
    })
}

/// Diff `base_sha..HEAD`: return the sorted set of changed file paths and the
/// lines added to `inbox_rel` in that window.
fn diff_since(
    repo: &Repository,
    base_sha: &str,
    inbox_rel: &str,
) -> Result<(Vec<String>, Vec<String>)> {
    let base_tree = repo.find_commit(Oid::from_str(base_sha)?)?.tree()?;
    let head_tree = repo.head()?.peel_to_commit()?.tree()?;

    // Every changed file path (added / modified / deleted / renamed).
    let diff = repo.diff_tree_to_tree(Some(&base_tree), Some(&head_tree), None)?;
    let mut files: BTreeSet<String> = BTreeSet::new();
    diff.foreach(
        &mut |delta, _| {
            let path = delta.new_file().path().or_else(|| delta.old_file().path());
            if let Some(path) = path {
                files.insert(path.to_string_lossy().into_owned());
            }
            true
        },
        None,
        None,
        None,
    )?;

    // Added lines in the inbox, scoped by pathspec.
    let mut opts = DiffOptions::new();
    opts.pathspec(inbox_rel);
    let inbox_diff = repo.diff_tree_to_tree(Some(&base_tree), Some(&head_tree), Some(&mut opts))?;
    let mut added: Vec<String> = Vec::new();
    inbox_diff.foreach(
        &mut |_, _| true,
        None,
        None,
        Some(&mut |_delta, _hunk, line| {
            if line.origin_value() == DiffLineType::Addition
                && let Ok(text) = std::str::from_utf8(line.content())
            {
                added.push(text.trim_end_matches(['\n', '\r']).to_string());
            }
            true
        }),
    )?;

    Ok((files.into_iter().collect(), added))
}

/// Parse a feature's `plan.md` `## Affected Files` section into a list of file
/// paths. Each list item's first token (backticks stripped) is the path.
/// Returns an empty list when `plan.md` is absent or has no such section.
fn read_plan_affected(feature_dir: &Path) -> Vec<String> {
    let Ok(content) = std::fs::read_to_string(feature_dir.join("plan.md")) else {
        return Vec::new();
    };
    let mut files = Vec::new();
    for line in section_lines(&content, "Affected Files") {
        let trimmed = line.trim_start();
        let Some(rest) = trimmed.strip_prefix("- ") else {
            continue;
        };
        let token = rest
            .trim()
            .trim_start_matches('`')
            .split(['`', ' '])
            .next()
            .unwrap_or("")
            .trim();
        if !token.is_empty() {
            files.push(token.to_string());
        }
    }
    files
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use git2::{IndexAddOption, Repository, Signature};
    use std::fs;

    /// Stage everything and commit; returns the new commit's sha.
    fn commit_all(repo: &Repository, message: &str) -> String {
        let mut index = repo.index().unwrap();
        index.add_all(["*"], IndexAddOption::DEFAULT, None).unwrap();
        index.write().unwrap();
        let tree = repo.find_tree(index.write_tree().unwrap()).unwrap();
        let sig = Signature::now("Test", "test@example.com").unwrap();
        let parent = repo
            .head()
            .ok()
            .and_then(|h| h.target())
            .and_then(|oid| repo.find_commit(oid).ok());
        let parents: Vec<&git2::Commit> = parent.as_ref().into_iter().collect();
        repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &parents)
            .unwrap()
            .to_string()
    }

    fn write(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    fn spec(status: &str) -> String {
        format!("---\nstatus: {status}\ndependencies: []\n---\n\n# X\n")
    }

    fn args(feature: &str, since: Option<&str>) -> ComputeReviewScopeArgs {
        ComputeReviewScopeArgs {
            feature: feature.to_string(),
            since: since.map(str::to_string),
        }
    }

    /// A repo where 001-x goes planned → in-progress, then two source files
    /// are added after the transition. Returns (tempdir, in-progress sha).
    fn repo_with_progress() -> (tempfile::TempDir, String) {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        let spec_path = tmp.path().join("specs/001-x/spec.md");
        write(&spec_path, &spec("planned"));
        commit_all(&repo, "feat: plan");
        write(&spec_path, &spec("in-progress"));
        let sha = commit_all(&repo, "chore: begin");
        write(&tmp.path().join("src/a.rs"), "fn a() {}\n");
        write(&tmp.path().join("src/b.rs"), "fn b() {}\n");
        commit_all(&repo, "feat: implement");
        (tmp, sha)
    }

    #[test]
    fn diff_base_is_the_in_progress_commit_and_scope_lists_modified_files() {
        let (tmp, sha) = repo_with_progress();
        let result = run(&args("001-x", None), tmp.path()).unwrap();
        assert_eq!(result.diff_base, sha);
        assert!(result.modified_since.contains(&"src/a.rs".to_string()));
        assert!(result.modified_since.contains(&"src/b.rs".to_string()));
        assert_eq!(result.scope, result.modified_since);
    }

    #[test]
    fn since_override_replaces_the_derived_base() {
        let (tmp, _sha) = repo_with_progress();
        // Override to HEAD → no files modified since HEAD.
        let result = run(&args("001-x", Some("HEAD")), tmp.path()).unwrap();
        let head = Repository::discover(tmp.path())
            .unwrap()
            .head()
            .unwrap()
            .peel_to_commit()
            .unwrap()
            .id()
            .to_string();
        assert_eq!(result.diff_base, head);
        assert!(result.modified_since.is_empty());
    }

    #[test]
    fn plan_affected_wins_when_it_is_the_larger_set() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        let spec_path = tmp.path().join("specs/001-x/spec.md");
        write(&spec_path, &spec("planned"));
        commit_all(&repo, "feat: plan");
        write(&spec_path, &spec("in-progress"));
        commit_all(&repo, "chore: begin");
        // Only one file modified since; plan lists three.
        write(&tmp.path().join("src/only.rs"), "fn only() {}\n");
        write(
            &tmp.path().join("specs/001-x/plan.md"),
            "# Plan\n\n## Affected Files\n\n- `src/one.rs`\n- `src/two.rs` — the second\n- `src/three.rs`\n",
        );
        commit_all(&repo, "feat: implement");
        let result = run(&args("001-x", None), tmp.path()).unwrap();
        assert_eq!(
            result.plan_affected,
            vec!["src/one.rs", "src/two.rs", "src/three.rs"]
        );
        // plan_affected (3) > modified_since (2: only.rs + plan.md) → plan wins.
        assert_eq!(result.scope, result.plan_affected);
    }

    #[test]
    fn captured_issues_lists_inbox_additions_in_the_window() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        let spec_path = tmp.path().join("specs/001-x/spec.md");
        let inbox = tmp.path().join("specs/inbox.md");
        write(&spec_path, &spec("planned"));
        write(&inbox, "# Inbox\n\n- pre-existing item\n");
        commit_all(&repo, "feat: plan");
        write(&spec_path, &spec("in-progress"));
        commit_all(&repo, "chore: begin");
        // Append two issues to the inbox during the work window.
        write(
            &inbox,
            "# Inbox\n\n- pre-existing item\n- captured: leak in a.rs\n- captured: missing check in b.rs\n",
        );
        commit_all(&repo, "chore: capture issues");
        let result = run(&args("001-x", None), tmp.path()).unwrap();
        assert!(
            result
                .captured_issues
                .contains(&"- captured: leak in a.rs".to_string())
        );
        assert!(
            result
                .captured_issues
                .contains(&"- captured: missing check in b.rs".to_string())
        );
        // The pre-existing item predates diff-base → not captured.
        assert!(
            !result
                .captured_issues
                .contains(&"- pre-existing item".to_string())
        );
    }

    #[test]
    fn no_in_progress_and_no_since_yields_empty_scope() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        write(&tmp.path().join("specs/001-x/spec.md"), &spec("planned"));
        commit_all(&repo, "feat: plan");
        let result = run(&args("001-x", None), tmp.path()).unwrap();
        assert!(result.diff_base.is_empty());
        assert!(result.scope.is_empty());
        assert!(result.modified_since.is_empty());
        assert!(result.captured_issues.is_empty());
    }

    #[test]
    fn missing_feature_is_operational_error() {
        let tmp = tempfile::tempdir().unwrap();
        Repository::init(tmp.path()).unwrap();
        let err = run(&args("999-nope", None), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::FeatureNotFound { .. }));
    }

    #[test]
    fn plan_absent_falls_back_to_modified_since() {
        let (tmp, _sha) = repo_with_progress();
        // No plan.md written → plan_affected empty → scope = modified_since.
        let result = run(&args("001-x", None), tmp.path()).unwrap();
        assert!(result.plan_affected.is_empty());
        assert_eq!(result.scope, result.modified_since);
        assert!(!result.scope.is_empty());
    }
}
