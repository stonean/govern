//! Deterministic runtime for the `govern` pipeline.
//!
//! Implements the architecture described in
//! [`specs/022-deterministic-runtime/spec.md`]. The runtime exposes two
//! surfaces — an MCP server (`gvrn mcp`) and a subprocess interpreter
//! (`gvrn exec`) — sharing a common library of primitives.
//!
//! # Stability
//!
//! **This library is an implementation detail of the `gvrn` binary.**
//! The supported public surface is the CLI (`gvrn --help`) and the
//! JSON-over-stdio / MCP protocol it speaks. Linking against this crate
//! directly is **not supported** in v0.x: every module (`interpreter`,
//! `io`, `mcp`, `parser`, `primitives`, `schema`) may change shape
//! without a semver bump. Consume `gvrn` by installing the binary
//! (`cargo install gvrn`) and invoking it via stdio, or by running the
//! MCP server and connecting an MCP-capable host. A curated library
//! surface may land in a future major release; until then, treat the
//! items below as private to the runtime.

pub mod host;
pub mod interpreter;
pub mod io;
pub mod mcp;
pub mod parser;
pub mod primitives;
pub mod schema;
