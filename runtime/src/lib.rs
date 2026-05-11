//! Deterministic runtime for the `govern` pipeline.
//!
//! Implements the architecture described in
//! [`specs/022-deterministic-runtime/spec.md`]. The runtime exposes two
//! surfaces — an MCP server (`runtime mcp`) and a subprocess interpreter
//! (`runtime exec`) — sharing a common library of primitives.

pub mod interpreter;
pub mod io;
pub mod mcp;
pub mod parser;
pub mod primitives;
pub mod schema;
