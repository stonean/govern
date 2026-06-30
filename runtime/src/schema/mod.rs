//! Typed schemas for primitives, the JSON-over-stdio protocol, and
//! initial-release LLM extension points.
//!
//! Each submodule mirrors one type group from
//! [`specs/022-deterministic-runtime/data-model.md`]:
//!
//! - [`procedure`] — the AST emitted by the procedure parser.
//! - [`protocol`] — the JSON-over-stdio envelope and message types.
//! - [`primitives`] — per-primitive args/result shapes.
//! - [`extensions`] — the three initial-release extension-point payloads.
//! - [`services`] — the `[services]` registry shape from `.govern.toml`
//!   (spec 030 cross-service references).
//! - [`paths`] — the `[paths]` block shape from `.govern.toml`, resolving the
//!   configurable spec-root directory name (spec 040).

pub mod extensions;
pub mod paths;
pub mod primitives;
pub mod procedure;
pub mod protocol;
pub mod services;
