//! `dashboard` — single-call pipeline-state surface for `/{project}:status`.
//!
//! Returns everything `/{project}:status` needs to render the full pipeline
//! view in one MCP round-trip: per-spec inventory (status, deps, tags,
//! open-question count, artifact existence, scenarios count, blocked-by),
//! the repo-wide `tags-union`, the `.govern.toml` review-state summary,
//! and the optional session target (with scenario detail when one is
//! targeted). The session is read from `.govern.session.toml` at the repo
//! root — host-agnostic, project-name-agnostic, no caller-supplied path.
//! Read-only with respect to filesystem state; no atomic-write concerns.
//!
//! Defined by `specs/022-deterministic-runtime/scenarios/dashboard-primitive.md`.

use std::collections::{BTreeSet, HashMap};
use std::path::Path;

use serde::Deserialize;

use crate::primitives::write_session::SESSION_FILE;
use crate::primitives::{
    PrimitiveError, Result, is_feature_slug, read_text, section_lines, split_frontmatter,
};
use crate::schema::paths;
use crate::schema::primitives::{
    DashboardArgs, DashboardConfig, DashboardResult, DashboardScenarioDetail,
    DashboardSessionTarget, DashboardSpec, Frontmatter,
};

/// Statuses that satisfy a dependency. A dep at `draft` blocks downstream
/// consumers; anything `clarified` and above is acceptable. Mirrors the
/// blocked-by rule the markdown encoded before the primitive existed.
const UNBLOCKING_STATUSES: &[&str] = &["clarified", "planned", "in-progress", "done"];

/// Execute the `dashboard` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::MissingSpecFile`] when an `NNN-feature`
/// directory under `specs/` lacks a `spec.md` (the directory naming
/// convention promises one), [`PrimitiveError::Io`] on filesystem
/// failures, [`PrimitiveError::Yaml`] when any spec's frontmatter is
/// malformed, or [`PrimitiveError::Toml`] when `.govern.toml` or
/// `.govern.session.toml` is malformed.
pub fn run(_args: &DashboardArgs, repo: &Path) -> Result<DashboardResult> {
    let specs = load_specs(repo)?;
    let tags_union = compute_tags_union(&specs);
    let config = load_config(repo)?;
    let session_target = load_session_target(repo)?;
    Ok(DashboardResult {
        session_target,
        specs,
        tags_union,
        config,
    })
}

/// Walk `specs/` and build the per-spec entry list. Non-`NNN-feature`
/// directories are skipped; missing `spec.md` halts with a structured
/// error. After the first pass, walks the list a second time to fill in
/// each spec's `blocked-by` (needs every spec's status to be known).
fn load_specs(repo: &Path) -> Result<Vec<DashboardSpec>> {
    let specs_dir = paths::specs_dir(repo);
    let mut entries: Vec<DashboardSpec> = Vec::new();
    if !specs_dir.is_dir() {
        return Ok(entries);
    }
    let mut dir_names: Vec<String> = Vec::new();
    let read_dir = std::fs::read_dir(&specs_dir).map_err(|source| PrimitiveError::Io {
        path: specs_dir.clone(),
        source,
    })?;
    for entry in read_dir {
        let entry = entry.map_err(|source| PrimitiveError::Io {
            path: specs_dir.clone(),
            source,
        })?;
        let file_type = entry.file_type().map_err(|source| PrimitiveError::Io {
            path: entry.path(),
            source,
        })?;
        if !file_type.is_dir() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        if !is_feature_slug(&name) {
            continue;
        }
        dir_names.push(name);
    }
    dir_names.sort();

    for slug in dir_names {
        entries.push(load_one_spec(repo, &slug)?);
    }

    let status_by_slug: HashMap<String, String> = entries
        .iter()
        .map(|s| (s.slug.clone(), s.status.clone()))
        .collect();
    for spec in &mut entries {
        spec.blocked_by = spec
            .dependencies
            .iter()
            .filter(|dep| {
                let status = status_by_slug.get(dep.as_str()).map_or("", String::as_str);
                !UNBLOCKING_STATUSES.contains(&status)
            })
            .cloned()
            .collect();
    }
    Ok(entries)
}

/// Load one spec's dashboard entry. `blocked_by` is filled in by the
/// caller after every spec's status has been read.
fn load_one_spec(repo: &Path, slug: &str) -> Result<DashboardSpec> {
    let root = paths::Paths::load(repo).specs_root;
    let feature_dir = repo.join(&root).join(slug);
    let spec_path = feature_dir.join("spec.md");
    if !spec_path.is_file() {
        return Err(PrimitiveError::MissingSpecFile {
            root,
            feature: slug.to_string(),
        });
    }
    let content = read_text(&spec_path)?;
    let (fm_text, body) = split_frontmatter(&content, &spec_path)?;
    let frontmatter: Frontmatter =
        serde_norway::from_str(fm_text).map_err(|source| PrimitiveError::Yaml {
            path: spec_path.clone(),
            source,
        })?;
    let open_question_count = count_open_questions(body);
    let scenarios_dir = feature_dir.join("scenarios");
    Ok(DashboardSpec {
        slug: slug.to_string(),
        status: frontmatter.status,
        dependencies: frontmatter.dependencies,
        tags: frontmatter.tags,
        open_question_count,
        has_plan: feature_dir.join("plan.md").is_file(),
        has_tasks: feature_dir.join("tasks.md").is_file(),
        has_data_model: feature_dir.join("data-model.md").is_file(),
        scenarios_count: count_scenario_files(&scenarios_dir),
        blocked_by: Vec::new(),
    })
}

/// Count `*.md` files directly under `scenarios_dir`. Subdirectories and
/// non-markdown files are excluded. Returns 0 when the directory is
/// absent or unreadable.
fn count_scenario_files(scenarios_dir: &Path) -> u32 {
    let Ok(read_dir) = std::fs::read_dir(scenarios_dir) else {
        return 0;
    };
    let mut count: u32 = 0;
    for entry in read_dir.flatten() {
        let Ok(file_type) = entry.file_type() else {
            continue;
        };
        if !file_type.is_file() {
            continue;
        }
        if entry
            .path()
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("md"))
        {
            count += 1;
        }
    }
    count
}

/// Count unresolved entries in a spec body's `## Open Questions` section.
/// Top-level list items (`-` bullets) are entries; the canonical
/// `*None — all resolved.*` placeholder is treated as zero. Continuation
/// lines and nested sub-bullets inside an entry don't add to the count.
/// Shares section traversal with `read_spec::parse_open_questions` via
/// the shared `section_lines` helper — the two consumers only differ in
/// how they fold the yielded lines into their result shape.
fn count_open_questions(body: &str) -> u32 {
    let mut count: u32 = 0;
    for line in section_lines(body, "Open Questions") {
        let trimmed = line.trim_start();
        if let Some(rest) = trimmed.strip_prefix("- ") {
            let entry = rest.trim();
            if !entry.is_empty() && entry != "*None — all resolved.*" {
                count += 1;
            }
        }
    }
    count
}

/// Compute the sorted, deduplicated union of every spec's `tags` array.
fn compute_tags_union(specs: &[DashboardSpec]) -> Vec<String> {
    let mut set: BTreeSet<String> = BTreeSet::new();
    for spec in specs {
        for tag in &spec.tags {
            set.insert(tag.clone());
        }
    }
    set.into_iter().collect()
}

/// Minimal TOML shape: just enough of `.govern.toml` to extract the
/// `[[review.disabled-rule-files]]` entries' `file` basenames. Unknown
/// keys are accepted; the primitive only reports what it knows.
#[derive(Deserialize, Default)]
struct GovernConfig {
    #[serde(default)]
    review: Option<ReviewConfig>,
}

#[derive(Deserialize, Default)]
struct ReviewConfig {
    #[serde(default, rename = "disabled-rule-files")]
    disabled_rule_files: Vec<DisabledRuleFile>,
}

#[derive(Deserialize)]
struct DisabledRuleFile {
    file: String,
}

/// Read `.govern.toml` and summarize the review-state for the dashboard.
fn load_config(repo: &Path) -> Result<DashboardConfig> {
    let toml_path = repo.join(".govern.toml");
    if !toml_path.is_file() {
        return Ok(DashboardConfig {
            present: false,
            disabled_rule_files: Vec::new(),
        });
    }
    let content = read_text(&toml_path)?;
    let parsed: GovernConfig = toml::from_str(&content).map_err(|source| PrimitiveError::Toml {
        path: toml_path.clone(),
        source,
    })?;
    let disabled_rule_files = parsed
        .review
        .map(|r| r.disabled_rule_files)
        .unwrap_or_default()
        .into_iter()
        .map(|d| d.file)
        .collect();
    Ok(DashboardConfig {
        present: true,
        disabled_rule_files,
    })
}

/// Minimal session-file shape. The runtime exec subcommand seeds walker
/// context from the same file; the MCP surface reads it directly so MCP
/// callers don't need a second tool call. TOML keys are kebab-case
/// (`scenario-path`, `set-at`); the legacy JSON keys (`scenarioPath`,
/// `setAt`) are not accepted — adopters with the legacy `.claude/*-session.json`
/// file complete the migration via the `/govern` bootstrap pass.
#[derive(Deserialize)]
struct SessionFile {
    // Optional: a session file may exist carrying only the per-contributor
    // `cli-config-dir` (written by `/govern` before any target is selected).
    // No `feature` means no target to surface.
    #[serde(default)]
    feature: Option<String>,
    #[serde(default)]
    scenario: Option<String>,
    #[serde(default, rename = "scenario-path")]
    scenario_path: Option<String>,
}

/// Read `<repo>/.govern.session.toml` (when present) and populate the
/// session-target field. When the targeted scenario file exists, also
/// reads it to populate `scenario-detail`. The session field is echoed
/// as-recorded; `/{project}:target` is the corrective action for stale
/// slugs, not the dashboard.
fn load_session_target(repo: &Path) -> Result<Option<DashboardSessionTarget>> {
    let session_path = repo.join(SESSION_FILE);
    if !session_path.is_file() {
        return Ok(None);
    }
    let content = read_text(&session_path)?;
    let session: SessionFile = toml::from_str(&content).map_err(|source| PrimitiveError::Toml {
        path: session_path.clone(),
        source,
    })?;
    // A session file with no `feature` (e.g. only `cli-config-dir` recorded by
    // `/govern` before a target is selected) carries no target to surface.
    let Some(feature) = session.feature else {
        return Ok(None);
    };
    let scenario_detail = match (&session.scenario, &session.scenario_path) {
        (Some(_), Some(rel_path)) => load_scenario_detail(repo, rel_path)?,
        _ => None,
    };
    Ok(Some(DashboardSessionTarget {
        feature,
        scenario: session.scenario,
        scenario_detail,
    }))
}

/// Read a scenario file and extract its dashboard header detail. Returns
/// `Ok(None)` when the file doesn't exist (session pointing at a stale
/// scenario is the caller's problem, not the dashboard's).
fn load_scenario_detail(repo: &Path, rel_path: &str) -> Result<Option<DashboardScenarioDetail>> {
    let path = repo.join(rel_path);
    if !path.is_file() {
        return Ok(None);
    }
    let content = read_text(&path)?;
    let (fm_text, body) = split_frontmatter(&content, &path)?;
    let frontmatter: ScenarioFrontmatter =
        serde_norway::from_str(fm_text).map_err(|source| PrimitiveError::Yaml {
            path: path.clone(),
            source,
        })?;
    let section = frontmatter
        .section
        .or(frontmatter.spec_ref)
        .unwrap_or_default();
    let context_summary = context_summary(body);
    let open_question_count = count_open_questions(body);
    Ok(Some(DashboardScenarioDetail {
        section,
        context_summary,
        open_question_count,
    }))
}

/// Scenario frontmatter shape — `section` is the post-017 field; `spec-ref`
/// is the pre-017 legacy field still encountered on older scenarios.
#[derive(Deserialize)]
struct ScenarioFrontmatter {
    #[serde(default)]
    section: Option<String>,
    #[serde(default, rename = "spec-ref")]
    spec_ref: Option<String>,
}

/// First non-blank, non-HTML-comment line of the scenario body's
/// `## Context` section, trimmed. Empty string when the section is
/// absent or contains only blanks and comments. Shares section
/// traversal with the open-question counter via `section_lines`.
fn context_summary(body: &str) -> String {
    for line in section_lines(body, "Context") {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with("<!--") {
            return trimmed.to_string();
        }
    }
    String::new()
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use tempfile::TempDir;

    /// Write a minimal spec.md to `repo/specs/{slug}/spec.md` with the
    /// given frontmatter body plus a `## Open Questions` section that
    /// stays empty unless `open_questions` overrides it.
    fn write_spec(repo: &Path, slug: &str, frontmatter: &str, open_questions: &str) {
        let dir = repo.join("specs").join(slug);
        std::fs::create_dir_all(&dir).unwrap();
        let content =
            format!("---\n{frontmatter}---\n\n# {slug}\n\n## Open Questions\n\n{open_questions}\n");
        std::fs::write(dir.join("spec.md"), content).unwrap();
    }

    /// Write the canonical session TOML at `<repo>/.govern.session.toml`.
    /// All tests use the same path — the consolidation removed every
    /// per-host / per-project variability.
    fn write_session_toml(repo: &Path, body: &str) {
        std::fs::write(repo.join(".govern.session.toml"), body).unwrap();
    }

    #[test]
    fn empty_specs_dir_returns_empty_list() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("specs")).unwrap();
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        assert!(result.specs.is_empty());
        assert!(result.tags_union.is_empty());
        assert!(!result.config.present);
        assert!(result.session_target.is_none());
    }

    #[test]
    fn missing_specs_dir_returns_empty_list() {
        let tmp = TempDir::new().unwrap();
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        assert!(result.specs.is_empty());
    }

    #[test]
    fn happy_path_aggregates_inventory() {
        let tmp = TempDir::new().unwrap();
        write_spec(
            tmp.path(),
            "001-alpha",
            "status: done\ndependencies: []\ntags: [format, pipeline]\n",
            "*None — all resolved.*",
        );
        write_spec(
            tmp.path(),
            "002-beta",
            "status: planned\ndependencies: [001-alpha]\ntags: [pipeline]\n",
            "- What about edge case Z?\n- And case W?",
        );
        // Add a plan.md and tasks.md for 002-beta to exercise artifact
        // existence; leave 001-alpha bare.
        std::fs::write(tmp.path().join("specs/002-beta/plan.md"), "# plan").unwrap();
        std::fs::write(tmp.path().join("specs/002-beta/tasks.md"), "# tasks").unwrap();
        std::fs::create_dir_all(tmp.path().join("specs/002-beta/scenarios")).unwrap();
        std::fs::write(tmp.path().join("specs/002-beta/scenarios/x.md"), "# x").unwrap();
        std::fs::write(tmp.path().join("specs/002-beta/scenarios/y.md"), "# y").unwrap();
        std::fs::write(tmp.path().join("specs/002-beta/scenarios/README"), "# r").unwrap();

        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        assert_eq!(result.specs.len(), 2);
        assert_eq!(result.specs[0].slug, "001-alpha");
        assert_eq!(result.specs[1].slug, "002-beta");

        let alpha = &result.specs[0];
        assert_eq!(alpha.status, "done");
        assert!(alpha.dependencies.is_empty());
        assert_eq!(alpha.tags, vec!["format", "pipeline"]);
        assert_eq!(alpha.open_question_count, 0);
        assert!(!alpha.has_plan);
        assert!(!alpha.has_tasks);
        assert!(!alpha.has_data_model);
        assert_eq!(alpha.scenarios_count, 0);
        assert!(alpha.blocked_by.is_empty());

        let beta = &result.specs[1];
        assert_eq!(beta.status, "planned");
        assert_eq!(beta.dependencies, vec!["001-alpha"]);
        assert_eq!(beta.tags, vec!["pipeline"]);
        assert_eq!(beta.open_question_count, 2);
        assert!(beta.has_plan);
        assert!(beta.has_tasks);
        assert!(!beta.has_data_model);
        assert_eq!(beta.scenarios_count, 2);
        assert!(beta.blocked_by.is_empty());

        assert_eq!(result.tags_union, vec!["format", "pipeline"]);
    }

    #[test]
    fn non_pattern_dirs_skip_silently() {
        let tmp = TempDir::new().unwrap();
        write_spec(
            tmp.path(),
            "001-real",
            "status: draft\ndependencies: []\n",
            "*None — all resolved.*",
        );
        std::fs::create_dir_all(tmp.path().join("specs/templates")).unwrap();
        std::fs::create_dir_all(tmp.path().join("specs/.hidden")).unwrap();
        std::fs::write(tmp.path().join("specs/inbox.md"), "# inbox").unwrap();
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        assert_eq!(result.specs.len(), 1);
        assert_eq!(result.specs[0].slug, "001-real");
    }

    #[test]
    fn missing_spec_md_is_operational_error() {
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join("specs/099-broken")).unwrap();
        let err = run(&DashboardArgs::default(), tmp.path()).unwrap_err();
        match err {
            PrimitiveError::MissingSpecFile { feature, root } => {
                assert_eq!(feature, "099-broken");
                assert_eq!(root, "specs");
            }
            other => panic!("expected MissingSpecFile, got {other:?}"),
        }
    }

    #[test]
    fn blocked_by_computes_from_dependency_status() {
        let tmp = TempDir::new().unwrap();
        // 001 is draft → blocks anything depending on it.
        write_spec(
            tmp.path(),
            "001-draft",
            "status: draft\ndependencies: []\n",
            "*None — all resolved.*",
        );
        // 002 is clarified → unblocks.
        write_spec(
            tmp.path(),
            "002-ready",
            "status: clarified\ndependencies: []\n",
            "*None — all resolved.*",
        );
        // 003 depends on both: 001 blocks it, 002 doesn't.
        write_spec(
            tmp.path(),
            "003-dep",
            "status: planned\ndependencies: [001-draft, 002-ready]\n",
            "*None — all resolved.*",
        );
        // 004 names a nonexistent dep — treated as blocking (empty status).
        write_spec(
            tmp.path(),
            "004-ghost",
            "status: planned\ndependencies: [999-missing]\n",
            "*None — all resolved.*",
        );
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        let by_slug: HashMap<_, _> = result.specs.iter().map(|s| (s.slug.as_str(), s)).collect();
        assert!(by_slug["001-draft"].blocked_by.is_empty());
        assert!(by_slug["002-ready"].blocked_by.is_empty());
        assert_eq!(by_slug["003-dep"].blocked_by, vec!["001-draft"]);
        assert_eq!(by_slug["004-ghost"].blocked_by, vec!["999-missing"]);
    }

    #[test]
    fn govern_toml_absent_returns_present_false() {
        let tmp = TempDir::new().unwrap();
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        assert!(!result.config.present);
        assert!(result.config.disabled_rule_files.is_empty());
    }

    #[test]
    fn govern_toml_present_but_empty_returns_present_true() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".govern.toml"), "# empty\n").unwrap();
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        assert!(result.config.present);
        assert!(result.config.disabled_rule_files.is_empty());
    }

    #[test]
    fn govern_toml_disabled_rule_files_extracted() {
        let tmp = TempDir::new().unwrap();
        let toml = r#"
[[review.disabled-rule-files]]
file = "accessibility-frontend.md"
reason = "Internal admin UI; not yet adopting WCAG AA."

[[review.disabled-rule-files]]
file = "performance-frontend.md"
reason = "Deferred until v2 perf budget lands."
"#;
        std::fs::write(tmp.path().join(".govern.toml"), toml).unwrap();
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        assert!(result.config.present);
        assert_eq!(
            result.config.disabled_rule_files,
            vec!["accessibility-frontend.md", "performance-frontend.md"]
        );
    }

    #[test]
    fn govern_toml_parse_failure_is_operational_error() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(tmp.path().join(".govern.toml"), "[[review.broken\n").unwrap();
        let err = run(&DashboardArgs::default(), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::Toml { .. }));
    }

    #[test]
    fn session_absent_returns_none() {
        let tmp = TempDir::new().unwrap();
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        assert!(result.session_target.is_none());
    }

    #[test]
    fn session_feature_only_populates_target() {
        let tmp = TempDir::new().unwrap();
        write_session_toml(
            tmp.path(),
            "feature = \"022-deterministic-runtime\"\npath = \"specs/022-deterministic-runtime\"\nset-at = \"2026-05-23T12:34:56Z\"\n",
        );
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        let target = result.session_target.unwrap();
        assert_eq!(target.feature, "022-deterministic-runtime");
        assert!(target.scenario.is_none());
        assert!(target.scenario_detail.is_none());
    }

    #[test]
    fn dashboard_reads_session_from_same_path_regardless_of_project_name() {
        // Headline of the consolidation: the dashboard reads
        // `.govern.session.toml` at the repo root. The path doesn't depend
        // on project name (`gov` vs `anvil`) or AI CLI (`.claude/` vs
        // `.augment/`). The legacy `.claude/{project}-session.json` files
        // are not consulted — adopters migrate via /govern.
        let tmp = TempDir::new().unwrap();
        write_session_toml(
            tmp.path(),
            "feature = \"002-observability\"\npath = \"specs/002-observability\"\nset-at = \"2026-05-23T12:34:56Z\"\n",
        );
        // No `.claude/` or `.augment/` directory needs to exist.
        assert!(!tmp.path().join(".claude").exists());
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        let target = result.session_target.unwrap();
        assert_eq!(target.feature, "002-observability");
    }

    #[test]
    fn legacy_json_session_file_is_ignored() {
        // An adopter who hasn't yet run /govern post-consolidation may
        // still have `.claude/gov-session.json` on disk. The dashboard
        // does not read it — only `.govern.session.toml`. Adopters in
        // this state see "no target" until they re-/gov:target or
        // /govern migrates them.
        let tmp = TempDir::new().unwrap();
        std::fs::create_dir_all(tmp.path().join(".claude")).unwrap();
        std::fs::write(
            tmp.path().join(".claude/gov-session.json"),
            r#"{"feature":"legacy-feature","path":"specs/legacy"}"#,
        )
        .unwrap();
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        assert!(result.session_target.is_none());
    }

    #[test]
    fn malformed_session_toml_is_operational_error() {
        let tmp = TempDir::new().unwrap();
        write_session_toml(tmp.path(), "feature = [unclosed array\n");
        let err = run(&DashboardArgs::default(), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::Toml { .. }));
    }

    #[test]
    fn session_with_scenario_populates_detail() {
        let tmp = TempDir::new().unwrap();
        write_spec(
            tmp.path(),
            "022-foo",
            "status: in-progress\ndependencies: []\n",
            "*None — all resolved.*",
        );
        let scenarios = tmp.path().join("specs/022-foo/scenarios");
        std::fs::create_dir_all(&scenarios).unwrap();
        std::fs::write(
            scenarios.join("widget.md"),
            "---\nsection: \"Follow-on scenarios\"\n---\n\n# Widget\n\n## Context\n\nWidget exists to demonstrate gizmos.\n\n## Open Questions\n\n- One unresolved item.\n",
        )
        .unwrap();
        write_session_toml(
            tmp.path(),
            "feature = \"022-foo\"\npath = \"specs/022-foo\"\nscenario = \"widget\"\nscenario-path = \"specs/022-foo/scenarios/widget.md\"\nset-at = \"2026-05-23T12:34:56Z\"\n",
        );
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        let target = result.session_target.unwrap();
        assert_eq!(target.feature, "022-foo");
        assert_eq!(target.scenario.as_deref(), Some("widget"));
        let detail = target.scenario_detail.unwrap();
        assert_eq!(detail.section, "Follow-on scenarios");
        assert_eq!(
            detail.context_summary,
            "Widget exists to demonstrate gizmos."
        );
        assert_eq!(detail.open_question_count, 1);
    }

    #[test]
    fn session_with_stale_scenario_returns_target_without_detail() {
        let tmp = TempDir::new().unwrap();
        write_session_toml(
            tmp.path(),
            "feature = \"022-foo\"\npath = \"specs/022-foo\"\nscenario = \"ghost\"\nscenario-path = \"specs/022-foo/scenarios/ghost.md\"\nset-at = \"2026-05-23T12:34:56Z\"\n",
        );
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        let target = result.session_target.unwrap();
        assert_eq!(target.feature, "022-foo");
        assert_eq!(target.scenario.as_deref(), Some("ghost"));
        assert!(target.scenario_detail.is_none());
    }

    #[test]
    fn open_question_count_ignores_continuation_lines() {
        let tmp = TempDir::new().unwrap();
        write_spec(
            tmp.path(),
            "001-x",
            "status: draft\ndependencies: []\n",
            "- First question with\n  a continuation line.\n- Second question.",
        );
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        assert_eq!(result.specs[0].open_question_count, 2);
    }
}
