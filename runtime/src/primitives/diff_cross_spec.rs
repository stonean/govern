//! `diff-cross-spec` — the cross-spec impact surface for `/gov:implement`.
//!
//! The deterministic filter implement steps 7 and 12 previously re-derived
//! by hand per task (step 12's prose self-declared "no primitive owns this
//! filter yet"; spec 022, scenario coverage-expansion-primitives): the
//! diff from the feature's first spec-dir commit — the same base
//! `derive-boundary` computes, through the shared
//! [`first_commit_for_prefix`] walk — scoped to the spec root and filtered
//! to paths outside the feature's own directory, plus the lines added to
//! `{specs-root}/inbox.md` in the same window (§brownfield-inbox capture).
//!
//! The diff runs against the working tree (index and untracked files
//! included), not `HEAD`: the per-task summary (step 7) fires before the
//! task's commit, when the run's inbox captures and any sibling-spec edits
//! are still uncommitted. On a clean tree the result equals the
//! documented `git diff <first-commit>..HEAD -- {specs-root}/` form
//! (step 12). Read-only.
//!
//! `/gov:review`'s captured-issues section stays on
//! `compute-review-scope`, whose window starts at the in-progress
//! transition — the review wants the current work window, not the
//! feature's whole history.

use std::collections::BTreeSet;
use std::path::Path;

use git2::{DiffLineType, DiffOptions, Repository};

use crate::primitives::derive_boundary::first_commit_for_prefix;
use crate::primitives::{PrimitiveError, Result, bullet_text};
use crate::schema::paths;
use crate::schema::primitives::{DiffCrossSpecArgs, DiffCrossSpecResult};

/// Execute the `diff-cross-spec` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeatureNotFound`] when the feature directory
/// is absent, [`PrimitiveError::NoSpecHistory`] when no commit touches the
/// spec dir, and [`PrimitiveError::Git`] for any libgit2 failure.
pub fn run(args: &DiffCrossSpecArgs, repo: &Path) -> Result<DiffCrossSpecResult> {
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
    let root_prefix = format!("{}/", layout.specs_root);
    let spec_prefix = format!("{}/{}/", layout.specs_root, args.feature);
    let inbox_rel = format!("{}/inbox.md", layout.specs_root);

    let first_commit = first_commit_for_prefix(&repository, &spec_prefix)?.ok_or_else(|| {
        PrimitiveError::NoSpecHistory {
            root: layout.specs_root.clone(),
            feature: args.feature.clone(),
        }
    })?;
    let head_oid = repository.head()?.peel_to_commit()?.id();
    let first_tree = repository.find_commit(first_commit)?.tree()?;

    // One diff, first-commit tree → working tree, scoped to the spec root.
    // Untracked files (a brand-new sibling scenario, a fresh inbox.md) must
    // surface with content, so their inbox lines count as additions.
    let mut opts = DiffOptions::new();
    opts.pathspec(&layout.specs_root)
        .include_untracked(true)
        .recurse_untracked_dirs(true)
        .show_untracked_content(true);
    let diff = repository.diff_tree_to_workdir_with_index(Some(&first_tree), Some(&mut opts))?;

    let mut cross_spec: BTreeSet<String> = BTreeSet::new();
    let mut inbox_additions: Vec<String> = Vec::new();
    diff.foreach(
        &mut |delta, _| {
            for path in [delta.old_file().path(), delta.new_file().path()]
                .into_iter()
                .flatten()
            {
                let s = path.to_string_lossy().replace('\\', "/");
                // Belt and braces over the pathspec: keep spec-root paths
                // only, drop the feature's own dir, and route the inbox to
                // its dedicated field instead.
                if s.starts_with(&root_prefix) && !s.starts_with(&spec_prefix) && s != inbox_rel {
                    cross_spec.insert(s);
                }
            }
            true
        },
        None,
        None,
        Some(&mut |delta, _hunk, line| {
            let is_inbox = delta
                .new_file()
                .path()
                .or_else(|| delta.old_file().path())
                .is_some_and(|p| p.to_string_lossy().replace('\\', "/") == inbox_rel);
            if is_inbox
                && line.origin_value() == DiffLineType::Addition
                && let Ok(text) = std::str::from_utf8(line.content())
            {
                // Keep item bullets only (shared bullet grammar): a
                // brand-new inbox file diffs with its heading and blank
                // lines as additions too, and those are structure, not
                // captured issues.
                let line_text = text.trim_end_matches(['\n', '\r']);
                if bullet_text(line_text).is_some() {
                    inbox_additions.push(line_text.to_string());
                }
            }
            true
        }),
    )?;

    Ok(DiffCrossSpecResult {
        first_commit: first_commit.to_string(),
        current_head: head_oid.to_string(),
        cross_spec_paths: cross_spec.into_iter().collect(),
        inbox_additions,
    })
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

    fn args(feature: &str) -> DiffCrossSpecArgs {
        DiffCrossSpecArgs {
            feature: feature.into(),
        }
    }

    /// A repo where 020-demo starts, then a sibling spec, the inbox, the
    /// feature's own dir, and non-spec code all change.
    fn seeded_repo() -> tempfile::TempDir {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        write(&tmp.path().join("README.md"), "# repo\n");
        write(
            &tmp.path().join("specs/007-sibling/spec.md"),
            "---\nstatus: done\n---\n\n# 007\n",
        );
        write(&tmp.path().join("specs/inbox.md"), "# Inbox\n");
        commit_all(&repo, "chore: init");
        write(
            &tmp.path().join("specs/020-demo/spec.md"),
            "---\nstatus: planned\n---\n\n# 020\n",
        );
        commit_all(&repo, "feat(020): plan");
        // Post-first-commit changes: the feature's own dir, a sibling
        // spec, an inbox capture, and non-spec code.
        write(
            &tmp.path().join("specs/020-demo/tasks.md"),
            "# Tasks\n\n- [x] 1\n",
        );
        write(
            &tmp.path().join("specs/007-sibling/spec.md"),
            "---\nstatus: done\n---\n\n# 007\n\nNew criterion.\n",
        );
        write(
            &tmp.path().join("specs/inbox.md"),
            "# Inbox\n\n- security: token logged in plaintext\n",
        );
        write(&tmp.path().join("runtime/src/main.rs"), "fn main() {}\n");
        commit_all(&repo, "feat(020): implement");
        tmp
    }

    #[test]
    fn reports_sibling_changes_and_inbox_additions_only() {
        let tmp = seeded_repo();
        let result = run(&args("020-demo"), tmp.path()).unwrap();
        assert_eq!(
            result.cross_spec_paths,
            vec!["specs/007-sibling/spec.md".to_string()],
            "own feature dir, inbox, and non-spec code are all excluded"
        );
        assert_eq!(
            result.inbox_additions,
            vec!["- security: token logged in plaintext".to_string()],
            "only the added inbox lines report (heading unchanged)"
        );
        assert!(!result.first_commit.is_empty());
        assert!(!result.current_head.is_empty());
    }

    #[test]
    fn clean_window_reports_empty_lists() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        write(
            &tmp.path().join("specs/020-demo/spec.md"),
            "---\nstatus: planned\n---\n\n# 020\n",
        );
        commit_all(&repo, "feat(020): plan");
        let result = run(&args("020-demo"), tmp.path()).unwrap();
        assert!(result.cross_spec_paths.is_empty());
        assert!(result.inbox_additions.is_empty());
    }

    #[test]
    fn uncommitted_working_tree_changes_surface() {
        // Step 7 fires before the task's commit: an untracked sibling
        // scenario and an uncommitted inbox capture must both surface.
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        write(
            &tmp.path().join("specs/020-demo/spec.md"),
            "---\nstatus: in-progress\n---\n\n# 020\n",
        );
        write(&tmp.path().join("specs/inbox.md"), "# Inbox\n");
        commit_all(&repo, "feat(020): begin");
        // Uncommitted: new sibling scenario file + inbox append.
        write(
            &tmp.path().join("specs/007-sibling/scenarios/edge.md"),
            "---\nsection: \"Core\"\n---\n\n# Edge\n",
        );
        write(
            &tmp.path().join("specs/inbox.md"),
            "# Inbox\n\n- leak: connection pool never drained\n",
        );
        let result = run(&args("020-demo"), tmp.path()).unwrap();
        assert_eq!(
            result.cross_spec_paths,
            vec!["specs/007-sibling/scenarios/edge.md".to_string()]
        );
        assert_eq!(
            result.inbox_additions,
            vec!["- leak: connection pool never drained".to_string()]
        );
    }

    #[test]
    fn honors_configured_specs_root() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = Repository::init(tmp.path()).unwrap();
        write(
            &tmp.path().join(".govern.toml"),
            "[paths]\nspecs-root = \"governance\"\n",
        );
        write(
            &tmp.path().join("governance/020-demo/spec.md"),
            "---\nstatus: planned\n---\n\n# 020\n",
        );
        commit_all(&repo, "feat(020): plan");
        write(
            &tmp.path().join("governance/007-sib/spec.md"),
            "---\nstatus: draft\n---\n\n# 007\n",
        );
        write(
            &tmp.path().join("governance/inbox.md"),
            "# Inbox\n\n- captured under the custom root\n",
        );
        commit_all(&repo, "feat(020): work");
        let result = run(&args("020-demo"), tmp.path()).unwrap();
        assert_eq!(
            result.cross_spec_paths,
            vec!["governance/007-sib/spec.md".to_string()]
        );
        assert_eq!(
            result.inbox_additions,
            vec!["- captured under the custom root".to_string()]
        );
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
        let err = run(&args("030-orphan"), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::NoSpecHistory { .. }));
    }

    #[test]
    fn missing_feature_and_traversal_error() {
        let tmp = seeded_repo();
        let err = run(&args("099-absent"), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::FeatureNotFound { .. }));
        let err = run(&args("../escape"), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidPath { .. }));
    }
}
