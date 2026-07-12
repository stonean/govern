//! Canonical primitive-name registry.
//!
//! Single source of truth for the runtime's primitive names. The parser's
//! `PRIMITIVE_NAMES` and the MCP server's `TOOL_NAMES` are both defined
//! from [`PRIMITIVE_REGISTRY`]; `framework/runtime-tools.txt` (the shipped
//! manifest) is asserted set-equal in `runtime/tests/mcp.rs`, and the
//! interpreter's `dispatch_primitive` is asserted to handle every registry
//! name in `interpreter::tests`. The two remaining hand-written surfaces —
//! the rmcp `#[tool]` methods and the clap subcommand enum in `main.rs` —
//! cannot consume a const slice, so the tests above pin them instead of a
//! macro.

/// Every primitive name exposed by the runtime, in manifest order. Names
/// are bare `<verb>-<noun>` strings; server-level namespacing (`gvrn`) is
/// supplied by the host's MCP registration.
pub(crate) const PRIMITIVE_REGISTRY: &[&str] = &[
    "read-spec",
    "read-tasks",
    "mark-task",
    "mark-criterion",
    "set-status",
    "derive-boundary",
    "discover-rule-files",
    "process-waivers",
    "compute-review-scope",
    "write-review",
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
    "prune-tasks",
    "dashboard",
    "write-session",
    "resolve-references",
    "resolve-feature",
    "create-feature",
    "create-plan-artifacts",
    "check-review-gate",
    "append-inbox",
    "remove-inbox-item",
    "check-artifacts",
];
