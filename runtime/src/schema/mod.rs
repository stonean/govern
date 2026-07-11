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
//!
//! Two crate-internal constant registries also live here as the single
//! source of truth for their respective closed sets: `registry` (primitive
//! names) and `status` (spec lifecycle statuses).

pub mod extensions;
pub mod paths;
pub mod primitives;
pub mod procedure;
pub mod protocol;
pub(crate) mod registry;
pub mod services;
pub(crate) mod status;
