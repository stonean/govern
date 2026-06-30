//! `set-status` — update the `status:` field in spec frontmatter.

use std::path::Path;

use crate::primitives::{
    PrimitiveError, Result, read_text, rel_path, split_frontmatter, write_atomic,
};
use crate::schema::paths;
use crate::schema::primitives::{SetStatusArgs, SetStatusResult};

/// Execute the `set-status` primitive.
///
/// Refuses the write when the caller's `from` value does not match the
/// current on-disk status. The atomic create-then-rename pattern leaves
/// `spec.md` unchanged on a crash mid-write.
///
/// # Errors
///
/// Returns [`PrimitiveError::FeatureNotFound`] when the feature directory
/// is missing, [`PrimitiveError::MissingFrontmatter`] when the spec lacks
/// `---` fences, [`PrimitiveError::StatusFieldMissing`] when no `status:`
/// key is present, [`PrimitiveError::StatusMismatch`] when `args.from`
/// does not match disk, or [`PrimitiveError::Io`] for filesystem failures.
pub fn run(args: &SetStatusArgs, repo: &Path) -> Result<SetStatusResult> {
    let feature_dir = paths::specs_dir(repo).join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            feature: args.feature.clone(),
        });
    }
    let spec_path = feature_dir.join("spec.md");
    let content = read_text(&spec_path)?;
    let (fm_text, _body) = split_frontmatter(&content, &spec_path)?;

    let (line_offset, current_value, value_range) = locate_status_field(fm_text, &args.feature)?;

    if current_value != args.from {
        return Err(PrimitiveError::StatusMismatch {
            feature: args.feature.clone(),
            expected: args.from.clone(),
            actual: current_value,
        });
    }

    let mut new_content = String::with_capacity(content.len() + args.to.len());
    let absolute_start = line_offset + value_range.start;
    let absolute_end = line_offset + value_range.end;
    new_content.push_str(&content[..absolute_start]);
    new_content.push_str(&args.to);
    new_content.push_str(&content[absolute_end..]);

    if args.to != args.from {
        write_atomic(&spec_path, &new_content)?;
    }

    Ok(SetStatusResult {
        previous: args.from.clone(),
        current: args.to.clone(),
        path: rel_path(&spec_path, repo),
    })
}

/// Find the `status:` line inside the frontmatter text. Returns
/// `(byte_offset_of_fm_inside_full_content, current_value, value_range_within_fm)`.
fn locate_status_field(
    fm_text: &str,
    feature: &str,
) -> Result<(usize, String, std::ops::Range<usize>)> {
    let fm_start_in_full = "---\n".len();
    let mut cursor: usize = 0;
    for line in fm_text.split_inclusive('\n') {
        let line_start = cursor;
        cursor += line.len();
        let stripped = line.trim_end_matches(['\n', '\r']);
        let Some(rest) = stripped.strip_prefix("status:") else {
            continue;
        };
        let leading_ws_in_value = rest.len() - rest.trim_start().len();
        let value_with_trailing = &rest[leading_ws_in_value..];
        let value = value_with_trailing.trim_end();
        let value_start = line_start + "status:".len() + leading_ws_in_value;
        let value_end = value_start + value.len();
        return Ok((fm_start_in_full, value.to_string(), value_start..value_end));
    }
    Err(PrimitiveError::StatusFieldMissing {
        feature: feature.into(),
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn write_spec(tmp: &std::path::Path, status: &str) {
        let feature_dir = tmp.join("specs/feat");
        fs::create_dir_all(&feature_dir).unwrap();
        let body = format!("---\nstatus: {status}\ndependencies: []\n---\n\n# feat\n\nbody.\n");
        fs::write(feature_dir.join("spec.md"), body).unwrap();
    }

    #[test]
    fn advances_status_atomically() {
        let tmp = tempdir().unwrap();
        write_spec(tmp.path(), "clarified");
        let result = run(
            &SetStatusArgs {
                feature: "feat".into(),
                from: "clarified".into(),
                to: "planned".into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.previous, "clarified");
        assert_eq!(result.current, "planned");
        let content = fs::read_to_string(tmp.path().join("specs/feat/spec.md")).unwrap();
        assert!(content.contains("status: planned"));
        assert!(!content.contains("status: clarified"));
    }

    #[test]
    fn rejects_when_from_does_not_match() {
        let tmp = tempdir().unwrap();
        write_spec(tmp.path(), "in-progress");
        let err = run(
            &SetStatusArgs {
                feature: "feat".into(),
                from: "planned".into(),
                to: "done".into(),
            },
            tmp.path(),
        )
        .unwrap_err();
        match err {
            PrimitiveError::StatusMismatch {
                expected, actual, ..
            } => {
                assert_eq!(expected, "planned");
                assert_eq!(actual, "in-progress");
            }
            other => panic!("expected StatusMismatch, got {other:?}"),
        }
        // And disk is unchanged.
        let content = fs::read_to_string(tmp.path().join("specs/feat/spec.md")).unwrap();
        assert!(content.contains("status: in-progress"));
    }

    #[test]
    fn no_op_when_from_equals_to_skips_write() {
        let tmp = tempdir().unwrap();
        write_spec(tmp.path(), "planned");
        let spec_path = tmp.path().join("specs/feat/spec.md");
        let mtime_before = fs::metadata(&spec_path).unwrap().modified().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(15));
        let result = run(
            &SetStatusArgs {
                feature: "feat".into(),
                from: "planned".into(),
                to: "planned".into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.previous, "planned");
        assert_eq!(result.current, "planned");
        let mtime_after = fs::metadata(&spec_path).unwrap().modified().unwrap();
        assert_eq!(mtime_before, mtime_after);
    }

    #[test]
    fn missing_status_field_errors() {
        let tmp = tempdir().unwrap();
        let feature_dir = tmp.path().join("specs/feat");
        fs::create_dir_all(&feature_dir).unwrap();
        let body = "---\ndependencies: []\n---\n\n# feat\n";
        fs::write(feature_dir.join("spec.md"), body).unwrap();
        let err = run(
            &SetStatusArgs {
                feature: "feat".into(),
                from: "draft".into(),
                to: "clarified".into(),
            },
            tmp.path(),
        )
        .unwrap_err();
        assert!(matches!(err, PrimitiveError::StatusFieldMissing { .. }));
    }

    #[test]
    fn dropping_named_tempfile_leaves_target_unchanged() {
        use std::io::Write;
        let tmp = tempdir().unwrap();
        write_spec(tmp.path(), "clarified");
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
