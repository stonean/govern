//! `create-plan-artifacts` — copy the plan/tasks/data-model templates
//! into an existing feature directory.
//!
//! The deterministic template-copy and existing-artifact-detection step of
//! `/gov:plan` (spec 022, scenario coverage-expansion-primitives): the
//! plan-side mirror of `create-feature`, which covers only `spec.md`. Each
//! copy is atomic and mode-preserving ([`write_atomic_bytes`] +
//! [`mirror_source_mode`], same as `create-feature`); templates resolve
//! through the shared [`resolve_template`] candidate order. The LLM fills
//! the copied sections afterwards via `writeSpecBody`.
//!
//! Pre-existing artifacts are never touched by default — they are reported
//! back as `kept`, feeding `/gov:plan`'s existing-artifact prompt ("keep
//! or replace?", default keep). Only an explicit `overwrite: true` — the
//! confirmed "replace" branch — copies fresh templates over them.
//! `data-model.md` joins the copy set only on request
//! (`include-data-model`; whether the feature has domain entities is the
//! host's judgment), but a pre-existing `data-model.md` is always reported
//! so the prompt sees the full artifact set.

use std::path::{Path, PathBuf};

use crate::primitives::apply_manifest::mirror_source_mode;
use crate::primitives::{PrimitiveError, Result, rel_path, resolve_template, write_atomic_bytes};
use crate::schema::paths;
use crate::schema::primitives::{
    CreatePlanArtifactsArgs, CreatePlanArtifactsResult, PlanArtifact, PlanArtifactAction,
};

/// Execute the `create-plan-artifacts` primitive against the given repo
/// root.
///
/// # Errors
///
/// Returns [`PrimitiveError::InvalidPath`] when `feature` is empty,
/// absolute, or carries a parent-directory component,
/// [`PrimitiveError::FeatureNotFound`] when the feature directory does not
/// exist, [`PrimitiveError::TemplateNotFound`] when any to-be-copied
/// artifact's template exists at neither candidate location (checked for
/// the whole copy set before the first write, so a missing template leaves
/// no partial scaffold), or [`PrimitiveError::Io`] for filesystem
/// failures. Pre-existing artifacts are the `kept` **domain outcome**,
/// never an error.
pub fn run(args: &CreatePlanArtifactsArgs, repo: &Path) -> Result<CreatePlanArtifactsResult> {
    super::validate_no_traversal(&args.feature)?;
    let root = paths::Paths::load(repo).specs_root;
    let feature_dir = repo.join(&root).join(&args.feature);
    if !feature_dir.is_dir() {
        return Err(PrimitiveError::FeatureNotFound {
            root,
            feature: args.feature.clone(),
        });
    }

    // Observe first, decide, then write: every artifact's existence is
    // read before any template resolution or copy.
    let requested: [(&str, bool); 3] = [
        ("plan.md", true),
        ("tasks.md", true),
        ("data-model.md", args.include_data_model),
    ];
    let mut pending: Vec<Pending> = Vec::with_capacity(requested.len());
    for (file, in_copy_set) in requested {
        let abs = feature_dir.join(file);
        let exists = abs.exists();
        // An absent data-model.md that was not requested is omitted from
        // the report entirely — there is nothing to say about it.
        if !exists && !in_copy_set {
            continue;
        }
        let action = if !exists {
            PlanArtifactAction::Created
        } else if in_copy_set && args.overwrite {
            PlanArtifactAction::Replaced
        } else {
            PlanArtifactAction::Kept
        };
        pending.push(Pending {
            file,
            abs,
            action,
            template: None,
        });
    }

    // Resolve every needed template before the first write, so a missing
    // template surfaces as one error with nothing half-copied. A call
    // where everything is kept resolves (and requires) no templates.
    for entry in &mut pending {
        if entry.action != PlanArtifactAction::Kept {
            entry.template = Some(resolve_template(repo, &root, entry.file)?);
        }
    }

    let mut artifacts = Vec::with_capacity(pending.len());
    for entry in pending {
        if let Some((_, template_abs)) = &entry.template {
            let bytes = std::fs::read(template_abs).map_err(|source| PrimitiveError::Io {
                path: template_abs.clone(),
                source,
            })?;
            write_atomic_bytes(&entry.abs, &bytes)?;
            mirror_source_mode(template_abs, &entry.abs)?;
        }
        artifacts.push(PlanArtifact {
            file: entry.file.to_string(),
            path: rel_path(&entry.abs, repo),
            action: entry.action,
            template: entry.template.map(|(rel, _)| rel),
        });
    }

    Ok(CreatePlanArtifactsResult {
        path: format!("{root}/{}", args.feature),
        artifacts,
    })
}

/// One artifact between the observe and write phases.
struct Pending {
    file: &'static str,
    abs: PathBuf,
    action: PlanArtifactAction,
    template: Option<(String, PathBuf)>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;
    use tempfile::tempdir;

    const PLAN_TEMPLATE: &str = "# Plan: {Feature}\n\n## Technical Decisions\n";
    const TASKS_TEMPLATE: &str = "# Tasks: {Feature}\n\n## 1. {Task}\n";
    const DATA_MODEL_TEMPLATE: &str = "# Data Model: {Feature}\n";

    fn seed(repo: &Path) {
        fs::create_dir_all(repo.join("specs/042-widget")).unwrap();
        fs::create_dir_all(repo.join("specs/templates")).unwrap();
        fs::write(repo.join("specs/templates/plan.md"), PLAN_TEMPLATE).unwrap();
        fs::write(repo.join("specs/templates/tasks.md"), TASKS_TEMPLATE).unwrap();
        fs::write(
            repo.join("specs/templates/data-model.md"),
            DATA_MODEL_TEMPLATE,
        )
        .unwrap();
    }

    fn args(feature: &str) -> CreatePlanArtifactsArgs {
        CreatePlanArtifactsArgs {
            feature: feature.into(),
            include_data_model: false,
            overwrite: false,
        }
    }

    fn artifact<'a>(result: &'a CreatePlanArtifactsResult, file: &str) -> &'a PlanArtifact {
        result
            .artifacts
            .iter()
            .find(|a| a.file == file)
            .unwrap_or_else(|| panic!("no {file} entry in {:?}", result.artifacts))
    }

    #[test]
    fn scaffolds_plan_and_tasks_from_templates() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        let result = run(&args("042-widget"), tmp.path()).unwrap();
        assert_eq!(result.path, "specs/042-widget");
        // Canonical order, data-model omitted (absent and not requested).
        assert_eq!(result.artifacts.len(), 2);
        assert_eq!(result.artifacts[0].file, "plan.md");
        assert_eq!(result.artifacts[1].file, "tasks.md");
        for entry in &result.artifacts {
            assert_eq!(entry.action, PlanArtifactAction::Created);
        }
        assert_eq!(
            artifact(&result, "plan.md").template.as_deref(),
            Some("specs/templates/plan.md")
        );
        let plan = fs::read_to_string(tmp.path().join("specs/042-widget/plan.md")).unwrap();
        assert_eq!(plan, PLAN_TEMPLATE, "plan.md is a byte copy");
        let tasks = fs::read_to_string(tmp.path().join("specs/042-widget/tasks.md")).unwrap();
        assert_eq!(tasks, TASKS_TEMPLATE, "tasks.md is a byte copy");
        assert!(!tmp.path().join("specs/042-widget/data-model.md").exists());
    }

    #[test]
    fn include_data_model_copies_the_third_artifact() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        let mut a = args("042-widget");
        a.include_data_model = true;
        let result = run(&a, tmp.path()).unwrap();
        assert_eq!(result.artifacts.len(), 3);
        assert_eq!(result.artifacts[2].file, "data-model.md");
        assert_eq!(result.artifacts[2].action, PlanArtifactAction::Created);
        let body = fs::read_to_string(tmp.path().join("specs/042-widget/data-model.md")).unwrap();
        assert_eq!(body, DATA_MODEL_TEMPLATE);
    }

    #[test]
    fn pre_existing_artifact_is_kept_untouched() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        let prior = "# Plan: widget\n\nhand-written work\n";
        fs::write(tmp.path().join("specs/042-widget/plan.md"), prior).unwrap();
        let result = run(&args("042-widget"), tmp.path()).unwrap();
        let plan = artifact(&result, "plan.md");
        assert_eq!(plan.action, PlanArtifactAction::Kept);
        assert!(plan.template.is_none(), "kept entries name no template");
        // The gap is still filled: tasks.md was missing and gets created.
        assert_eq!(
            artifact(&result, "tasks.md").action,
            PlanArtifactAction::Created
        );
        let body = fs::read_to_string(tmp.path().join("specs/042-widget/plan.md")).unwrap();
        assert_eq!(body, prior, "kept artifact is byte-identical");
    }

    #[test]
    fn overwrite_replaces_pre_existing_with_fresh_template() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        fs::write(tmp.path().join("specs/042-widget/plan.md"), "stale\n").unwrap();
        let mut a = args("042-widget");
        a.overwrite = true;
        let result = run(&a, tmp.path()).unwrap();
        let plan = artifact(&result, "plan.md");
        assert_eq!(plan.action, PlanArtifactAction::Replaced);
        assert_eq!(plan.template.as_deref(), Some("specs/templates/plan.md"));
        let body = fs::read_to_string(tmp.path().join("specs/042-widget/plan.md")).unwrap();
        assert_eq!(body, PLAN_TEMPLATE);
    }

    #[test]
    fn unrequested_data_model_is_reported_kept_and_never_touched() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        let prior = "# Data Model: widget\n\nentities\n";
        fs::write(tmp.path().join("specs/042-widget/data-model.md"), prior).unwrap();
        // Even the replace branch leaves an unrequested data-model alone.
        let mut a = args("042-widget");
        a.overwrite = true;
        let result = run(&a, tmp.path()).unwrap();
        let dm = artifact(&result, "data-model.md");
        assert_eq!(dm.action, PlanArtifactAction::Kept);
        let body = fs::read_to_string(tmp.path().join("specs/042-widget/data-model.md")).unwrap();
        assert_eq!(body, prior);
    }

    #[test]
    fn missing_template_errors_before_any_write() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        fs::remove_file(tmp.path().join("specs/templates/tasks.md")).unwrap();
        let err = run(&args("042-widget"), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::TemplateNotFound { .. }));
        assert!(
            !tmp.path().join("specs/042-widget/plan.md").exists(),
            "a missing tasks template must not leave a lone plan.md behind"
        );
    }

    #[test]
    fn all_kept_call_needs_no_templates() {
        let tmp = tempdir().unwrap();
        // No templates anywhere: with every artifact already on disk,
        // nothing needs copying and nothing errors.
        fs::create_dir_all(tmp.path().join("specs/042-widget")).unwrap();
        for file in ["plan.md", "tasks.md", "data-model.md"] {
            fs::write(tmp.path().join("specs/042-widget").join(file), "prior\n").unwrap();
        }
        let result = run(&args("042-widget"), tmp.path()).unwrap();
        assert_eq!(result.artifacts.len(), 3);
        for entry in &result.artifacts {
            assert_eq!(entry.action, PlanArtifactAction::Kept, "{}", entry.file);
        }
    }

    #[test]
    fn missing_feature_directory_errors() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        let err = run(&args("099-absent"), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::FeatureNotFound { .. }));
    }

    #[test]
    fn rejects_traversal_and_absolute_feature() {
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        for bad in ["../042-widget", "/etc", ""] {
            let err = run(&args(bad), tmp.path()).unwrap_err();
            assert!(
                matches!(err, PrimitiveError::InvalidPath { .. }),
                "expected InvalidPath for {bad:?}"
            );
        }
    }

    #[test]
    fn falls_back_to_framework_source_templates() {
        let tmp = tempdir().unwrap();
        fs::create_dir_all(tmp.path().join("specs/042-widget")).unwrap();
        fs::create_dir_all(tmp.path().join("framework/templates/spec")).unwrap();
        fs::write(
            tmp.path().join("framework/templates/spec/plan.md"),
            PLAN_TEMPLATE,
        )
        .unwrap();
        fs::write(
            tmp.path().join("framework/templates/spec/tasks.md"),
            TASKS_TEMPLATE,
        )
        .unwrap();
        let result = run(&args("042-widget"), tmp.path()).unwrap();
        assert_eq!(
            artifact(&result, "plan.md").template.as_deref(),
            Some("framework/templates/spec/plan.md")
        );
    }

    #[test]
    fn honors_configured_specs_root() {
        let tmp = tempdir().unwrap();
        fs::write(
            tmp.path().join(".govern.toml"),
            "[paths]\nspecs-root = \"governance\"\n",
        )
        .unwrap();
        fs::create_dir_all(tmp.path().join("governance/007-thing")).unwrap();
        fs::create_dir_all(tmp.path().join("governance/templates")).unwrap();
        fs::write(
            tmp.path().join("governance/templates/plan.md"),
            PLAN_TEMPLATE,
        )
        .unwrap();
        fs::write(
            tmp.path().join("governance/templates/tasks.md"),
            TASKS_TEMPLATE,
        )
        .unwrap();
        let result = run(&args("007-thing"), tmp.path()).unwrap();
        assert_eq!(result.path, "governance/007-thing");
        assert_eq!(
            artifact(&result, "plan.md").path,
            "governance/007-thing/plan.md"
        );
        assert!(tmp.path().join("governance/007-thing/tasks.md").is_file());
    }

    #[cfg(unix)]
    #[test]
    fn copied_artifacts_mirror_template_mode() {
        use std::os::unix::fs::PermissionsExt;
        let tmp = tempdir().unwrap();
        seed(tmp.path());
        let template = tmp.path().join("specs/templates/plan.md");
        let mut perms = fs::metadata(&template).unwrap().permissions();
        perms.set_mode(0o644);
        fs::set_permissions(&template, perms).unwrap();

        run(&args("042-widget"), tmp.path()).unwrap();
        let dest_mode = fs::metadata(tmp.path().join("specs/042-widget/plan.md"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(
            dest_mode, 0o644,
            "write_atomic_bytes lands 0600; the template mode must be mirrored"
        );
    }
}
