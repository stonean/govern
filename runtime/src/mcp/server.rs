//! MCP server exposing every primitive as a tool.
//!
//! Naming convention is `gov-rt:<verb>-<noun>` per the resolved-questions
//! section of the spec. Tools are async wrappers around the synchronous
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
use rmcp::model::{ServerCapabilities, ServerInfo};
use rmcp::{ServerHandler, tool, tool_handler, tool_router};

use crate::primitives;
use crate::primitives::gate_confirm::GatePromptPayload;
use crate::schema::primitives::{
    CheckRuleIdsArgs, CheckRuleIdsResult, CheckStuckArgs, CheckStuckResult, CheckboxToggleResult,
    DeriveBoundaryArgs, DeriveBoundaryResult, GateConfirmArgs, LintMarkdownArgs,
    LintMarkdownResult, MarkCriterionArgs, MarkTaskArgs, ReadSpecArgs, ReadSpecResult,
    ReadTasksArgs, ReadTasksResult, ResolveAnchorArgs, ResolveAnchorResult, RunGeneratorArgs,
    RunGeneratorResult, SetStatusArgs, SetStatusResult, TraverseDepsArgs, TraverseDepsResult,
    ValidateFrontmatterArgs, ValidateFrontmatterResult,
};

/// Canonical MCP tool names exposed by the server, in manifest order.
pub const TOOL_NAMES: &[&str] = &[
    "gov-rt:read-spec",
    "gov-rt:read-tasks",
    "gov-rt:mark-task",
    "gov-rt:mark-criterion",
    "gov-rt:set-status",
    "gov-rt:derive-boundary",
    "gov-rt:check-stuck",
    "gov-rt:validate-frontmatter",
    "gov-rt:resolve-anchor",
    "gov-rt:traverse-deps",
    "gov-rt:check-rule-ids",
    "gov-rt:run-generator",
    "gov-rt:lint-markdown",
    "gov-rt:gate-confirm",
];

/// MCP server. Cloned per request by `rmcp`, so all state lives behind
/// `Arc` and tool dispatch is `&self`.
#[derive(Clone)]
pub struct GovRuntimeServer {
    repo: Arc<PathBuf>,
    tool_router: ToolRouter<Self>,
}

impl GovRuntimeServer {
    /// Construct a new server rooted at `repo` (the path every primitive
    /// resolves relative paths against).
    #[must_use]
    pub fn new(repo: PathBuf) -> Self {
        Self {
            repo: Arc::new(repo),
            tool_router: Self::tool_router(),
        }
    }

    fn repo(&self) -> &std::path::Path {
        self.repo.as_path()
    }
}

#[tool_router]
impl GovRuntimeServer {
    #[tool(
        name = "gov-rt:read-spec",
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
        name = "gov-rt:read-tasks",
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
        name = "gov-rt:mark-task",
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
        name = "gov-rt:mark-criterion",
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
        name = "gov-rt:set-status",
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
        name = "gov-rt:derive-boundary",
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
        name = "gov-rt:check-stuck",
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
        name = "gov-rt:validate-frontmatter",
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
        name = "gov-rt:resolve-anchor",
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
        name = "gov-rt:traverse-deps",
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
        name = "gov-rt:check-rule-ids",
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
        name = "gov-rt:run-generator",
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
        name = "gov-rt:lint-markdown",
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
        name = "gov-rt:gate-confirm",
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
}

#[tool_handler]
impl ServerHandler for GovRuntimeServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                "Deterministic runtime for the govern pipeline. Exposes per-primitive tools; \
                 see specs/022-deterministic-runtime/ for the protocol contract."
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
