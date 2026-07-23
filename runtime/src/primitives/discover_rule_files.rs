//! `discover-rule-files` — deterministic rule-file selection for `/gov:review`.
//!
//! Owns `/gov:review`'s rule-file selection end-to-end: list the rule-file
//! directory
//! (`framework/rules/` in govern's own repo, `{specs-root}/rules/` in
//! adopters), classify each file by basename suffix, apply the
//! `[rules] surfaces` selection, then the `[[review.disabled-rule-files]]`
//! filter — returning the selected set plus the ordered stdout notice lines
//! the command must emit verbatim.
//!
//! The `[rules] surfaces` key, when set, is authoritative (and validated with
//! the fail-fast semantics review documents). When unset, selection falls back
//! to the host-detected stack passed in `detected-surfaces` — stack detection
//! itself is a semantic step that stays with the host/LLM; the primitive only
//! consumes the resolved surfaces. When neither is supplied, every recognized
//! surface is loaded (conservative: review everything).
//!
//! Defined by
//! `specs/022-deterministic-runtime/scenarios/review-runtime-acceleration.md`.

use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::primitives::{PrimitiveError, Result, read_text};
use crate::schema::paths;
use crate::schema::primitives::{DiscoverRuleFilesArgs, DiscoverRuleFilesResult};

/// Recognized, selectable surface members for `[rules] surfaces`.
/// Cross-cutting (`-cross.md`) files are unconditional and are not a
/// selectable surface — listing `"cross"` is rejected.
const VALID_SURFACES: &[&str] = &["backend", "frontend"];

/// Minimum trimmed length (Unicode codepoints) of a disabled-rule-file
/// `reason`. Mirrors the review contract's audit-trail requirement.
const MIN_REASON_LEN: usize = 16;

/// Suffix classification of a rule file.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Surface {
    /// `*-backend.md`.
    Backend,
    /// `*-frontend.md`.
    Frontend,
    /// `*-cross.md` — loaded unconditionally.
    Cross,
    /// Any other suffix — loaded for every stack with a warning.
    Unrecognized,
}

/// Classify a rule-file basename by its suffix.
fn classify(basename: &str) -> Surface {
    if basename.ends_with("-backend.md") {
        Surface::Backend
    } else if basename.ends_with("-frontend.md") {
        Surface::Frontend
    } else if basename.ends_with("-cross.md") {
        Surface::Cross
    } else {
        Surface::Unrecognized
    }
}

/// Execute the `discover-rule-files` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::Toml`] when `.govern.toml` is malformed,
/// [`PrimitiveError::InvalidSurfacesMember`] when `[rules] surfaces` names a
/// value outside `{backend, frontend}`, [`PrimitiveError::InvalidSurfacesType`]
/// when the key is not a list of strings, or [`PrimitiveError::Io`] on
/// filesystem failures listing the rule-file directory.
pub fn run(args: &DiscoverRuleFilesArgs, repo: &Path) -> Result<DiscoverRuleFilesResult> {
    let config = load_govern_toml(repo)?;
    let (rules_dir_path, rules_dir_rel) = resolve_rules_dir(repo);

    let mut notices: Vec<String> = Vec::new();
    let mut selected: Vec<String> = Vec::new();

    if let Some(dir) = rules_dir_path {
        let all = list_rule_files(&dir)?;
        let surfaces = resolve_surfaces(config.rules.as_ref(), &args.detected_surfaces)?;

        for name in &all {
            match classify(name) {
                Surface::Backend => {
                    if has(&surfaces, "backend") {
                        selected.push(name.clone());
                    }
                }
                Surface::Frontend => {
                    if has(&surfaces, "frontend") {
                        selected.push(name.clone());
                    }
                }
                Surface::Cross => selected.push(name.clone()),
                Surface::Unrecognized => {
                    selected.push(name.clone());
                    notices.push(format!(
                        "rule file {name} has unrecognized suffix — loading for all stacks; rename to -backend.md, -frontend.md, or -cross.md"
                    ));
                }
            }
        }

        apply_disabled_filter(
            config.review.as_ref(),
            &all,
            &mut selected,
            &mut notices,
            paths::config_display_name(repo),
        );
    }

    selected.sort();
    notices.push(format!("loading rule files: {}", selected.join(", ")));

    Ok(DiscoverRuleFilesResult {
        rules_dir: rules_dir_rel,
        selected,
        notices,
    })
}

/// `true` when `surfaces` contains `member`.
fn has(surfaces: &[String], member: &str) -> bool {
    surfaces.iter().any(|s| s == member)
}

/// Locate the rule-file directory. Adopters keep rules under
/// `{specs-root}/rules/`; govern's own repo keeps them under
/// `framework/rules/`. Prefer the adopter path when it exists (that is the
/// discriminator — a govern checkout has `framework/rules/` but no
/// `specs/rules/`). Returns `(None, "")` when neither exists.
fn resolve_rules_dir(repo: &Path) -> (Option<PathBuf>, String) {
    let layout = paths::Paths::load(repo);
    let adopter = repo.join(&layout.specs_root).join("rules");
    if adopter.is_dir() {
        return (Some(adopter), format!("{}/rules", layout.specs_root));
    }
    let own = repo.join("framework").join("rules");
    if own.is_dir() {
        return (Some(own), "framework/rules".to_string());
    }
    (None, String::new())
}

/// List `*.md` basenames directly under `dir`, sorted for determinism.
fn list_rule_files(dir: &Path) -> Result<Vec<String>> {
    let mut names = Vec::new();
    let read_dir = std::fs::read_dir(dir).map_err(|source| PrimitiveError::Io {
        path: dir.into(),
        source,
    })?;
    for entry in read_dir {
        let entry = entry.map_err(|source| PrimitiveError::Io {
            path: dir.into(),
            source,
        })?;
        let file_type = entry.file_type().map_err(|source| PrimitiveError::Io {
            path: entry.path(),
            source,
        })?;
        if !file_type.is_file() {
            continue;
        }
        if entry
            .path()
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            names.push(entry.file_name().to_string_lossy().into_owned());
        }
    }
    names.sort();
    Ok(names)
}

/// Resolve the effective surfaces. Config `[rules] surfaces`, when present, is
/// authoritative (and validated); when unset, fall back to `detected`; when
/// that is also empty, load every recognized surface.
fn resolve_surfaces(rules: Option<&RulesSection>, detected: &[String]) -> Result<Vec<String>> {
    match rules.and_then(|r| r.surfaces.as_ref()) {
        Some(value) => validate_surfaces(value),
        None => {
            if detected.is_empty() {
                Ok(VALID_SURFACES.iter().map(|s| (*s).to_string()).collect())
            } else {
                // The `detected-surfaces` arg crosses the MCP boundary, so
                // validate its members the same way a `[rules] surfaces`
                // config value is validated. An unrecognized member (e.g.
                // "Backend" or "back-end") would otherwise select neither
                // surface and silently drop that surface's rule files from
                // the review set — fail fast instead.
                for member in detected {
                    validate_surface_member(member)?;
                }
                Ok(detected.to_vec())
            }
        }
    }
}

/// Reject a surface name outside `{backend, frontend}`. Shared by the
/// `[rules] surfaces` config check and the MCP `detected-surfaces` check so
/// the two agree on the accepted set.
fn validate_surface_member(member: &str) -> Result<()> {
    if VALID_SURFACES.contains(&member) {
        Ok(())
    } else {
        Err(PrimitiveError::InvalidSurfacesMember {
            value: member.to_string(),
        })
    }
}

/// Validate a `[rules] surfaces` value: it must be a list of strings, each a
/// member of `{backend, frontend}`. The empty list is valid (cross-only).
fn validate_surfaces(value: &toml::Value) -> Result<Vec<String>> {
    let toml::Value::Array(items) = value else {
        return Err(PrimitiveError::InvalidSurfacesType {
            got: toml_type_name(value).to_string(),
        });
    };
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        let toml::Value::String(member) = item else {
            return Err(PrimitiveError::InvalidSurfacesType {
                got: format!("a list containing {}", toml_type_name(item)),
            });
        };
        validate_surface_member(member)?;
        out.push(member.clone());
    }
    Ok(out)
}

/// Human-readable TOML type name for the type-mismatch diagnostic.
fn toml_type_name(value: &toml::Value) -> &'static str {
    match value {
        toml::Value::String(_) => "a string",
        toml::Value::Integer(_) => "an integer",
        toml::Value::Float(_) => "a float",
        toml::Value::Boolean(_) => "a boolean",
        toml::Value::Datetime(_) => "a datetime",
        toml::Value::Array(_) => "an array",
        toml::Value::Table(_) => "a table",
    }
}

/// Apply the `[[review.disabled-rule-files]]` filter to the post-selection
/// set, appending the ordered notice lines for each entry. Entries are
/// processed in config order; the first entry for a given `file` applies,
/// later duplicates warn. Malformed entries warn and are skipped without
/// dropping anything. `config_name` is the repo-relative resolved config
/// file the disable came from, rendered in the drop notice's provenance
/// tag (spec 042: `.govern/config.toml`, or the legacy root `.govern.toml`
/// pre-migration).
fn apply_disabled_filter(
    review: Option<&ReviewSection>,
    all: &[String],
    selected: &mut Vec<String>,
    notices: &mut Vec<String>,
    config_name: &str,
) {
    let Some(review) = review else { return };
    let mut seen: Vec<String> = Vec::new();
    for (index, entry) in review.disabled_rule_files.iter().enumerate() {
        let Some((file, reason)) = validate_disabled_entry(entry, index, notices) else {
            continue;
        };
        if seen.iter().any(|s| s == file) {
            notices.push(format!(
                "duplicate disabled-rule-file: {file} — entry [{index}] ignored"
            ));
            continue;
        }
        seen.push(file.to_string());

        if let Some(pos) = selected.iter().position(|s| s == file) {
            selected.remove(pos);
            notices.push(format!(
                "disabled-rule-file: {file} — {} ({config_name})",
                collapse_whitespace(reason)
            ));
        } else if all.iter().any(|s| s == file) {
            notices.push(format!(
                "disabled-rule-file (no-op): {file} not selected by stack detection"
            ));
        } else {
            notices.push(format!(
                "unknown disabled-rule-file: {file} (no such file in the rule-file directory)"
            ));
        }
    }
}

/// Validate one disabled-rule-file entry, returning `(file, reason)` when
/// well-formed. Emits a `malformed …` notice and returns `None` otherwise.
fn validate_disabled_entry<'a>(
    entry: &'a toml::Value,
    index: usize,
    notices: &mut Vec<String>,
) -> Option<(&'a str, &'a str)> {
    let toml::Value::Table(table) = entry else {
        notices.push(format!(
            "malformed disabled-rule-file at review.disabled-rule-files[{index}]: entry is not a table"
        ));
        return None;
    };
    let file = table.get("file").and_then(toml::Value::as_str);
    let reason = table.get("reason").and_then(toml::Value::as_str);
    match (file, reason) {
        (None, _) => {
            notices.push(format!(
                "malformed disabled-rule-file at review.disabled-rule-files[{index}]: missing 'file'"
            ));
            None
        }
        (Some(_), None) => {
            notices.push(format!(
                "malformed disabled-rule-file at review.disabled-rule-files[{index}]: missing 'reason'"
            ));
            None
        }
        (Some(file), Some(reason)) => {
            if reason.trim().chars().count() < MIN_REASON_LEN {
                notices.push(format!(
                    "malformed disabled-rule-file at review.disabled-rule-files[{index}]: reason must be at least {MIN_REASON_LEN} characters"
                ));
                None
            } else {
                Some((file, reason))
            }
        }
    }
}

/// Collapse internal whitespace (including TOML multi-line-string newlines) to
/// single spaces — the disabled-rule-file notice is single-line by contract.
fn collapse_whitespace(reason: &str) -> String {
    reason.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Parse `.govern.toml`, returning defaults when the file is absent.
fn load_govern_toml(repo: &Path) -> Result<GovernToml> {
    let path = paths::config_path(repo);
    if !path.is_file() {
        return Ok(GovernToml::default());
    }
    let content = read_text(&path)?;
    toml::from_str(&content).map_err(|source| PrimitiveError::Toml { path, source })
}

/// Minimal `.govern.toml` shape: the `[rules]` and `[review]` sections this
/// primitive consults. Unknown keys are accepted.
#[derive(Deserialize, Default)]
struct GovernToml {
    #[serde(default)]
    rules: Option<RulesSection>,
    #[serde(default)]
    review: Option<ReviewSection>,
}

/// `[rules]` — `surfaces` is kept as a raw `toml::Value` so the primitive can
/// distinguish unset (None) from an empty list, and produce the contract's
/// type/member diagnostics rather than a generic parse error.
#[derive(Deserialize, Default)]
struct RulesSection {
    #[serde(default)]
    surfaces: Option<toml::Value>,
}

/// `[review]` — the `disabled-rule-files` entries, kept as raw values so
/// malformed entries (missing `file`/`reason`, short `reason`) can be reported
/// rather than failing the whole parse.
#[derive(Deserialize, Default)]
struct ReviewSection {
    #[serde(default, rename = "disabled-rule-files")]
    disabled_rule_files: Vec<toml::Value>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use tempfile::TempDir;

    /// Build a repo with `framework/rules/{files}` and an optional
    /// `.govern.toml`. Returns the tempdir.
    fn setup(files: &[&str], govern_toml: Option<&str>) -> TempDir {
        let tmp = TempDir::new().unwrap();
        let rules = tmp.path().join("framework/rules");
        std::fs::create_dir_all(&rules).unwrap();
        for f in files {
            std::fs::write(rules.join(f), "# rule\n").unwrap();
        }
        if let Some(body) = govern_toml {
            std::fs::write(tmp.path().join(".govern.toml"), body).unwrap();
        }
        tmp
    }

    fn args(detected: &[&str]) -> DiscoverRuleFilesArgs {
        DiscoverRuleFilesArgs {
            detected_surfaces: detected.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    const THREE: &[&str] = &[
        "security-backend.md",
        "accessibility-frontend.md",
        "quality-cross.md",
    ];

    #[test]
    fn rejects_unrecognized_detected_surface_member() {
        // The MCP-boundary `detected-surfaces` arg is validated like a config
        // value: an unrecognized member fails fast rather than silently
        // selecting neither surface and dropping that surface's rule files.
        let tmp = setup(THREE, None);
        let err = run(&args(&["Backend"]), tmp.path()).unwrap_err();
        assert!(
            matches!(&err, PrimitiveError::InvalidSurfacesMember { value } if value == "Backend"),
            "expected InvalidSurfacesMember, got {err:?}"
        );
    }

    #[test]
    fn unset_surfaces_no_detection_loads_all_recognized() {
        let tmp = setup(THREE, None);
        let result = run(&args(&[]), tmp.path()).unwrap();
        assert_eq!(
            result.selected,
            vec![
                "accessibility-frontend.md",
                "quality-cross.md",
                "security-backend.md"
            ]
        );
        assert_eq!(result.rules_dir, "framework/rules");
        assert_eq!(
            result.notices,
            vec![
                "loading rule files: accessibility-frontend.md, quality-cross.md, security-backend.md"
            ]
        );
    }

    #[test]
    fn surfaces_backend_keeps_backend_and_cross_drops_frontend() {
        let tmp = setup(THREE, Some("[rules]\nsurfaces = [\"backend\"]\n"));
        let result = run(&args(&[]), tmp.path()).unwrap();
        assert_eq!(
            result.selected,
            vec!["quality-cross.md", "security-backend.md"]
        );
    }

    #[test]
    fn empty_surfaces_is_cross_only() {
        let tmp = setup(THREE, Some("[rules]\nsurfaces = []\n"));
        let result = run(&args(&[]), tmp.path()).unwrap();
        assert_eq!(result.selected, vec!["quality-cross.md"]);
    }

    #[test]
    fn unset_surfaces_falls_back_to_detected_stack() {
        let tmp = setup(THREE, None);
        let result = run(&args(&["frontend"]), tmp.path()).unwrap();
        assert_eq!(
            result.selected,
            vec!["accessibility-frontend.md", "quality-cross.md"]
        );
    }

    #[test]
    fn config_surfaces_override_detected_stack() {
        // Config wins even when a detected stack is supplied.
        let tmp = setup(THREE, Some("[rules]\nsurfaces = [\"backend\"]\n"));
        let result = run(&args(&["frontend"]), tmp.path()).unwrap();
        assert_eq!(
            result.selected,
            vec!["quality-cross.md", "security-backend.md"]
        );
    }

    #[test]
    fn unrecognized_member_fails_fast() {
        let tmp = setup(THREE, Some("[rules]\nsurfaces = [\"fullstack\"]\n"));
        let err = run(&args(&[]), tmp.path()).unwrap_err();
        match err {
            PrimitiveError::InvalidSurfacesMember { value } => assert_eq!(value, "fullstack"),
            other => panic!("expected InvalidSurfacesMember, got {other:?}"),
        }
    }

    #[test]
    fn mixed_valid_and_invalid_members_fail_on_the_invalid_one() {
        let tmp = setup(
            THREE,
            Some("[rules]\nsurfaces = [\"backend\", \"fullstack\"]\n"),
        );
        let err = run(&args(&[]), tmp.path()).unwrap_err();
        assert!(matches!(
            err,
            PrimitiveError::InvalidSurfacesMember { value } if value == "fullstack"
        ));
    }

    #[test]
    fn cross_is_not_a_selectable_surface() {
        let tmp = setup(THREE, Some("[rules]\nsurfaces = [\"cross\"]\n"));
        let err = run(&args(&[]), tmp.path()).unwrap_err();
        assert!(matches!(
            err,
            PrimitiveError::InvalidSurfacesMember { value } if value == "cross"
        ));
    }

    #[test]
    fn non_list_surfaces_is_a_type_error() {
        let tmp = setup(THREE, Some("[rules]\nsurfaces = \"backend\"\n"));
        let err = run(&args(&[]), tmp.path()).unwrap_err();
        match err {
            PrimitiveError::InvalidSurfacesType { got } => assert_eq!(got, "a string"),
            other => panic!("expected InvalidSurfacesType, got {other:?}"),
        }
    }

    #[test]
    fn unrecognized_suffix_loads_for_all_stacks_with_warning() {
        let tmp = setup(&["security-backend.md", "internal-notes.md"], None);
        let result = run(&args(&["backend"]), tmp.path()).unwrap();
        assert!(result.selected.contains(&"internal-notes.md".to_string()));
        assert_eq!(
            result.notices[0],
            "rule file internal-notes.md has unrecognized suffix — loading for all stacks; rename to -backend.md, -frontend.md, or -cross.md"
        );
    }

    #[test]
    fn disabled_rule_file_drops_selected_file_with_notice() {
        let toml = "[[review.disabled-rule-files]]\nfile = \"accessibility-frontend.md\"\nreason = \"Internal admin UI; not yet adopting WCAG AA.\"\n";
        let tmp = setup(THREE, Some(toml));
        let result = run(&args(&[]), tmp.path()).unwrap();
        assert!(
            !result
                .selected
                .contains(&"accessibility-frontend.md".to_string())
        );
        assert!(result.notices.contains(
            &"disabled-rule-file: accessibility-frontend.md — Internal admin UI; not yet adopting WCAG AA. (.govern.toml)".to_string()
        ));
    }

    #[test]
    fn disabled_notice_names_new_layout_config() {
        // Same drop, config under `.govern/config.toml` — the provenance
        // tag names the resolved file, not a hardcoded legacy literal.
        let toml = "[[review.disabled-rule-files]]\nfile = \"accessibility-frontend.md\"\nreason = \"Internal admin UI; not yet adopting WCAG AA.\"\n";
        let tmp = setup(THREE, None);
        let cfg_dir = tmp.path().join(".govern");
        std::fs::create_dir_all(&cfg_dir).unwrap();
        std::fs::write(cfg_dir.join("config.toml"), toml).unwrap();
        let result = run(&args(&[]), tmp.path()).unwrap();
        assert!(result.notices.contains(
            &"disabled-rule-file: accessibility-frontend.md — Internal admin UI; not yet adopting WCAG AA. (.govern/config.toml)".to_string()
        ));
    }

    #[test]
    fn disabled_rule_file_non_selected_is_a_no_op_notice() {
        // Backend-only selection; disabling a frontend file that isn't selected.
        let toml = "[rules]\nsurfaces = [\"backend\"]\n\n[[review.disabled-rule-files]]\nfile = \"accessibility-frontend.md\"\nreason = \"Not adopting WCAG AA on this surface yet.\"\n";
        let tmp = setup(THREE, Some(toml));
        let result = run(&args(&[]), tmp.path()).unwrap();
        assert!(result.notices.contains(
            &"disabled-rule-file (no-op): accessibility-frontend.md not selected by stack detection".to_string()
        ));
    }

    #[test]
    fn disabled_rule_file_unknown_warns() {
        let toml = "[[review.disabled-rule-files]]\nfile = \"nonexistent-cross.md\"\nreason = \"This file was renamed or moved elsewhere.\"\n";
        let tmp = setup(THREE, Some(toml));
        let result = run(&args(&[]), tmp.path()).unwrap();
        assert!(result.notices.contains(
            &"unknown disabled-rule-file: nonexistent-cross.md (no such file in the rule-file directory)".to_string()
        ));
    }

    #[test]
    fn disabled_rule_file_missing_reason_is_malformed() {
        let toml = "[[review.disabled-rule-files]]\nfile = \"quality-cross.md\"\n";
        let tmp = setup(THREE, Some(toml));
        let result = run(&args(&[]), tmp.path()).unwrap();
        assert!(
            result.notices.contains(
                &"malformed disabled-rule-file at review.disabled-rule-files[0]: missing 'reason'"
                    .to_string()
            )
        );
        // Not dropped — malformed entries change nothing.
        assert!(result.selected.contains(&"quality-cross.md".to_string()));
    }

    #[test]
    fn disabled_rule_file_short_reason_is_malformed() {
        let toml =
            "[[review.disabled-rule-files]]\nfile = \"quality-cross.md\"\nreason = \"too short\"\n";
        let tmp = setup(THREE, Some(toml));
        let result = run(&args(&[]), tmp.path()).unwrap();
        assert!(result.notices.contains(
            &"malformed disabled-rule-file at review.disabled-rule-files[0]: reason must be at least 16 characters".to_string()
        ));
    }

    #[test]
    fn duplicate_disabled_rule_file_first_applies_rest_warn() {
        let toml = "[[review.disabled-rule-files]]\nfile = \"quality-cross.md\"\nreason = \"First entry, this one applies.\"\n\n[[review.disabled-rule-files]]\nfile = \"quality-cross.md\"\nreason = \"Second entry, should be ignored.\"\n";
        let tmp = setup(THREE, Some(toml));
        let result = run(&args(&[]), tmp.path()).unwrap();
        assert!(!result.selected.contains(&"quality-cross.md".to_string()));
        assert!(result.notices.contains(
            &"duplicate disabled-rule-file: quality-cross.md — entry [1] ignored".to_string()
        ));
    }

    #[test]
    fn reason_whitespace_is_collapsed_in_the_notice() {
        let toml = "[[review.disabled-rule-files]]\nfile = \"quality-cross.md\"\nreason = \"\"\"\nInternal admin UI;\nnot adopting WCAG AA yet.\n\"\"\"\n";
        let tmp = setup(THREE, Some(toml));
        let result = run(&args(&[]), tmp.path()).unwrap();
        assert!(result.notices.contains(
            &"disabled-rule-file: quality-cross.md — Internal admin UI; not adopting WCAG AA yet. (.govern.toml)".to_string()
        ));
    }

    #[test]
    fn notice_ordering_unrecognized_then_disabled_then_loading() {
        let toml = "[[review.disabled-rule-files]]\nfile = \"security-backend.md\"\nreason = \"Backend rules disabled for this probe.\"\n";
        let tmp = setup(&["security-backend.md", "internal-notes.md"], Some(toml));
        let result = run(&args(&["backend"]), tmp.path()).unwrap();
        assert_eq!(result.notices.len(), 3);
        assert!(
            result.notices[0].starts_with("rule file internal-notes.md has unrecognized suffix")
        );
        assert!(result.notices[1].starts_with("disabled-rule-file: security-backend.md"));
        assert!(result.notices[2].starts_with("loading rule files:"));
    }

    #[test]
    fn adopter_layout_uses_specs_rules_directory() {
        let tmp = TempDir::new().unwrap();
        let rules = tmp.path().join("specs/rules");
        std::fs::create_dir_all(&rules).unwrap();
        std::fs::write(rules.join("quality-cross.md"), "# rule\n").unwrap();
        let result = run(&args(&[]), tmp.path()).unwrap();
        assert_eq!(result.rules_dir, "specs/rules");
        assert_eq!(result.selected, vec!["quality-cross.md"]);
    }

    #[test]
    fn no_rule_directory_yields_empty_selection() {
        let tmp = TempDir::new().unwrap();
        let result = run(&args(&[]), tmp.path()).unwrap();
        assert_eq!(result.rules_dir, "");
        assert!(result.selected.is_empty());
        assert_eq!(result.notices, vec!["loading rule files: "]);
    }

    #[test]
    fn malformed_govern_toml_is_operational_error() {
        let tmp = setup(THREE, Some("[[review.broken\n"));
        let err = run(&args(&[]), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::Toml { .. }));
    }
}
