//! `mark-criterion` — flip a single acceptance-criterion checkbox in `spec.md`.

use std::path::Path;

use crate::primitives::{
    PrimitiveError, Result, read_text, rel_path, section_line_indices, write_atomic,
};
use crate::schema::paths;
use crate::schema::primitives::{CheckboxToggleResult, MarkCriterionArgs};

use super::checkbox::{find_checkbox_line, flip_checkbox_at};

const ACCEPTANCE_HEADING: &str = "Acceptance Criteria";

/// Execute the `mark-criterion` primitive.
///
/// Locates the acceptance criterion at `args.criterion_index` (0-based,
/// ordered as in the spec body) and flips its checkbox to `args.checked`.
/// Uses the same atomic create-then-rename write pattern as `mark-task`.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeatureNotFound`] when the feature directory is
/// missing, [`PrimitiveError::CriterionOutOfRange`] when the index exceeds
/// the number of criteria, or [`PrimitiveError::Io`] for filesystem
/// failures.
pub fn run(args: &MarkCriterionArgs, repo: &Path) -> Result<CheckboxToggleResult> {
    super::validate_no_traversal(&args.feature)?;
    let root = paths::Paths::load(repo).specs_root;
    let feature_dir = repo.join(&root).join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            root: root.clone(),
            feature: args.feature.clone(),
        });
    }
    let spec_path = feature_dir.join("spec.md");
    let content = read_text(&spec_path)?;
    let lines: Vec<&str> = content.split_inclusive('\n').collect();

    let checkbox_lines = collect_checkbox_line_indices(&lines);
    let (line_idx, marker_idx) = *checkbox_lines.get(args.criterion_index).ok_or_else(|| {
        PrimitiveError::CriterionOutOfRange {
            root: root.clone(),
            feature: args.feature.clone(),
            criterion_index: args.criterion_index,
            total: checkbox_lines.len(),
        }
    })?;

    let (previous, new_line) = flip_checkbox_at(lines[line_idx], marker_idx, args.checked);
    let mut new_content = String::new();
    for (idx, line) in lines.iter().enumerate() {
        if idx == line_idx {
            new_content.push_str(&new_line);
        } else {
            new_content.push_str(line);
        }
    }

    if previous != args.checked {
        write_atomic(&spec_path, &new_content)?;
    }

    Ok(CheckboxToggleResult {
        previous,
        current: args.checked,
        path: rel_path(&spec_path, repo),
    })
}

/// Collect `(line_index, marker_index)` for every addressable checkbox in
/// the Acceptance Criteria section. Uses the shared comment/fence-aware
/// walker ([`section_line_indices`]) — the same walker `read-spec`'s
/// criteria listing consumes — so criterion index N here is exactly the
/// criterion `read-spec` reports at index N, and a checkbox embedded in a
/// template guidance comment or fenced code block is never flippable.
fn collect_checkbox_line_indices(lines: &[&str]) -> Vec<(usize, usize)> {
    section_line_indices(lines, ACCEPTANCE_HEADING)
        .into_iter()
        .filter_map(|idx| {
            find_checkbox_line(lines[idx]).map(|(_bracket, marker_idx)| (idx, marker_idx))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;
    use tempfile::tempdir;

    const SPEC: &str = "---\nstatus: in-progress\ndependencies: []\n---\n\n# feat\n\n## Acceptance Criteria\n\n- [ ] First criterion.\n- [x] Second criterion (pre-checked).\n- [ ] Third criterion.\n\n## Non-Goals\n\n- [ ] Not a criterion — outside the section.\n";

    fn write_fixture(tmp: &std::path::Path) {
        let feature_dir = tmp.join("specs/feat");
        fs::create_dir_all(&feature_dir).unwrap();
        fs::write(feature_dir.join("spec.md"), SPEC).unwrap();
    }

    #[test]
    fn flips_first_criterion() {
        let tmp = tempdir().unwrap();
        write_fixture(tmp.path());
        let result = run(
            &MarkCriterionArgs {
                feature: "feat".into(),
                criterion_index: 0,
                checked: true,
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.previous);
        assert!(result.current);
        assert_eq!(result.path, "specs/feat/spec.md");
        let new_content = fs::read_to_string(tmp.path().join("specs/feat/spec.md")).unwrap();
        assert!(new_content.contains("- [x] First criterion."));
        assert!(new_content.contains("- [x] Second criterion (pre-checked)."));
        assert!(new_content.contains("- [ ] Third criterion."));
        assert!(new_content.contains("- [ ] Not a criterion"));
    }

    #[test]
    fn unchecks_second_criterion() {
        let tmp = tempdir().unwrap();
        write_fixture(tmp.path());
        let result = run(
            &MarkCriterionArgs {
                feature: "feat".into(),
                criterion_index: 1,
                checked: false,
            },
            tmp.path(),
        )
        .unwrap();
        assert!(result.previous);
        assert!(!result.current);
        let new_content = fs::read_to_string(tmp.path().join("specs/feat/spec.md")).unwrap();
        assert!(new_content.contains("- [ ] Second criterion (pre-checked)."));
    }

    #[test]
    fn out_of_range_index_errors() {
        let tmp = tempdir().unwrap();
        write_fixture(tmp.path());
        let err = run(
            &MarkCriterionArgs {
                feature: "feat".into(),
                criterion_index: 99,
                checked: true,
            },
            tmp.path(),
        )
        .unwrap_err();
        match err {
            PrimitiveError::CriterionOutOfRange { total, .. } => assert_eq!(total, 3),
            other => panic!("expected CriterionOutOfRange, got {other:?}"),
        }
    }

    #[test]
    fn template_state_spec_has_no_flippable_criteria() {
        // The shipped spec template embeds an example `- [ ]` checkbox
        // inside the Acceptance Criteria guidance comment; it must not be
        // addressable (scenario spec-side-parser-hardening).
        let repo_root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .to_path_buf();
        let template =
            fs::read_to_string(repo_root.join("framework/templates/spec/spec.md")).unwrap();
        let tmp = tempdir().unwrap();
        let feature_dir = tmp.path().join("specs/feat");
        fs::create_dir_all(&feature_dir).unwrap();
        fs::write(feature_dir.join("spec.md"), &template).unwrap();

        let err = run(
            &MarkCriterionArgs {
                feature: "feat".into(),
                criterion_index: 0,
                checked: true,
            },
            tmp.path(),
        )
        .unwrap_err();
        match err {
            PrimitiveError::CriterionOutOfRange { total, .. } => assert_eq!(total, 0),
            other => panic!("expected CriterionOutOfRange, got {other:?}"),
        }
        // The comment-embedded example checkbox is untouched.
        let on_disk = fs::read_to_string(tmp.path().join("specs/feat/spec.md")).unwrap();
        assert_eq!(on_disk, template);
    }

    #[test]
    fn comment_embedded_checkbox_does_not_shift_indexes() {
        // A guidance-comment checkbox ahead of the real criteria must be
        // invisible to index addressing: index 0 is the first REAL checkbox.
        let tmp = tempdir().unwrap();
        let feature_dir = tmp.path().join("specs/feat");
        fs::create_dir_all(&feature_dir).unwrap();
        let spec = "---\nstatus: in-progress\ndependencies: []\n---\n\n# feat\n\n\
                    ## Acceptance Criteria\n\n\
                    <!--\n- [ ] Example inside comment\n-->\n\n\
                    - [ ] Real criterion.\n";
        fs::write(feature_dir.join("spec.md"), spec).unwrap();

        let result = run(
            &MarkCriterionArgs {
                feature: "feat".into(),
                criterion_index: 0,
                checked: true,
            },
            tmp.path(),
        )
        .unwrap();
        assert!(!result.previous);
        let new_content = fs::read_to_string(tmp.path().join("specs/feat/spec.md")).unwrap();
        assert!(new_content.contains("- [ ] Example inside comment"));
        assert!(new_content.contains("- [x] Real criterion."));
    }

    #[test]
    fn dropping_named_tempfile_leaves_target_unchanged() {
        use std::io::Write;
        let tmp = tempdir().unwrap();
        write_fixture(tmp.path());
        let spec_path = tmp.path().join("specs/feat/spec.md");
        let original = fs::read_to_string(&spec_path).unwrap();
        {
            let parent = spec_path.parent().unwrap();
            let mut tf = tempfile::NamedTempFile::new_in(parent).unwrap();
            tf.write_all(b"INTERRUPTED CONTENT").unwrap();
        }
        assert_eq!(original, fs::read_to_string(&spec_path).unwrap());
    }
}
