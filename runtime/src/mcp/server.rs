//! MCP server exposing every primitive as a tool.
//!
//! Tool names are bare `<verb>-<noun>` strings (e.g. `read-spec`).
//! Server-level namespacing is supplied by the MCP server registration —
//! the adopter registers this server as `gvrn` in `.mcp.json`, which
//! makes the Claude Code-side wire identifier `mcp__gvrn__<verb>-<noun>`.
//! Tools are async wrappers around the synchronous
//! `primitives::<name>::run` functions; the server holds an `Arc<PathBuf>`
//! to the repo root that every primitive operates on.
//!
//! `gate-confirm` is special-cased: the MCP surface returns the prompt
//! payload (gate + prompt + fresh request-id) without blocking. The LLM
//! orchestrator is responsible for routing the prompt to the user. The
//! subprocess-interpreter surface (CLI `runtime gate-confirm`) handles
//! the blocking handshake via [`crate::primitives::gate_confirm::run_blocking`].

use std::path::PathBuf;
use std::sync::Arc;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{JsonObject, ServerCapabilities, ServerInfo};
use rmcp::{ServerHandler, tool, tool_handler, tool_router};
use serde_json::Value;

use crate::primitives;
use crate::primitives::gate_confirm::GatePromptPayload;
use crate::schema::primitives::{
    AppendTaskArgs, AppendTaskResult, ApplyManifestArgs, ApplyManifestResult, CheckRuleIdsArgs,
    CheckRuleIdsResult, CheckStuckArgs, CheckStuckResult, CheckboxToggleResult, CreateScenarioArgs,
    CreateScenarioResult, DashboardArgs, DashboardResult, DeriveBoundaryArgs, DeriveBoundaryResult,
    DiscoverRuleFilesArgs, DiscoverRuleFilesResult, EnforceManifestArgs, EnforceManifestResult,
    ExtractArchiveArgs, ExtractArchiveResult, FetchArchiveArgs, FetchArchiveResult,
    GateConfirmArgs, LintMarkdownArgs, LintMarkdownResult, MarkCriterionArgs, MarkTaskArgs,
    MergeClaudeMdArgs, MergeClaudeMdResult, MergeManagedBlockArgs, MergeManagedBlockResult,
    MergePermissionsArgs, MergePermissionsResult, MigrateSessionFileArgs, MigrateSessionFileResult,
    ProcessWaiversArgs, ProcessWaiversResult, ReadSpecArgs, ReadSpecResult, ReadTasksArgs,
    ReadTasksResult, ResolveAnchorArgs, ResolveAnchorResult, ResolveReferencesArgs,
    ResolveReferencesResult, RunGeneratorArgs, RunGeneratorResult, SetStatusArgs, SetStatusResult,
    SubstituteTemplatesArgs, SubstituteTemplatesResult, TraverseDepsArgs, TraverseDepsResult,
    ValidateFrontmatterArgs, ValidateFrontmatterResult, WriteSessionArgs, WriteSessionResult,
};

/// Canonical MCP tool names exposed by the server, in manifest order.
pub const TOOL_NAMES: &[&str] = &[
    "read-spec",
    "read-tasks",
    "mark-task",
    "mark-criterion",
    "set-status",
    "derive-boundary",
    "discover-rule-files",
    "process-waivers",
    "check-stuck",
    "validate-frontmatter",
    "resolve-anchor",
    "traverse-deps",
    "check-rule-ids",
    "run-generator",
    "lint-markdown",
    "gate-confirm",
    "fetch-archive",
    "extract-archive",
    "substitute-templates",
    "merge-claude-md",
    "apply-manifest",
    "enforce-manifest",
    "merge-managed-block",
    "merge-permissions",
    "migrate-session-file",
    "create-scenario",
    "append-task",
    "dashboard",
    "write-session",
    "resolve-references",
];

/// MCP server. Cloned per request by `rmcp`, so all state lives behind
/// `Arc` and tool dispatch is `&self`.
#[derive(Clone)]
pub struct GovRuntimeServer {
    repo: Arc<PathBuf>,
    // Dispatch router the `#[tool_handler(router = self.tool_router)]` impl
    // reads for both `list_tools` and `call_tool`. Built once in `new()` and
    // post-processed by `strip_nonstandard_formats` so every served schema is
    // sanitized. The handler must point here rather than at its default
    // `Self::tool_router()`, which would regenerate a fresh, unsanitized
    // router on every call.
    tool_router: ToolRouter<Self>,
}

impl GovRuntimeServer {
    /// Construct a new server rooted at `repo` (the path every primitive
    /// resolves relative paths against).
    #[must_use]
    pub fn new(repo: PathBuf) -> Self {
        let mut tool_router = Self::tool_router();
        strip_nonstandard_formats(&mut tool_router);
        Self {
            repo: Arc::new(repo),
            tool_router,
        }
    }

    fn repo(&self) -> &std::path::Path {
        self.repo.as_path()
    }
}

/// Strip non-standard numeric `format` hints from every tool schema.
///
/// `schemars` stamps OpenAPI-style `format` values (`uint32`, `uint64`,
/// `uint8`, …) onto every Rust integer/float field. JSON Schema defines
/// no `format` values for numeric types, so strict MCP clients (opencode
/// and other Ajv-based validators) log `unknown format "uint32" ignored`
/// for each occurrence in a tool's input/output schema. The hints carry no
/// validation meaning for our consumers, so we drop them at construction —
/// leaving string formats (`date-time`, `uri`, `uuid`, …) untouched.
fn strip_nonstandard_formats(router: &mut ToolRouter<GovRuntimeServer>) {
    for route in router.map.values_mut() {
        route.attr.input_schema = sanitized_schema(&route.attr.input_schema);
        if let Some(output) = route.attr.output_schema.as_ref() {
            route.attr.output_schema = Some(sanitized_schema(output));
        }
    }
}

/// Clone a tool schema, drop each numeric node's `format`, and re-wrap it.
fn sanitized_schema(schema: &Arc<JsonObject>) -> Arc<JsonObject> {
    let mut value = Value::Object(schema.as_ref().clone());
    strip_numeric_formats(&mut value);
    let Value::Object(map) = value else {
        unreachable!("a JSON Schema root is always an object");
    };
    Arc::new(map)
}

/// Recursively remove `format` from any node typed `integer` or `number`.
fn strip_numeric_formats(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if node_is_numeric(map) {
                map.remove("format");
            }
            for child in map.values_mut() {
                strip_numeric_formats(child);
            }
        }
        Value::Array(items) => items.iter_mut().for_each(strip_numeric_formats),
        _ => {}
    }
}

/// A schema node is numeric when its `type` is (or includes) `integer` or
/// `number` — the two JSON Schema types that admit no standard `format`.
fn node_is_numeric(map: &JsonObject) -> bool {
    fn is_numeric_type(ty: &Value) -> bool {
        matches!(ty.as_str(), Some("integer" | "number"))
    }
    match map.get("type") {
        Some(ty @ Value::String(_)) => is_numeric_type(ty),
        Some(Value::Array(types)) => types.iter().any(is_numeric_type),
        _ => false,
    }
}

#[tool_router]
impl GovRuntimeServer {
    #[tool(
        name = "read-spec",
        description = "Parse spec frontmatter and body sections."
    )]
    async fn read_spec(
        &self,
        params: Parameters<ReadSpecArgs>,
    ) -> Result<Json<ReadSpecResult>, String> {
        primitives::read_spec::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "read-tasks",
        description = "Parse `tasks.md` into a structured task list."
    )]
    async fn read_tasks(
        &self,
        params: Parameters<ReadTasksArgs>,
    ) -> Result<Json<ReadTasksResult>, String> {
        primitives::read_tasks::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "mark-task",
        description = "Flip a single subtask checkbox in `tasks.md` (atomic rewrite)."
    )]
    async fn mark_task(
        &self,
        params: Parameters<MarkTaskArgs>,
    ) -> Result<Json<CheckboxToggleResult>, String> {
        primitives::mark_task::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "mark-criterion",
        description = "Flip a single acceptance-criterion checkbox in `spec.md`."
    )]
    async fn mark_criterion(
        &self,
        params: Parameters<MarkCriterionArgs>,
    ) -> Result<Json<CheckboxToggleResult>, String> {
        primitives::mark_criterion::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "set-status",
        description = "Update the `status:` field in spec frontmatter, guarded by `from`."
    )]
    async fn set_status(
        &self,
        params: Parameters<SetStatusArgs>,
    ) -> Result<Json<SetStatusResult>, String> {
        primitives::set_status::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "derive-boundary",
        description = "Derive the runtime write boundary from git history."
    )]
    async fn derive_boundary(
        &self,
        params: Parameters<DeriveBoundaryArgs>,
    ) -> Result<Json<DeriveBoundaryResult>, String> {
        primitives::derive_boundary::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "check-stuck",
        description = "Count `tasks.md` commits since status entered `in-progress`."
    )]
    async fn check_stuck(
        &self,
        params: Parameters<CheckStuckArgs>,
    ) -> Result<Json<CheckStuckResult>, String> {
        primitives::check_stuck::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "validate-frontmatter",
        description = "Validate frontmatter shape against the pipeline schema."
    )]
    async fn validate_frontmatter(
        &self,
        params: Parameters<ValidateFrontmatterArgs>,
    ) -> Result<Json<ValidateFrontmatterResult>, String> {
        primitives::validate_frontmatter::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "resolve-anchor",
        description = "Verify `§anchor` references resolve to `<!-- §anchor -->` markers."
    )]
    async fn resolve_anchor(
        &self,
        params: Parameters<ResolveAnchorArgs>,
    ) -> Result<Json<ResolveAnchorResult>, String> {
        primitives::resolve_anchor::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "traverse-deps",
        description = "Traverse spec dependencies and check status compatibility."
    )]
    async fn traverse_deps(
        &self,
        params: Parameters<TraverseDepsArgs>,
    ) -> Result<Json<TraverseDepsResult>, String> {
        primitives::traverse_deps::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "check-rule-ids",
        description = "Verify cited rule IDs exist in rule files and aren't deprecated."
    )]
    async fn check_rule_ids(
        &self,
        params: Parameters<CheckRuleIdsArgs>,
    ) -> Result<Json<CheckRuleIdsResult>, String> {
        primitives::check_rule_ids::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "run-generator",
        description = "Invoke a bash generator with `--dry-run`; non-zero exit is drift."
    )]
    async fn run_generator(
        &self,
        params: Parameters<RunGeneratorArgs>,
    ) -> Result<Json<RunGeneratorResult>, String> {
        primitives::run_generator::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "lint-markdown",
        description = "Wrap `npx markdownlint-cli2` and surface violations."
    )]
    async fn lint_markdown(
        &self,
        params: Parameters<LintMarkdownArgs>,
    ) -> Result<Json<LintMarkdownResult>, String> {
        primitives::lint_markdown::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "gate-confirm",
        description = "Return the gate prompt payload (non-blocking). The orchestrator routes the prompt to the user and supplies the confirmed decision out-of-band; this tool never blocks waiting for input."
    )]
    async fn gate_confirm(
        &self,
        params: Parameters<GateConfirmArgs>,
    ) -> Result<Json<GatePromptPayload>, String> {
        let request_id = primitives::gate_confirm::fresh_request_id();
        Ok(Json(primitives::gate_confirm::prompt_payload(
            &params.0,
            &request_id,
        )))
    }

    #[tool(
        name = "fetch-archive",
        description = "Download an archive plus its sha256 sidecar and verify the hash."
    )]
    async fn fetch_archive(
        &self,
        params: Parameters<FetchArchiveArgs>,
    ) -> Result<Json<FetchArchiveResult>, String> {
        primitives::fetch_archive::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "extract-archive",
        description = "Extract a local `.tar.gz` / `.zip` archive into a destination directory."
    )]
    async fn extract_archive(
        &self,
        params: Parameters<ExtractArchiveArgs>,
    ) -> Result<Json<ExtractArchiveResult>, String> {
        primitives::extract_archive::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "substitute-templates",
        description = "Walk a source tree, apply `{key}` substitutions, and write to a destination."
    )]
    async fn substitute_templates(
        &self,
        params: Parameters<SubstituteTemplatesArgs>,
    ) -> Result<Json<SubstituteTemplatesResult>, String> {
        primitives::substitute_templates::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "merge-claude-md",
        description = "Idempotently merge a framework-managed block into the adopter's CLAUDE.md."
    )]
    async fn merge_claude_md(
        &self,
        params: Parameters<MergeClaudeMdArgs>,
    ) -> Result<Json<MergeClaudeMdResult>, String> {
        primitives::merge_claude_md::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "apply-manifest",
        description = "Strategy-aware bulk substitute + write driven by a manifest."
    )]
    async fn apply_manifest(
        &self,
        params: Parameters<ApplyManifestArgs>,
    ) -> Result<Json<ApplyManifestResult>, String> {
        primitives::apply_manifest::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "enforce-manifest",
        description = "Remove files in a directory that are not in the expected manifest."
    )]
    async fn enforce_manifest(
        &self,
        params: Parameters<EnforceManifestArgs>,
    ) -> Result<Json<EnforceManifestResult>, String> {
        primitives::enforce_manifest::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "merge-managed-block",
        description = "Idempotently merge a framework-managed block with configurable marker shape."
    )]
    async fn merge_managed_block(
        &self,
        params: Parameters<MergeManagedBlockArgs>,
    ) -> Result<Json<MergeManagedBlockResult>, String> {
        primitives::merge_managed_block::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "merge-permissions",
        description = "Idempotently merge a canonical permission allow/deny set into a JSON file with exact-match dedup."
    )]
    async fn merge_permissions(
        &self,
        params: Parameters<MergePermissionsArgs>,
    ) -> Result<Json<MergePermissionsResult>, String> {
        primitives::merge_permissions::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "migrate-session-file",
        description = "Translate a pre-0.10.0 legacy session JSON (`.claude/{project}-session.json`) into the consolidated `.govern.session.toml` at the repo root, applying camelCase→kebab-case key renames (`scenarioPath`→`scenario-path`, `setAt`→`set-at`), and delete the legacy file. Idempotent: no-op when no legacy file is present; preserves any existing `.govern.session.toml`."
    )]
    async fn migrate_session_file(
        &self,
        params: Parameters<MigrateSessionFileArgs>,
    ) -> Result<Json<MigrateSessionFileResult>, String> {
        primitives::migrate_session_file::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "create-scenario",
        description = "Write a new scenarios/{slug}.md file under a feature with frontmatter and prose body."
    )]
    async fn create_scenario(
        &self,
        params: Parameters<CreateScenarioArgs>,
    ) -> Result<Json<CreateScenarioResult>, String> {
        primitives::create_scenario::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "discover-rule-files",
        description = "Select rule files for /gov:review by suffix, [rules] surfaces, and disabled-rule-files."
    )]
    async fn discover_rule_files(
        &self,
        params: Parameters<DiscoverRuleFilesArgs>,
    ) -> Result<Json<DiscoverRuleFilesResult>, String> {
        primitives::discover_rule_files::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "process-waivers",
        description = "Classify a spec's review.waivers against currently-firing findings (apply/expire/dedup)."
    )]
    async fn process_waivers(
        &self,
        params: Parameters<ProcessWaiversArgs>,
    ) -> Result<Json<ProcessWaiversResult>, String> {
        primitives::process_waivers::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "append-task",
        description = "Append a numbered task block to a feature's tasks.md (atomic rewrite)."
    )]
    async fn append_task(
        &self,
        params: Parameters<AppendTaskArgs>,
    ) -> Result<Json<AppendTaskResult>, String> {
        primitives::append_task::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "dashboard",
        description = "Single-call pipeline-state surface for /{project}:status. Returns the per-spec inventory (status, deps, tags, open-question count, artifact existence, scenarios count, blocked-by), the repo-wide tags-union, the .govern.toml review-state summary, and the optional session target read from .govern.session.toml."
    )]
    async fn dashboard(
        &self,
        params: Parameters<DashboardArgs>,
    ) -> Result<Json<DashboardResult>, String> {
        primitives::dashboard::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "write-session",
        description = "Atomically merge-write `.govern.session.toml` (repo root, gitignored, single path for every adopter). A target write (supply `feature`+`path`, optional `scenario`) sets the target and preserves the per-contributor `cli-config-dir`; a host-config write (supply only `cli-config-dir`) sets the agent config-dir and preserves the existing target. Pairs with `dashboard`'s read of the same file; allowing this MCP tool once suppresses the per-invocation Write permission prompt the host-write path triggers."
    )]
    async fn write_session(
        &self,
        params: Parameters<WriteSessionArgs>,
    ) -> Result<Json<WriteSessionResult>, String> {
        primitives::write_session::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }

    #[tool(
        name = "resolve-references",
        description = "Resolve a consumer feature's `references:` index against `.govern.toml` [services]: for each cross-service reference, read the linked spec's live lifecycle status from its local checkout and classify the outcome (ok / unregistered / not-checked-out / broken / status-unreadable)."
    )]
    async fn resolve_references(
        &self,
        params: Parameters<ResolveReferencesArgs>,
    ) -> Result<Json<ResolveReferencesResult>, String> {
        primitives::resolve_references::run(&params.0, self.repo())
            .map(Json)
            .map_err(|e| e.to_string())
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for GovRuntimeServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build()).with_instructions(
            "Deterministic runtime for the govern pipeline. Exposes per-primitive tools; \
                 see specs/022-deterministic-runtime/ for the protocol contract.",
        )
    }
}
