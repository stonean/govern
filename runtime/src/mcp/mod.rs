//! MCP server wiring built on `rmcp`.
//!
//! Exposes every primitive in [`crate::primitives`] as an MCP tool with
//! bare `<verb>-<noun>` names. Server-level namespacing comes from the
//! adopter's `.mcp.json` server registration (typically `gvrn`), so the
//! Claude Code-side wire identifier is `mcp__gvrn__<verb>-<noun>`. Tool
//! input schemas are derived from each primitive's args struct via
//! `schemars::JsonSchema`; handlers delegate to the primitive's pure-Rust
//! function and serialize the result.

pub mod server;
