//! Canonical spec lifecycle status sets.
//!
//! Single source of truth for the constitution's lifecycle set
//! (§text-first-artifacts): `validate-frontmatter` (membership findings),
//! `set-status` (from/to argument validation), `resolve-references`
//! (linked-spec status read), and `traverse-deps` (compatibility subset)
//! all consume these constants instead of hand-maintaining copies.

/// The constitution's lifecycle set, in pipeline order.
pub(crate) const ALLOWED_STATUSES: &[&str] =
    &["draft", "clarified", "planned", "in-progress", "done"];

/// Statuses a dependency may carry and still be compatible for consumers
/// (`traverse-deps`): the lifecycle tail from `planned` onward. `draft` and
/// `clarified` block dependents because there is no committed plan to build
/// against. Derived from [`ALLOWED_STATUSES`] so the subset cannot drift
/// from the canonical order.
pub(crate) const COMPATIBLE_STATUSES: &[&str] = ALLOWED_STATUSES.split_at(2).1;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compatible_statuses_are_the_planned_onward_tail() {
        assert_eq!(COMPATIBLE_STATUSES, &["planned", "in-progress", "done"]);
        assert!(
            COMPATIBLE_STATUSES
                .iter()
                .all(|s| ALLOWED_STATUSES.contains(s)),
            "compatibility subset must stay within the lifecycle set"
        );
    }
}
