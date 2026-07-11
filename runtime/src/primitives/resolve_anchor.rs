//! `resolve-anchor` — verify every `§<anchor>` reference resolves to a
//! `<!-- §anchor -->` marker within the same file.

#![allow(clippy::expect_used)]

use std::collections::HashSet;
use std::path::Path;
use std::sync::OnceLock;

use regex::Regex;

use crate::primitives::{Result, read_text, resolve_path};
use crate::schema::primitives::{AnchorReference, ResolveAnchorArgs, ResolveAnchorResult};

/// Execute the `resolve-anchor` primitive.
///
/// # Errors
///
/// Returns [`crate::primitives::PrimitiveError::Io`] when the file cannot
/// be read.
pub fn run(args: &ResolveAnchorArgs, repo: &Path) -> Result<ResolveAnchorResult> {
    let path = resolve_path(repo, &args.path);
    let content = read_text(&path)?;

    let markers = collect_markers(&content);
    let mut references: Vec<AnchorReference> = Vec::new();
    let mut unresolved: HashSet<String> = HashSet::new();
    for (line_no, line) in content.lines().enumerate() {
        let line_no = u32::try_from(line_no + 1).unwrap_or(u32::MAX);
        for cap in reference_regex().captures_iter(line) {
            let anchor = cap[1].to_string();
            if is_within_marker_comment(line, cap.get(0).map_or(0, |m| m.start())) {
                continue;
            }
            let resolved = markers.contains(&anchor);
            if !resolved {
                unresolved.insert(anchor.clone());
            }
            references.push(AnchorReference {
                anchor,
                line: line_no,
                resolved,
            });
        }
    }

    let mut unresolved: Vec<String> = unresolved.into_iter().collect();
    unresolved.sort();
    Ok(ResolveAnchorResult {
        references,
        unresolved,
    })
}

fn collect_markers(content: &str) -> HashSet<String> {
    marker_regex()
        .captures_iter(content)
        .map(|c| c[1].to_string())
        .collect()
}

fn marker_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| {
        Regex::new(r"<!--\s*§([A-Za-z][A-Za-z0-9_-]*)\s*-->").expect("hard-coded regex compiles")
    })
}

fn reference_regex() -> &'static Regex {
    static R: OnceLock<Regex> = OnceLock::new();
    R.get_or_init(|| Regex::new(r"§([A-Za-z][A-Za-z0-9_-]*)").expect("hard-coded regex compiles"))
}

fn is_within_marker_comment(line: &str, match_start: usize) -> bool {
    let before = &line[..match_start];
    let after = &line[match_start..];
    before.contains("<!--") && after.contains("-->")
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
    fn resolves_constitution_anchors() {
        let repo = fixture_repo();
        let result = run(
            &ResolveAnchorArgs {
                path: "framework/constitution.md".into(),
            },
            &repo,
        )
        .unwrap();

        let resolved_refs: Vec<&str> = result
            .references
            .iter()
            .filter(|r| r.resolved)
            .map(|r| r.anchor.as_str())
            .collect();
        assert!(resolved_refs.contains(&"runtime-boundary"));
        assert!(resolved_refs.contains(&"spec-phase"));

        assert_eq!(result.unresolved, vec!["unknown-anchor".to_string()]);
    }

    #[test]
    fn markers_are_excluded_from_references() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("only-markers.md");
        std::fs::write(&path, "# only markers\n\n<!-- §foo -->\n<!-- §bar -->\n").unwrap();
        let result = run(
            &ResolveAnchorArgs {
                path: path.to_string_lossy().into(),
            },
            tmp.path(),
        )
        .unwrap();
        assert!(result.references.is_empty());
        assert!(result.unresolved.is_empty());
    }
}
