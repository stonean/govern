//! MCP server wiring built on `rmcp`.
//!
//! Exposes every primitive in [`crate::primitives`] as an MCP tool under
//! the `gov-rt:<verb>-<noun>` naming convention. Tool input schemas are
//! derived from each primitive's args struct via `schemars::JsonSchema`;
//! handlers delegate to the primitive's pure-Rust function and serialize
//! the result.

pub mod server;
