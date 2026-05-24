//! `merge-permissions` — idempotently install or update a canonical
//! permission allow/deny set into a JSON file, removing exact-match
//! duplicates from each array. The destination path is host-supplied
//! (typically the bootstrap-substituted `{cli-config-dir}/settings.local.json`,
//! e.g. `.claude/settings.local.json` on Claude Code or
//! `.augment/settings.json` on Auggie); no default — `path` is required.
//!
//! The primitive is the deterministic surface `/configure` calls; see
//! spec 022's `framework-list-dedup` scenario for the contract.
//!
//! Behavior summary:
//!
//! - **File does not exist** → write
//!   `{ "permissions": { "allow": [...canonical], "deny": [...canonical] } }`
//!   and emit `created`.
//! - **File exists, parses as JSON** → dedup exact-match entries in
//!   `permissions.allow` and `permissions.deny`, then ensure every
//!   canonical entry is present (append at end, preserving prior
//!   order). If the post-merge value equals the pre-merge value
//!   structurally, emit `unchanged` and skip the write (preserves
//!   mtime for build-tool idempotency).
//! - **File exists, malformed JSON** → return
//!   [`PrimitiveError::Json`]; do not write.
//! - **`permissions.allow` / `permissions.deny` field exists but is
//!   not an array** → return [`PrimitiveError::JsonSchema`]; do not
//!   write.
//!
//! Atomic writes use the project-wide tempfile + rename helper. Field
//! order under `permissions` follows insertion order via
//! `serde_json`'s `preserve_order` feature.

#![allow(clippy::expect_used)]

use std::path::Path;

use serde_json::{Map, Value, json};

use crate::primitives::{PrimitiveError, Result, read_text, write_atomic};
use crate::schema::primitives::{MergePermissionsArgs, MergePermissionsResult};

/// Execute the `merge-permissions` primitive.
///
/// # Errors
///
/// - [`PrimitiveError::Io`] on local filesystem failures.
/// - [`PrimitiveError::Json`] when an existing file fails JSON parse.
/// - [`PrimitiveError::JsonSchema`] when `permissions.allow` /
///   `permissions.deny` exists but is not an array (e.g., null,
///   object, string).
pub fn run(args: &MergePermissionsArgs, repo: &Path) -> Result<MergePermissionsResult> {
    let candidate = Path::new(&args.path);
    let target_path = if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo.join(candidate)
    };

    let existing = match target_path.try_exists() {
        Ok(true) => Some(read_text(&target_path)?),
        Ok(false) => None,
        Err(source) => {
            return Err(PrimitiveError::Io {
                path: target_path.clone(),
                source,
            });
        }
    };

    let MergeOutcome {
        post_value,
        action,
        allow_added,
        allow_deduped,
        deny_added,
        deny_deduped,
    } = compute_merge(existing.as_deref(), &args.allow, &args.deny, &target_path)?;

    if action != "unchanged" {
        let serialized = serialize_pretty(&post_value);
        write_atomic(&target_path, &serialized)?;
    }

    Ok(MergePermissionsResult {
        path: target_path.to_string_lossy().into_owned(),
        action: action.into(),
        allow_added,
        allow_deduped,
        deny_added,
        deny_deduped,
    })
}

/// Pretty-print with 2-space indent and a trailing newline. The
/// `serde_json` `preserve_order` feature keeps `Map`'s insertion
/// order intact across serialization. `to_string_pretty` is
/// infallible on `serde_json::Value` (no non-string keys, no I/O),
/// so the `.expect` documents the invariant rather than handling a
/// reachable failure mode.
fn serialize_pretty(value: &Value) -> String {
    let mut out =
        serde_json::to_string_pretty(value).expect("serde_json::Value serializes infallibly");
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out
}

struct MergeOutcome {
    post_value: Value,
    action: &'static str,
    allow_added: u32,
    allow_deduped: u32,
    deny_added: u32,
    deny_deduped: u32,
}

fn compute_merge(
    existing: Option<&str>,
    canonical_allow: &[String],
    canonical_deny: &[String],
    path: &Path,
) -> Result<MergeOutcome> {
    match existing {
        None => Ok(fresh_merge(canonical_allow, canonical_deny)),
        Some(text) => existing_merge(text, canonical_allow, canonical_deny, path),
    }
}

fn fresh_merge(canonical_allow: &[String], canonical_deny: &[String]) -> MergeOutcome {
    let allow_added = u32::try_from(canonical_allow.len()).unwrap_or(u32::MAX);
    let deny_added = u32::try_from(canonical_deny.len()).unwrap_or(u32::MAX);
    let post_value = json!({
        "permissions": {
            "allow": canonical_allow,
            "deny": canonical_deny,
        }
    });
    MergeOutcome {
        post_value,
        action: "created",
        allow_added,
        allow_deduped: 0,
        deny_added,
        deny_deduped: 0,
    }
}

fn existing_merge(
    text: &str,
    canonical_allow: &[String],
    canonical_deny: &[String],
    path: &Path,
) -> Result<MergeOutcome> {
    let original: Value = serde_json::from_str(text).map_err(|source| PrimitiveError::Json {
        path: path.into(),
        source,
    })?;

    let mut post_value = original.clone();
    let permissions = ensure_permissions_object(&mut post_value, path)?;

    let (allow_added, allow_deduped) = merge_array(permissions, "allow", canonical_allow, path)?;
    let (deny_added, deny_deduped) = merge_array(permissions, "deny", canonical_deny, path)?;

    let action = if post_value == original {
        "unchanged"
    } else {
        "updated"
    };

    Ok(MergeOutcome {
        post_value,
        action,
        allow_added,
        allow_deduped,
        deny_added,
        deny_deduped,
    })
}

/// Ensure the top-level value has a `permissions` object, returning a
/// mutable reference to its `Map`. If the top-level value is not an
/// object, return a schema error rather than silently overwriting.
fn ensure_permissions_object<'a>(
    value: &'a mut Value,
    path: &Path,
) -> Result<&'a mut Map<String, Value>> {
    let Some(root) = value.as_object_mut() else {
        return Err(PrimitiveError::JsonSchema {
            path: path.into(),
            reason: "top-level value is not a JSON object".into(),
        });
    };
    let permissions = root.entry("permissions").or_insert_with(|| json!({}));
    match permissions.as_object_mut() {
        Some(map) => Ok(map),
        None => Err(PrimitiveError::JsonSchema {
            path: path.into(),
            reason: "`permissions` field exists but is not a JSON object".into(),
        }),
    }
}

/// Apply the dedup + canonical-presence passes to one array field on
/// `permissions`. Returns `(added, deduped)` counts. `field` is
/// `"allow"` or `"deny"`.
fn merge_array(
    permissions: &mut Map<String, Value>,
    field: &str,
    canonical: &[String],
    path: &Path,
) -> Result<(u32, u32)> {
    let Some(array_value) = permissions.get_mut(field) else {
        permissions.insert(
            field.into(),
            Value::Array(canonical.iter().map(|s| Value::String(s.clone())).collect()),
        );
        let added = u32::try_from(canonical.len()).unwrap_or(u32::MAX);
        return Ok((added, 0));
    };

    let Some(arr) = array_value.as_array_mut() else {
        return Err(PrimitiveError::JsonSchema {
            path: path.into(),
            reason: format!("`permissions.{field}` exists but is not an array"),
        });
    };

    // Dedup pass: first occurrence wins; later duplicates removed in place.
    let mut seen: Vec<String> = Vec::with_capacity(arr.len());
    let mut deduped = 0u32;
    let mut idx = 0;
    while idx < arr.len() {
        if let Some(s) = arr[idx].as_str() {
            let s_owned = s.to_string();
            if seen.contains(&s_owned) {
                arr.remove(idx);
                deduped = deduped.saturating_add(1);
                continue;
            }
            seen.push(s_owned);
        }
        // Non-string entries are preserved verbatim and not considered
        // for dedup. The canonical set is string-valued; non-string
        // entries don't collide.
        idx += 1;
    }

    // Canonical-presence pass: append any canonical entry not already
    // present (by string-equality), preserving canonical-set order.
    let mut added = 0u32;
    for entry in canonical {
        if !seen.contains(entry) {
            arr.push(Value::String(entry.clone()));
            seen.push(entry.clone());
            added = added.saturating_add(1);
        }
    }

    Ok((added, deduped))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;

    fn args(path: &str, allow: &[&str], deny: &[&str]) -> MergePermissionsArgs {
        MergePermissionsArgs {
            path: path.to_string(),
            allow: allow.iter().map(|s| (*s).to_string()).collect(),
            deny: deny.iter().map(|s| (*s).to_string()).collect(),
        }
    }

    #[test]
    fn creates_file_when_absent_with_canonical_set() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join(".claude/settings.local.json");
        let result = run(
            &args(
                ".claude/settings.local.json",
                &["Edit", "Bash(ls *)"],
                &["Bash(rm -rf *)"],
            ),
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.action, "created");
        assert_eq!(result.allow_added, 2);
        assert_eq!(result.deny_added, 1);
        assert_eq!(result.allow_deduped, 0);
        assert_eq!(result.deny_deduped, 0);

        let body = fs::read_to_string(&path).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed["permissions"]["allow"][0], "Edit");
        assert_eq!(parsed["permissions"]["allow"][1], "Bash(ls *)");
        assert_eq!(parsed["permissions"]["deny"][0], "Bash(rm -rf *)");
    }

    #[test]
    fn writes_to_host_supplied_path_for_non_claude_adopter() {
        // Auggie keeps its permissions at `.augment/settings.json`; the
        // primitive writes wherever the caller says without baking in a
        // Claude-shaped default.
        let tmp = tempfile::tempdir().unwrap();
        let result = run(&args(".augment/settings.json", &["Edit"], &[]), tmp.path()).unwrap();
        assert!(result.path.ends_with(".augment/settings.json"));
        assert_eq!(result.action, "created");
        assert!(tmp.path().join(".augment/settings.json").is_file());
        assert!(
            !tmp.path().join(".claude").exists(),
            "Auggie write must not create a Claude-shaped sibling"
        );
    }

    #[test]
    fn dedups_existing_duplicates() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("settings.json");
        fs::write(
            &path,
            r#"{"permissions":{"allow":["Edit","Bash(ls *)","Edit","Write","Edit"],"deny":[]}}"#,
        )
        .unwrap();
        let result = run(
            &args("settings.json", &["Edit", "Bash(ls *)"], &[]),
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.action, "updated");
        assert_eq!(result.allow_added, 0);
        assert_eq!(
            result.allow_deduped, 2,
            "two extra 'Edit' entries should be removed"
        );

        let body = fs::read_to_string(&path).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();
        let allow = parsed["permissions"]["allow"].as_array().unwrap();
        assert_eq!(allow.len(), 3, "Edit + Bash(ls *) + Write after dedup");
        assert_eq!(allow[0], "Edit", "first occurrence wins");
        assert_eq!(allow[1], "Bash(ls *)");
        assert_eq!(allow[2], "Write");
    }

    #[test]
    fn dedup_includes_non_canonical_entries() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("s.json");
        fs::write(
            &path,
            r#"{"permissions":{"allow":["UserAdded","UserAdded","Other","UserAdded"],"deny":[]}}"#,
        )
        .unwrap();
        let result = run(&args("s.json", &[], &[]), tmp.path()).unwrap();
        assert_eq!(result.allow_deduped, 2);
        assert_eq!(result.allow_added, 0);
        let body = fs::read_to_string(&path).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();
        let allow = parsed["permissions"]["allow"].as_array().unwrap();
        assert_eq!(allow.len(), 2);
        assert_eq!(allow[0], "UserAdded");
        assert_eq!(allow[1], "Other");
    }

    #[test]
    fn appends_missing_canonical_at_end_preserving_order() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("s.json");
        fs::write(
            &path,
            r#"{"permissions":{"allow":["UserA","UserB"],"deny":[]}}"#,
        )
        .unwrap();
        let result = run(
            &args("s.json", &["Canonical1", "Canonical2"], &[]),
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.action, "updated");
        assert_eq!(result.allow_added, 2);
        assert_eq!(result.allow_deduped, 0);

        let body = fs::read_to_string(&path).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();
        let allow = parsed["permissions"]["allow"].as_array().unwrap();
        assert_eq!(
            allow
                .iter()
                .map(|v| v.as_str().unwrap())
                .collect::<Vec<_>>(),
            vec!["UserA", "UserB", "Canonical1", "Canonical2"]
        );
    }

    #[test]
    fn canonical_present_is_not_re_appended() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("s.json");
        fs::write(
            &path,
            r#"{"permissions":{"allow":["Canonical1","UserA"],"deny":[]}}"#,
        )
        .unwrap();
        let result = run(
            &args("s.json", &["Canonical1", "Canonical2"], &[]),
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.allow_added, 1, "only Canonical2 was missing");
        assert_eq!(result.allow_deduped, 0);
    }

    #[test]
    fn unchanged_when_canonical_present_and_no_duplicates() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("s.json");
        let original = r#"{
  "permissions": {
    "allow": [
      "Edit",
      "Bash(ls *)"
    ],
    "deny": [
      "Bash(rm -rf *)"
    ]
  }
}
"#;
        fs::write(&path, original).unwrap();
        let mtime_before = fs::metadata(&path).unwrap().modified().unwrap();

        let result = run(
            &args("s.json", &["Edit", "Bash(ls *)"], &["Bash(rm -rf *)"]),
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.action, "unchanged");
        assert_eq!(result.allow_added, 0);
        assert_eq!(result.allow_deduped, 0);

        // mtime preserved (no write happened).
        assert_eq!(
            fs::metadata(&path).unwrap().modified().unwrap(),
            mtime_before
        );

        // File content untouched byte-for-byte.
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
    }

    #[test]
    fn preserves_untouched_top_level_fields() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("s.json");
        fs::write(
            &path,
            r#"{
  "permissions": {
    "allow": ["UserA"],
    "deny": [],
    "additionalDirectories": ["/foo", "/bar"]
  },
  "defaultMode": "default",
  "customField": {"nested": "value"}
}"#,
        )
        .unwrap();
        let result = run(&args("s.json", &["Canonical1"], &[]), tmp.path()).unwrap();
        assert_eq!(result.action, "updated");

        let body = fs::read_to_string(&path).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed["defaultMode"], "default");
        assert_eq!(parsed["customField"]["nested"], "value");
        assert_eq!(parsed["permissions"]["additionalDirectories"][0], "/foo");
        assert_eq!(parsed["permissions"]["additionalDirectories"][1], "/bar");
    }

    #[test]
    fn missing_permissions_object_is_added() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("s.json");
        fs::write(&path, r#"{"defaultMode": "default"}"#).unwrap();
        let result = run(&args("s.json", &["Edit"], &["Bash(rm -rf *)"]), tmp.path()).unwrap();
        assert_eq!(result.action, "updated");

        let body = fs::read_to_string(&path).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed["defaultMode"], "default");
        assert_eq!(parsed["permissions"]["allow"][0], "Edit");
        assert_eq!(parsed["permissions"]["deny"][0], "Bash(rm -rf *)");
    }

    #[test]
    fn missing_allow_array_is_seeded() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("s.json");
        fs::write(&path, r#"{"permissions":{"deny":["Existing"]}}"#).unwrap();
        let result = run(&args("s.json", &["Edit"], &[]), tmp.path()).unwrap();
        assert_eq!(result.action, "updated");
        assert_eq!(result.allow_added, 1);

        let body = fs::read_to_string(&path).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed["permissions"]["allow"][0], "Edit");
        assert_eq!(parsed["permissions"]["deny"][0], "Existing");
    }

    #[test]
    fn malformed_json_returns_parse_error() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("s.json");
        fs::write(&path, r#"{"permissions": {"allow": [oops}"#).unwrap();
        let err = run(&args("s.json", &["Edit"], &[]), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::Json { .. }));
        // File should be unchanged.
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("oops"));
    }

    #[test]
    fn non_array_allow_returns_schema_error() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("s.json");
        fs::write(
            &path,
            r#"{"permissions": {"allow": "not-an-array", "deny": []}}"#,
        )
        .unwrap();
        let err = run(&args("s.json", &["Edit"], &[]), tmp.path()).unwrap_err();
        match err {
            PrimitiveError::JsonSchema { reason, .. } => {
                assert!(reason.contains("permissions.allow"));
                assert!(reason.contains("not an array"));
            }
            other => panic!("expected JsonSchema, got {other:?}"),
        }
    }

    #[test]
    fn non_object_top_level_returns_schema_error() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("s.json");
        fs::write(&path, "[]").unwrap();
        let err = run(&args("s.json", &["Edit"], &[]), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::JsonSchema { .. }));
    }

    #[test]
    fn deny_array_is_independently_handled() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("s.json");
        fs::write(
            &path,
            r#"{"permissions":{"allow":[],"deny":["Bash(rm -rf *)","Bash(rm -rf *)","Other"]}}"#,
        )
        .unwrap();
        let result = run(&args("s.json", &[], &["Bash(rm -rf *)"]), tmp.path()).unwrap();
        assert_eq!(result.deny_deduped, 1);
        assert_eq!(result.deny_added, 0);
        assert_eq!(result.allow_added, 0);
        assert_eq!(result.allow_deduped, 0);
    }

    #[test]
    fn non_string_entries_preserved_and_ignored_for_dedup() {
        let tmp = tempfile::tempdir().unwrap();
        let path = tmp.path().join("s.json");
        fs::write(
            &path,
            r#"{"permissions":{"allow":["Edit",42,"Edit",null,"Edit"],"deny":[]}}"#,
        )
        .unwrap();
        let result = run(&args("s.json", &[], &[]), tmp.path()).unwrap();
        assert_eq!(result.allow_deduped, 2, "duplicate 'Edit' entries removed");

        let body = fs::read_to_string(&path).unwrap();
        let parsed: Value = serde_json::from_str(&body).unwrap();
        let allow = parsed["permissions"]["allow"].as_array().unwrap();
        // After dedup: "Edit", 42, null
        assert_eq!(allow.len(), 3);
        assert_eq!(allow[0], "Edit");
        assert_eq!(allow[1], 42);
        assert_eq!(allow[2], Value::Null);
    }
}
