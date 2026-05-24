//! `migrate-session-file` — translate a pre-0.10.0 legacy session file
//! (`.claude/{project}-session.json`, host- and project-name-specific) into
//! the consolidated repo-root `.govern.session.toml`, then delete the
//! legacy file.
//!
//! The translation:
//! - Renames the camelCase JSON keys to kebab-case (`scenarioPath` →
//!   `scenario-path`, `setAt` → `set-at`).
//! - Leaves every other top-level key intact under its existing name.
//!   Adopters with non-standard usage (walker-context-seed fields,
//!   custom adopter additions) keep those values.
//! - Writes via the standard tempfile + rename pattern.
//!
//! Backs `framework/migrations/session-file-consolidate.md` on the
//! runtime path. The markdown-only fallback in the migration body
//! describes the same translation by hand for adopters without `gvrn`
//! on `PATH`.

#![allow(clippy::expect_used)]

use std::path::Path;

use serde_json::{Map, Value as JsonValue};

use crate::primitives::write_session::SESSION_FILE;
use crate::primitives::{PrimitiveError, Result, read_text, validate_no_traversal, write_atomic};
use crate::schema::primitives::{MigrateSessionFileArgs, MigrateSessionFileResult};

/// Execute the `migrate-session-file` primitive against `repo`.
///
/// # Errors
///
/// Returns [`PrimitiveError::InvalidPath`] when `legacy-path` is absolute
/// or contains a parent-directory component, [`PrimitiveError::Json`]
/// when the legacy file is malformed JSON, [`PrimitiveError::JsonSchema`]
/// when the legacy file's top-level value isn't a JSON object, or
/// [`PrimitiveError::Io`] for filesystem failures during read, write,
/// or delete.
pub fn run(args: &MigrateSessionFileArgs, repo: &Path) -> Result<MigrateSessionFileResult> {
    validate_no_traversal(&args.legacy_path)?;
    let legacy_path = repo.join(&args.legacy_path);
    let dest_path = repo.join(SESSION_FILE);

    // Idempotency: no legacy file → no-op. Adopters who never had a
    // session file, or who already migrated, hit this branch.
    if !legacy_path.is_file() {
        return Ok(MigrateSessionFileResult {
            source: args.legacy_path.clone(),
            dest: SESSION_FILE.into(),
            action: "no-legacy".into(),
            legacy_deleted: false,
        });
    }

    // If `.govern.session.toml` is already present, the adopter has
    // already targeted post-consolidation — preserve the new file, but
    // still delete the legacy one (it's superseded and would otherwise
    // confuse future readers).
    if dest_path.is_file() {
        std::fs::remove_file(&legacy_path).map_err(|source| PrimitiveError::Io {
            path: legacy_path.clone(),
            source,
        })?;
        return Ok(MigrateSessionFileResult {
            source: args.legacy_path.clone(),
            dest: SESSION_FILE.into(),
            action: "kept-existing".into(),
            legacy_deleted: true,
        });
    }

    // Translate JSON → TOML with key renames.
    let content = read_text(&legacy_path)?;
    let parsed: JsonValue =
        serde_json::from_str(&content).map_err(|source| PrimitiveError::Json {
            path: legacy_path.clone(),
            source,
        })?;
    let JsonValue::Object(legacy_map) = parsed else {
        return Err(PrimitiveError::JsonSchema {
            path: legacy_path.clone(),
            reason: "top-level value is not a JSON object".into(),
        });
    };

    let toml_body = render_toml(legacy_map);
    write_atomic(&dest_path, &toml_body)?;

    std::fs::remove_file(&legacy_path).map_err(|source| PrimitiveError::Io {
        path: legacy_path.clone(),
        source,
    })?;

    Ok(MigrateSessionFileResult {
        source: args.legacy_path.clone(),
        dest: SESSION_FILE.into(),
        action: "migrated".into(),
        legacy_deleted: true,
    })
}

/// Apply the key renames (`scenarioPath` → `scenario-path`,
/// `setAt` → `set-at`; everything else preserved) and render the result
/// as a TOML document.
///
/// `toml::to_string` over a `toml::Value::Table` built from a JSON
/// object is infallible — every JSON value the converter accepts has a
/// well-formed TOML representation — so the `.expect` documents the
/// invariant rather than handling a reachable failure mode. Same pattern
/// as `write_session::run_with_now` and `merge_permissions::serialize_pretty`.
fn render_toml(legacy_map: Map<String, JsonValue>) -> String {
    let mut table = toml::value::Table::new();
    for (key, value) in legacy_map {
        let new_key = match key.as_str() {
            "scenarioPath" => "scenario-path".to_string(),
            "setAt" => "set-at".to_string(),
            _ => key,
        };
        if let Some(tv) = json_to_toml_value(value) {
            table.insert(new_key, tv);
        }
        // JSON null has no TOML representation; the session schema does
        // not use nulls in practice, but if one is present the field is
        // dropped (consistent with TOML's missing-key semantics).
    }
    toml::to_string(&toml::Value::Table(table)).expect("session TOML serializes infallibly")
}

/// Recursive JSON-`Value` → TOML-`Value` converter. Handles every shape
/// the session file or walker-context-seed extensions can use: strings,
/// numbers (split into integer / float), booleans, arrays, sub-objects.
/// JSON `null` becomes `None` because TOML has no null type — the
/// caller drops the key in that case.
fn json_to_toml_value(value: JsonValue) -> Option<toml::Value> {
    Some(match value {
        JsonValue::Null => return None,
        JsonValue::Bool(b) => toml::Value::Boolean(b),
        JsonValue::Number(n) => {
            if let Some(i) = n.as_i64() {
                toml::Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                toml::Value::Float(f)
            } else {
                // Unrepresentable number (e.g., u64 > i64::MAX) — drop.
                return None;
            }
        }
        JsonValue::String(s) => toml::Value::String(s),
        JsonValue::Array(arr) => {
            toml::Value::Array(arr.into_iter().filter_map(json_to_toml_value).collect())
        }
        JsonValue::Object(obj) => {
            let mut table = toml::value::Table::new();
            for (k, v) in obj {
                if let Some(tv) = json_to_toml_value(v) {
                    table.insert(k, tv);
                }
            }
            toml::Value::Table(table)
        }
    })
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn args(legacy_path: &str) -> MigrateSessionFileArgs {
        MigrateSessionFileArgs {
            legacy_path: legacy_path.into(),
        }
    }

    fn write_legacy(repo: &Path, rel: &str, body: &str) {
        let path = repo.join(rel);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&path, body).unwrap();
    }

    #[test]
    fn no_legacy_file_is_idempotent_noop() {
        let tmp = tempdir().unwrap();
        let result = run(&args(".claude/gov-session.json"), tmp.path()).unwrap();
        assert_eq!(result.action, "no-legacy");
        assert_eq!(result.dest, ".govern.session.toml");
        assert!(!result.legacy_deleted);
        // No files materialize.
        assert!(!tmp.path().join(".govern.session.toml").exists());
    }

    #[test]
    fn translates_session_file_with_kebab_renames() {
        let tmp = tempdir().unwrap();
        write_legacy(
            tmp.path(),
            ".claude/gov-session.json",
            r#"{
  "feature": "022-deterministic-runtime",
  "path": "specs/022-deterministic-runtime",
  "scenario": "write-session-primitive",
  "scenarioPath": "specs/022-deterministic-runtime/scenarios/write-session-primitive.md",
  "setAt": "2026-05-23T12:34:56Z"
}
"#,
        );

        let result = run(&args(".claude/gov-session.json"), tmp.path()).unwrap();
        assert_eq!(result.action, "migrated");
        assert!(result.legacy_deleted);

        // Legacy gone.
        assert!(!tmp.path().join(".claude/gov-session.json").exists());

        // New file present, kebab-case keys.
        let body = fs::read_to_string(tmp.path().join(".govern.session.toml")).unwrap();
        assert!(
            body.contains("feature = \"022-deterministic-runtime\""),
            "{body}"
        );
        assert!(
            body.contains("path = \"specs/022-deterministic-runtime\""),
            "{body}"
        );
        assert!(
            body.contains("scenario = \"write-session-primitive\""),
            "{body}"
        );
        assert!(
            body.contains("scenario-path = \"specs/022-deterministic-runtime/scenarios/write-session-primitive.md\""),
            "{body}"
        );
        assert!(body.contains("set-at = \"2026-05-23T12:34:56Z\""), "{body}");
        // The camelCase keys MUST NOT survive into the TOML — if they
        // did, the dashboard reader would silently ignore them and the
        // adopter would lose their session-target.
        assert!(!body.contains("scenarioPath"), "{body}");
        assert!(!body.contains("setAt"), "{body}");
    }

    #[test]
    fn preserves_existing_target_toml_and_deletes_legacy() {
        let tmp = tempdir().unwrap();
        // Adopter already targeted post-consolidation:
        fs::write(
            tmp.path().join(".govern.session.toml"),
            "feature = \"new\"\npath = \"specs/new\"\nset-at = \"2026-05-24T00:00:00Z\"\n",
        )
        .unwrap();
        // ... but a stale legacy file is also on disk:
        write_legacy(
            tmp.path(),
            ".claude/gov-session.json",
            r#"{"feature":"old","path":"specs/old","setAt":"2026-05-01T00:00:00Z"}"#,
        );

        let result = run(&args(".claude/gov-session.json"), tmp.path()).unwrap();
        assert_eq!(result.action, "kept-existing");
        assert!(result.legacy_deleted);

        // Legacy gone, new file untouched.
        assert!(!tmp.path().join(".claude/gov-session.json").exists());
        let body = fs::read_to_string(tmp.path().join(".govern.session.toml")).unwrap();
        assert!(body.contains("feature = \"new\""));
        assert!(!body.contains("old"));
    }

    #[test]
    fn preserves_non_standard_top_level_keys() {
        // Adopters who used the session file as a walker-context seed
        // (the fixture pattern in this repo's runtime/tests/) have
        // extra keys beyond the session-target schema. The migration
        // preserves them under the same name.
        let tmp = tempdir().unwrap();
        write_legacy(
            tmp.path(),
            ".claude/gov-session.json",
            r#"{
  "feature": "022-deterministic-runtime",
  "path": "specs/022-deterministic-runtime",
  "setAt": "2026-05-23T12:34:56Z",
  "url": "https://example.test/archive.tar.gz",
  "entries": [{"source": "a.md", "dest": "b.md", "strategy": "update"}],
  "substitutions": {"project": "anvil"}
}
"#,
        );
        run(&args(".claude/gov-session.json"), tmp.path()).unwrap();

        let body = fs::read_to_string(tmp.path().join(".govern.session.toml")).unwrap();
        // Renames applied to known keys, non-standard keys preserved.
        assert!(body.contains("set-at = \"2026-05-23T12:34:56Z\""), "{body}");
        assert!(
            body.contains("url = \"https://example.test/archive.tar.gz\""),
            "{body}"
        );
        assert!(body.contains("[[entries]]"), "{body}");
        assert!(body.contains("source = \"a.md\""), "{body}");
        assert!(body.contains("[substitutions]"), "{body}");
        assert!(body.contains("project = \"anvil\""), "{body}");
    }

    #[test]
    fn rejects_legacy_path_with_parent_component() {
        let tmp = tempdir().unwrap();
        let err = run(&args("../escape.json"), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidPath { .. }));
    }

    #[test]
    fn rejects_absolute_legacy_path() {
        let tmp = tempdir().unwrap();
        let err = run(&args("/etc/passwd"), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidPath { .. }));
    }

    #[test]
    fn rejects_malformed_json() {
        let tmp = tempdir().unwrap();
        write_legacy(tmp.path(), ".claude/gov-session.json", "{not valid json");
        let err = run(&args(".claude/gov-session.json"), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::Json { .. }));
        // Legacy file unchanged on parse failure (atomic-write contract).
        assert!(tmp.path().join(".claude/gov-session.json").exists());
        assert!(!tmp.path().join(".govern.session.toml").exists());
    }

    #[test]
    fn rejects_non_object_top_level() {
        let tmp = tempdir().unwrap();
        write_legacy(tmp.path(), ".claude/gov-session.json", "[1, 2, 3]");
        let err = run(&args(".claude/gov-session.json"), tmp.path()).unwrap_err();
        assert!(matches!(err, PrimitiveError::JsonSchema { .. }));
    }

    #[test]
    fn target_dest_matches_write_session_constant() {
        // The migration's destination is the same file `write-session`
        // writes — they MUST agree on the path. If `SESSION_FILE` is
        // ever renamed, this assertion fails at compile/test time and
        // forces the migration body to be updated in lockstep.
        assert_eq!(SESSION_FILE, ".govern.session.toml");
    }
}
