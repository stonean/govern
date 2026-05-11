//! `mark-criterion` — flip a single acceptance-criterion checkbox in `spec.md`.

use std::path::Path;

use crate::primitives::{
    PrimitiveError, Result, parse_atx_heading, read_text, rel_path, write_atomic,
};
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
    let feature_dir = repo.join("specs").join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            feature: args.feature.clone(),
        });
    }
    let spec_path = feature_dir.join("spec.md");
    let content = read_text(&spec_path)?;
    let lines: Vec<&str> = content.split_inclusive('\n').collect();

    let section_range = locate_acceptance_range(&lines);
    let checkbox_lines = collect_checkbox_line_indices(&lines, section_range);
    let (line_idx, marker_idx) = *checkbox_lines.get(args.criterion_index).ok_or_else(|| {
        PrimitiveError::CriterionOutOfRange {
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

fn locate_acceptance_range(lines: &[&str]) -> std::ops::Range<usize> {
    let mut start: Option<usize> = None;
    let mut section_level: u8 = 0;
    for (idx, line) in lines.iter().enumerate() {
        let Some((level, heading)) = parse_atx_heading(line) else {
            continue;
        };
        if let Some(s) = start {
            if level <= section_level {
                return s..idx;
            }
        } else if heading == ACCEPTANCE_HEADING {
            start = Some(idx);
            section_level = level;
        }
    }
    start.map_or(0..0, |s| s..lines.len())
}

fn collect_checkbox_line_indices(
    lines: &[&str],
    range: std::ops::Range<usize>,
) -> Vec<(usize, usize)> {
    let mut out = Vec::new();
    for idx in range {
        if let Some((_bracket, marker_idx)) = find_checkbox_line(lines[idx]) {
            out.push((idx, marker_idx));
        }
    }
    out
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
