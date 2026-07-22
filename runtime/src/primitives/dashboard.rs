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
use std::fmt::Write as _;
use std::path::Path;

use serde::Deserialize;

use crate::host::Host;
use crate::primitives::resolve_references::{self, load_services};
use crate::primitives::{
    PrimitiveError, Result, ScenarioFrontmatter, feature_number, list_feature_dirs,
    list_scenario_files, read_text, section_lines, split_frontmatter,
};
use crate::schema::paths;
use crate::schema::primitives::{
    DashboardArgs, DashboardConfig, DashboardResult, DashboardScenarioDetail,
    DashboardSessionTarget, DashboardSpec, Frontmatter, ReferenceOutcome, ResolutionRecord,
    ResolveReferencesArgs,
};
use crate::schema::services::Services;
use crate::schema::status::{ALLOWED_STATUSES, UNBLOCKING_STATUSES};

/// Execute the `dashboard` primitive.
///
/// # Errors
///
/// Returns [`PrimitiveError::MissingSpecFile`] when an `NNN-feature`
/// directory under `specs/` lacks a `spec.md` (the directory naming
/// convention promises one), [`PrimitiveError::Io`] on filesystem
/// failures, [`PrimitiveError::Yaml`] when any spec's frontmatter is
/// malformed, or [`PrimitiveError::Toml`] when `.govern.toml` or
/// `.govern.session.toml` is malformed. A *targeted scenario* with
/// missing or malformed frontmatter is NOT an error — it degrades to a
/// detail-less session target (see [`load_scenario_detail`]).
pub fn run(_args: &DashboardArgs, repo: &Path) -> Result<DashboardResult> {
    let specs = load_specs(repo)?;
    let tags_union = compute_tags_union(&specs);
    let config = load_config(repo)?;
    let session_target = load_session_target(repo)?;
    let rendered_markdown =
        render_markdown(repo, &specs, &tags_union, &config, session_target.as_ref())?;
    Ok(DashboardResult {
        session_target,
        specs,
        tags_union,
        config,
        rendered_markdown,
    })
}

// ---------------------------------------------------------------------------
// Rendered pipeline view (spec 022, scenario coverage-expansion-primitives)
//
// `/gov:status` previously spent five of its six steps on LLM-side
// rendering of this payload. The runtime pre-renders the same four pieces
// — preamble, dashboard table, counts/callouts, references readout — as
// one markdown fragment the host may restyle. Returned data, never stdout
// printing: user-facing rendering stays with the host (§runtime-boundary).
// ---------------------------------------------------------------------------

/// Render the full pipeline view. Blocks are joined by blank lines; the
/// references readout is omitted entirely when no spec declares
/// references (a single-service adopter sees no change).
fn render_markdown(
    repo: &Path,
    specs: &[DashboardSpec],
    tags_union: &[String],
    config: &DashboardConfig,
    session_target: Option<&DashboardSessionTarget>,
) -> Result<String> {
    let project = Host::load(repo).project;
    let mut blocks = vec![
        render_preamble(specs, session_target, &project),
        render_table(specs, session_target, &project),
        render_callouts(specs, tags_union, config, &project),
    ];
    if let Some(readout) = render_references(repo, specs, &project)? {
        blocks.push(readout);
    }
    blocks.retain(|b| !b.is_empty());
    Ok(blocks.join("\n\n"))
}

/// The preamble line(s) above the table. With a session target:
/// `Target: {feature} / {status} / next: {next-action}`, plus a
/// `Scenario: …` line when a scenario is targeted (a scenario with
/// unresolved questions overrides the next action). Without one, the
/// pointer to `/{project}:target`.
fn render_preamble(
    specs: &[DashboardSpec],
    session_target: Option<&DashboardSessionTarget>,
    project: &str,
) -> String {
    let Some(target) = session_target else {
        return format!("No session target. Run /{project}:target to select one.");
    };
    // A target naming a spec that is not on disk degrades to placeholders
    // rather than failing the whole render.
    let (status, mut next) = specs.iter().find(|s| s.slug == target.feature).map_or_else(
        || ("unknown".to_string(), "—".to_string()),
        |s| (s.status.clone(), next_action(s, project)),
    );
    if target
        .scenario_detail
        .as_ref()
        .is_some_and(|d| d.open_question_count >= 1)
    {
        // A targeted scenario with unresolved questions owns the next
        // action regardless of the parent spec's status.
        next = format!("/{project}:clarify (scenario-targeted)");
    }
    let mut out = format!("Target: {} / {status} / next: {next}", target.feature);
    if let Some(scenario) = &target.scenario {
        let (section, open) = target
            .scenario_detail
            .as_ref()
            .map(|d| (d.section.clone(), d.open_question_count))
            .unwrap_or_default();
        let _ = write!(
            out,
            "\nScenario: {scenario} ({section}) — open-questions: {open}"
        );
    }
    out
}

/// The dashboard table, one row per spec in directory order. The session
/// target's Feature cell is bold; artifact flags render `✓`/`—`; the
/// Dependencies column shows sorted three-digit `NNN` prefixes (`—` when
/// empty).
fn render_table(
    specs: &[DashboardSpec],
    session_target: Option<&DashboardSessionTarget>,
    project: &str,
) -> String {
    let target_slug = session_target.map(|t| t.feature.as_str());
    let mut rows = vec![
        "| Feature | Status | Plan | Tasks | Data-model | Scenarios | Dependencies | Next Action |"
            .to_string(),
        "| --- | --- | --- | --- | --- | --- | --- | --- |".to_string(),
    ];
    for spec in specs {
        let feature = if Some(spec.slug.as_str()) == target_slug {
            format!("**{}**", spec.slug)
        } else {
            spec.slug.clone()
        };
        rows.push(format!(
            "| {feature} | {} | {} | {} | {} | {} | {} | {} |",
            spec.status,
            mark(spec.has_plan),
            mark(spec.has_tasks),
            mark(spec.has_data_model),
            spec.scenarios_count,
            dependency_prefixes(&spec.dependencies),
            next_action(spec, project),
        ));
    }
    rows.join("\n")
}

/// Counts per status plus the conditional callouts (blocked, recovery,
/// tags, disabled rule files), one line each.
fn render_callouts(
    specs: &[DashboardSpec],
    tags_union: &[String],
    config: &DashboardConfig,
    project: &str,
) -> String {
    let mut lines: Vec<String> = Vec::new();
    if !specs.is_empty() {
        // Lifecycle order; an out-of-set status (a hand-edited
        // frontmatter) still counts, appended after the known tiers.
        let mut parts: Vec<String> = Vec::new();
        let mut counted: Vec<&str> = Vec::new();
        for status in ALLOWED_STATUSES {
            let n = specs.iter().filter(|s| s.status == *status).count();
            if n > 0 {
                parts.push(format!("{status} {n}"));
                counted.push(status);
            }
        }
        for spec in specs {
            if !counted.contains(&spec.status.as_str()) {
                let n = specs.iter().filter(|s| s.status == spec.status).count();
                parts.push(format!("{} {n}", spec.status));
                counted.push(spec.status.as_str());
            }
        }
        lines.push(format!("Counts: {}", parts.join(" · ")));
    }
    let blocked: Vec<&str> = specs
        .iter()
        .filter(|s| !s.blocked_by.is_empty())
        .map(|s| s.slug.as_str())
        .collect();
    if !blocked.is_empty() {
        lines.push(format!(
            "Blocked: {} spec(s) — {}",
            blocked.len(),
            blocked.join(", ")
        ));
    }
    let recovery: Vec<&str> = specs
        .iter()
        .filter(|s| in_recovery(s))
        .map(|s| s.slug.as_str())
        .collect();
    if !recovery.is_empty() {
        lines.push(format!(
            "{} spec(s) in recovery state: {}. Run /{project}:clarify on each to walk the \
             questions; the spec reverts to draft and advances forward again.",
            recovery.len(),
            recovery.join(", ")
        ));
    }
    if !tags_union.is_empty() {
        lines.push(format!("tags: {}", tags_union.join(", ")));
    }
    if config.present && !config.disabled_rule_files.is_empty() {
        lines.push(format!(
            "disabled rule files: {} (.govern.toml) — {}",
            config.disabled_rule_files.len(),
            config.disabled_rule_files.join(", ")
        ));
    }
    lines.join("\n")
}

/// The cross-service references readout: one section per spec whose
/// derived `references:` index is non-empty, resolved through the same
/// classification `resolve-references` exposes (composed here so one
/// `dashboard` call renders the whole view). `None` when no spec declares
/// references.
fn render_references(
    repo: &Path,
    specs: &[DashboardSpec],
    project: &str,
) -> Result<Option<String>> {
    let services = load_services(repo)?;
    let mut sections: Vec<String> = Vec::new();
    for spec in specs {
        let resolved = resolve_references::run(
            &ResolveReferencesArgs {
                feature: spec.slug.clone(),
            },
            repo,
        )?;
        if resolved.references.is_empty() {
            continue;
        }
        let mut lines = vec![format!("**{}**", spec.slug), String::new()];
        for record in &resolved.references {
            lines.push(format!("- {}", reference_line(record, &services, project)));
        }
        sections.push(lines.join("\n"));
    }
    if sections.is_empty() {
        return Ok(None);
    }
    Ok(Some(format!("References:\n\n{}", sections.join("\n\n"))))
}

/// One readout line per resolution record, with the matched service's
/// `description` appended for orientation when present.
fn reference_line(record: &ResolutionRecord, services: &Services, project: &str) -> String {
    let name = match &record.service {
        Some(service) => format!("{service}/{}", record.spec),
        None => record.spec.clone(),
    };
    let body = match record.outcome {
        ReferenceOutcome::Ok => {
            format!("{name} → {}", record.status.as_deref().unwrap_or("unknown"))
        }
        ReferenceOutcome::Unregistered => format!(
            "{name} — status not attempted (unregistered; run /{project}:link to register the \
             service)"
        ),
        ReferenceOutcome::NotCheckedOut => format!("{name} → unknown (service not checked out)"),
        ReferenceOutcome::StatusUnreadable => format!("{name} → unknown (status unreadable)"),
        ReferenceOutcome::Broken => format!(
            "{name} → broken reference (target spec missing; also reported by /{project}:analyze)"
        ),
    };
    match record
        .service
        .as_deref()
        .and_then(|alias| services.0.get(alias))
        .and_then(|entry| entry.description.as_deref())
    {
        Some(description) if !description.is_empty() => format!("{body} — {description}"),
        _ => body,
    }
}

/// The Status → next action mapping, with the `clarify (recovery)`
/// override for a non-draft spec that has re-acquired open questions.
/// An out-of-set status echoes as-is (`validate-frontmatter` owns
/// flagging it).
fn next_action(spec: &DashboardSpec, project: &str) -> String {
    if in_recovery(spec) {
        return "clarify (recovery)".to_string();
    }
    match spec.status.as_str() {
        "draft" => format!("/{project}:clarify"),
        "clarified" => format!("/{project}:plan"),
        "planned" | "in-progress" => format!("/{project}:implement"),
        "done" => "done (spec is complete)".to_string(),
        other => other.to_string(),
    }
}

/// Recovery state: a spec past clarification whose body has re-acquired
/// open questions (usually a manual frontmatter edit).
fn in_recovery(spec: &DashboardSpec) -> bool {
    matches!(
        spec.status.as_str(),
        "clarified" | "planned" | "in-progress"
    ) && spec.open_question_count >= 1
}

/// Artifact-existence table mark.
fn mark(present: bool) -> &'static str {
    if present { "✓" } else { "—" }
}

/// The Dependencies column: sorted three-digit `NNN` prefixes from the
/// dependency slugs (`—` when empty; a slug without an `NNN-` prefix
/// passes through raw rather than vanishing).
fn dependency_prefixes(dependencies: &[String]) -> String {
    if dependencies.is_empty() {
        return "—".to_string();
    }
    let mut prefixes: Vec<String> = dependencies
        .iter()
        .map(|dep| feature_number(dep).map_or_else(|| dep.clone(), |n| format!("{n:03}")))
        .collect();
    prefixes.sort();
    prefixes.join(", ")
}

/// Walk `specs/` and build the per-spec entry list. Non-`NNN-feature`
/// directories are skipped (via the shared [`list_feature_dirs`]); a
/// present-but-missing `spec.md` still halts with a structured error from
/// [`load_one_spec`]. After the first pass, walks the list a second time
/// to fill in each spec's `blocked-by` (needs every spec's status known).
fn load_specs(repo: &Path) -> Result<Vec<DashboardSpec>> {
    let layout = paths::Paths::load(repo);
    let specs_dir = repo.join(&layout.specs_root);
    let mut entries: Vec<DashboardSpec> = Vec::new();

    for slug in list_feature_dirs(&specs_dir) {
        entries.push(load_one_spec(&specs_dir, &layout.specs_root, &slug)?);
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
fn load_one_spec(specs_dir: &Path, root: &str, slug: &str) -> Result<DashboardSpec> {
    let feature_dir = specs_dir.join(slug);
    let spec_path = feature_dir.join("spec.md");
    if !spec_path.is_file() {
        return Err(PrimitiveError::MissingSpecFile {
            root: root.to_owned(),
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

/// Count `*.md` files directly under `scenarios_dir` via the shared
/// case-insensitive [`list_scenario_files`] — the same set `check-artifacts`
/// derives scenario slugs from. Returns 0 when the directory is absent or
/// unreadable.
fn count_scenario_files(scenarios_dir: &Path) -> u32 {
    u32::try_from(list_scenario_files(scenarios_dir).len()).unwrap_or(u32::MAX)
}

/// Count unresolved entries in a spec body's `## Open Questions` section.
/// Every `- ` bullet at any indentation is an entry — nested sub-bullets
/// DO count, matching `read_spec::parse_open_questions`, which likewise
/// starts a new question on each `- ` line regardless of indent (the
/// two-paths parity). Non-bullet continuation lines don't add to the
/// count, and the canonical `*None — all resolved.*` placeholder is
/// treated as zero. Shares section traversal with `read_spec` via the
/// shared `section_lines` helper — the two consumers only differ in how
/// they fold the yielded lines into their result shape.
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
    let toml_path = paths::config_path(repo);
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
    let session_path = paths::session_path(repo);
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
/// `Ok(None)` when the file doesn't exist OR exists with missing/
/// malformed frontmatter — a parse failure degrades to a detail-less
/// target exactly like the stale-path case, so one bad scenario cannot
/// brick the whole `/gov:status` render (scenario
/// primitive-robustness-hardening). The corrective surface for both is
/// the same: re-target or fix the scenario file.
fn load_scenario_detail(repo: &Path, rel_path: &str) -> Result<Option<DashboardScenarioDetail>> {
    let path = repo.join(rel_path);
    if !path.is_file() {
        return Ok(None);
    }
    let content = read_text(&path)?;
    let Ok((fm_text, body)) = split_frontmatter(&content, &path) else {
        return Ok(None);
    };
    let Ok(frontmatter) = serde_norway::from_str::<ScenarioFrontmatter>(fm_text) else {
        return Ok(None);
    };
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

    #[test]
    fn open_question_count_includes_nested_sub_bullets() {
        // Documented behavior (read_spec parity): every `- ` bullet at
        // any indentation is an entry — nested sub-bullets DO count.
        let tmp = TempDir::new().unwrap();
        write_spec(
            tmp.path(),
            "001-x",
            "status: draft\ndependencies: []\n",
            "- Top-level question?\n  - Nested sub-bullet also counts.\n- Second top-level?",
        );
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        assert_eq!(result.specs[0].open_question_count, 3);
    }

    #[test]
    fn session_with_malformed_scenario_frontmatter_degrades_to_detail_less_target() {
        // Scenario primitive-robustness-hardening: a targeted scenario
        // whose frontmatter fails YAML parse must degrade exactly like
        // the missing-file case — target surfaced, no detail — instead
        // of erroring the whole dashboard (one bad scenario bricking
        // /gov:status).
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
            scenarios.join("broken.md"),
            "---\nsection: [unclosed\n---\n\n# Broken\n\n## Context\n\nStill here.\n",
        )
        .unwrap();
        write_session_toml(
            tmp.path(),
            "feature = \"022-foo\"\npath = \"specs/022-foo\"\nscenario = \"broken\"\nscenario-path = \"specs/022-foo/scenarios/broken.md\"\nset-at = \"2026-05-23T12:34:56Z\"\n",
        );
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        let target = result.session_target.unwrap();
        assert_eq!(target.feature, "022-foo");
        assert_eq!(target.scenario.as_deref(), Some("broken"));
        assert!(
            target.scenario_detail.is_none(),
            "malformed frontmatter must degrade, not error"
        );
    }

    #[test]
    fn session_with_scenario_missing_frontmatter_degrades_to_detail_less_target() {
        // No `---` fence at all — the split itself fails. Same
        // degradation as the parse-failure case.
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
            scenarios.join("bare.md"),
            "# Bare\n\n## Context\n\nNo frontmatter here.\n",
        )
        .unwrap();
        write_session_toml(
            tmp.path(),
            "feature = \"022-foo\"\npath = \"specs/022-foo\"\nscenario = \"bare\"\nscenario-path = \"specs/022-foo/scenarios/bare.md\"\nset-at = \"2026-05-23T12:34:56Z\"\n",
        );
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        let target = result.session_target.unwrap();
        assert_eq!(target.scenario.as_deref(), Some("bare"));
        assert!(target.scenario_detail.is_none());
    }

    /// Pin the slash-command namespace so rendered `/{project}:…` texts
    /// are deterministic (the default falls back to the tempdir's random
    /// basename).
    fn pin_project(repo: &Path, extra: &str) {
        std::fs::write(
            repo.join(".govern.toml"),
            format!("[host]\nproject = \"gov\"\n{extra}"),
        )
        .unwrap();
    }

    #[test]
    fn rendered_no_target_preamble_table_and_counts() {
        let tmp = TempDir::new().unwrap();
        pin_project(tmp.path(), "");
        write_spec(
            tmp.path(),
            "001-alpha",
            "status: draft\ndependencies: []\n",
            "- Undecided thing?",
        );
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        let rendered = &result.rendered_markdown;
        assert!(
            rendered.starts_with("No session target. Run /gov:target to select one."),
            "{rendered}"
        );
        assert!(
            rendered.contains(
                "| Feature | Status | Plan | Tasks | Data-model | Scenarios | Dependencies | Next Action |"
            ),
            "{rendered}"
        );
        assert!(
            rendered.contains("| 001-alpha | draft | — | — | — | 0 | — | /gov:clarify |"),
            "{rendered}"
        );
        assert!(rendered.contains("Counts: draft 1"), "{rendered}");
        assert!(
            !rendered.contains("References:"),
            "readout omitted with no references: {rendered}"
        );
    }

    #[test]
    fn rendered_target_bold_row_recovery_blocked_and_config_callouts() {
        let tmp = TempDir::new().unwrap();
        pin_project(
            tmp.path(),
            "\n[[review.disabled-rule-files]]\nfile = \"security-backend.md\"\nreason = \"n/a\"\n",
        );
        write_spec(
            tmp.path(),
            "001-alpha",
            "status: draft\ndependencies: []\n",
            "*None — all resolved.*",
        );
        // planned + open questions → recovery override; depends on the
        // draft spec → blocked.
        write_spec(
            tmp.path(),
            "002-beta",
            "status: planned\ndependencies: [001-alpha]\ntags: [pipeline]\n",
            "- What about Z?",
        );
        write_session_toml(
            tmp.path(),
            "feature = \"002-beta\"\npath = \"specs/002-beta\"\nset-at = \"2026-05-23T12:34:56Z\"\n",
        );
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        let rendered = &result.rendered_markdown;
        assert!(
            rendered.starts_with("Target: 002-beta / planned / next: clarify (recovery)"),
            "{rendered}"
        );
        assert!(
            rendered.contains("| **002-beta** | planned |"),
            "target row bolded: {rendered}"
        );
        assert!(rendered.contains("| 001-alpha | draft |"), "{rendered}");
        assert!(
            rendered.contains("| 001 | clarify (recovery) |"),
            "NNN dep prefix + recovery next action: {rendered}"
        );
        assert!(
            rendered.contains("Counts: draft 1 · planned 1"),
            "{rendered}"
        );
        assert!(
            rendered.contains("Blocked: 1 spec(s) — 002-beta"),
            "{rendered}"
        );
        assert!(
            rendered.contains(
                "1 spec(s) in recovery state: 002-beta. Run /gov:clarify on each to walk the questions; the spec reverts to draft and advances forward again."
            ),
            "{rendered}"
        );
        assert!(rendered.contains("tags: pipeline"), "{rendered}");
        assert!(
            rendered.contains("disabled rule files: 1 (.govern.toml) — security-backend.md"),
            "{rendered}"
        );
    }

    #[test]
    fn rendered_scenario_line_and_next_action_override() {
        let tmp = TempDir::new().unwrap();
        pin_project(tmp.path(), "");
        write_spec(
            tmp.path(),
            "022-foo",
            "status: done\ndependencies: []\n",
            "*None — all resolved.*",
        );
        let scenarios = tmp.path().join("specs/022-foo/scenarios");
        std::fs::create_dir_all(&scenarios).unwrap();
        std::fs::write(
            scenarios.join("edge.md"),
            "---\nsection: \"Core\"\n---\n\n# Edge\n\n## Context\n\nA thing.\n\n## Open Questions\n\n- Unresolved?\n",
        )
        .unwrap();
        write_session_toml(
            tmp.path(),
            "feature = \"022-foo\"\npath = \"specs/022-foo\"\nscenario = \"edge\"\nscenario-path = \"specs/022-foo/scenarios/edge.md\"\nset-at = \"2026-05-23T12:34:56Z\"\n",
        );
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        let rendered = &result.rendered_markdown;
        assert!(
            rendered.starts_with(
                "Target: 022-foo / done / next: /gov:clarify (scenario-targeted)\nScenario: edge (Core) — open-questions: 1"
            ),
            "scenario with open questions owns the next action: {rendered}"
        );
    }

    #[test]
    fn rendered_references_readout_covers_outcomes_and_descriptions() {
        let tmp = TempDir::new().unwrap();
        std::fs::write(
            tmp.path().join(".govern.toml"),
            "[host]\nproject = \"gov\"\n\n[services.api]\nrepo = \"https://github.com/acme/api\"\npath = \"checkouts/api\"\ndescription = \"Main API service\"\n",
        )
        .unwrap();
        // Linked checkout with one resolvable spec.
        let linked = tmp.path().join("checkouts/api/specs/003-user");
        std::fs::create_dir_all(&linked).unwrap();
        std::fs::write(
            linked.join("spec.md"),
            "---\nstatus: done\ndependencies: []\n---\n\n# 003\n",
        )
        .unwrap();
        write_spec(
            tmp.path(),
            "001-consumer",
            "status: in-progress\ndependencies: []\nreferences:\n  - service: api\n    spec: 003-user\n  - spec: 007-nav\n  - service: api\n    spec: 999-gone\n",
            "*None — all resolved.*",
        );
        let result = run(&DashboardArgs::default(), tmp.path()).unwrap();
        let rendered = &result.rendered_markdown;
        assert!(rendered.contains("References:"), "{rendered}");
        assert!(rendered.contains("**001-consumer**"), "{rendered}");
        assert!(
            rendered.contains("- api/003-user → done — Main API service"),
            "{rendered}"
        );
        assert!(
            rendered.contains(
                "- 007-nav — status not attempted (unregistered; run /gov:link to register the service)"
            ),
            "{rendered}"
        );
        assert!(
            rendered.contains(
                "- api/999-gone → broken reference (target spec missing; also reported by /gov:analyze) — Main API service"
            ),
            "{rendered}"
        );
    }
}
