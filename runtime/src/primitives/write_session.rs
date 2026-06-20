//! `write-session` — atomically rewrite the session state file at
//! `<repo>/.govern.session.toml`.
//!
//! Mirrors the write-side of [`crate::primitives::dashboard::load_session_target`].
//! The session file is the second of the two durable journals named by
//! spec 022 (markdown + `.govern.session.toml`, per
//! `specs/022-deterministic-runtime/plan.md` §No data persistence outside
//! session file + markdown); the read path is exposed by `dashboard`, and
//! this primitive is the matching write path.
//!
//! Pre-022 prose left the write to the host's file-writing tool (`Write`
//! on Claude Code), which on Claude Code surfaces a per-invocation
//! permission prompt that documented `Write(...)` allow entries have not
//! reliably suppressed. Routing the write through an MCP tool moves the
//! consent into the MCP tool-permission lane, so a single allow covers
//! every subsequent target/scenario-switch.
//!
//! The previous shape — host-specific JSON at `{cli-config-dir}/{project}-session.json`
//! (e.g., `.claude/gov-session.json`) — coupled the session location to
//! both the AI CLI (`.claude/` vs `.augment/`) and the adopting project's
//! name (`gov-session.json` vs `anvil-session.json`). Consolidating onto
//! `.govern.session.toml` at the repo root makes the path host-agnostic,
//! project-name-agnostic, and uniform across every adopter; the runtime
//! no longer hardcodes any AI CLI's config directory.

#![allow(clippy::expect_used)]

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::primitives::{PrimitiveError, Result, rel_path, validate_no_traversal, write_atomic};
use crate::schema::primitives::{WriteSessionArgs, WriteSessionResult};

/// Repo-relative path of the session file. Hardcoded — there is no host
/// variability to parameterize anymore; the file lives at the repo root
/// for every adopter.
pub(crate) const SESSION_FILE: &str = ".govern.session.toml";

/// Execute the `write-session` primitive against `repo`.
///
/// Writes a fresh TOML document at `<repo>/.govern.session.toml` via
/// tempfile + rename — same atomic-write pattern every other
/// state-modifying primitive (`mark-task`, `mark-criterion`,
/// `set-status`) uses.
///
/// # Errors
///
/// Returns [`PrimitiveError::MissingArgument`] when `scenario` and
/// `scenario-path` are not supplied together, [`PrimitiveError::InvalidPath`]
/// when any caller-supplied path contains a parent-directory component or
/// is absolute, or [`PrimitiveError::Io`] for filesystem failures during
/// the write.
pub fn run(args: &WriteSessionArgs, repo: &Path) -> Result<WriteSessionResult> {
    run_with_now(args, repo, SystemTime::now())
}

/// Implementation seam that lets unit tests inject a stable clock instead
/// of `SystemTime::now()`. The MCP and CLI surfaces both call [`run`],
/// which forwards the system clock.
pub(crate) fn run_with_now(
    args: &WriteSessionArgs,
    repo: &Path,
    now: SystemTime,
) -> Result<WriteSessionResult> {
    if let Some(path) = &args.path {
        validate_no_traversal(path)?;
    }
    if let Some(scenario_path) = &args.scenario_path {
        validate_no_traversal(scenario_path)?;
    }
    // `feature` and `path` are a pair — a target needs both.
    match (&args.feature, &args.path) {
        (Some(_), None) => {
            return Err(PrimitiveError::MissingArgument {
                primitive: "write-session".into(),
                argument: "path".into(),
                reason: "must be supplied together with `feature`".into(),
            });
        }
        (None, Some(_)) => {
            return Err(PrimitiveError::MissingArgument {
                primitive: "write-session".into(),
                argument: "feature".into(),
                reason: "must be supplied together with `path`".into(),
            });
        }
        _ => {}
    }
    match (&args.scenario, &args.scenario_path) {
        (Some(_), None) => {
            return Err(PrimitiveError::MissingArgument {
                primitive: "write-session".into(),
                argument: "scenario-path".into(),
                reason: "must be supplied together with `scenario`".into(),
            });
        }
        (None, Some(_)) => {
            return Err(PrimitiveError::MissingArgument {
                primitive: "write-session".into(),
                argument: "scenario".into(),
                reason: "must be supplied together with `scenario-path`".into(),
            });
        }
        _ => {}
    }
    // A scenario only means something inside a target write.
    if args.scenario.is_some() && args.feature.is_none() {
        return Err(PrimitiveError::MissingArgument {
            primitive: "write-session".into(),
            argument: "feature".into(),
            reason: "`scenario` requires a target write (supply `feature` and `path`)".into(),
        });
    }
    // Nothing to do unless this is a target write or a host-config write.
    if args.feature.is_none() && args.cli_config_dir.is_none() {
        return Err(PrimitiveError::MissingArgument {
            primitive: "write-session".into(),
            argument: "feature".into(),
            reason:
                "supply `feature`+`path` (target write) or `cli-config-dir` (host-config write)"
                    .into(),
        });
    }

    let session_path = repo.join(SESSION_FILE);
    let created = !session_path.exists();
    let existing = read_existing_session(&session_path);

    // Merge semantics. A *target write* (feature supplied) sets the target
    // fields from args — including clearing `scenario` when it's absent — and
    // stamps a fresh `set-at`, preserving the per-contributor `cli-config-dir`
    // unless overridden. A *host-config write* (no feature) sets
    // `cli-config-dir` and preserves the existing target verbatim.
    let record = if args.feature.is_some() {
        SessionRecord {
            feature: args.feature.clone(),
            path: args.path.clone(),
            scenario: args.scenario.clone(),
            scenario_path: args.scenario_path.clone(),
            set_at: Some(iso8601_utc(now)),
            cli_config_dir: args.cli_config_dir.clone().or(existing.cli_config_dir),
        }
    } else {
        SessionRecord {
            feature: existing.feature,
            path: existing.path,
            scenario: existing.scenario,
            scenario_path: existing.scenario_path,
            set_at: existing.set_at,
            cli_config_dir: args.cli_config_dir.clone(),
        }
    };
    // `toml::to_string` over a struct of `Option<String>` is infallible — no
    // non-string keys, no I/O, no exotic types — so the `expect` documents the
    // invariant rather than handling a reachable failure mode. Same pattern as
    // `merge_permissions::serialize_pretty`.
    let body = toml::to_string(&record).expect("session TOML serializes infallibly");

    write_atomic(&session_path, &body)?;

    Ok(WriteSessionResult {
        path: rel_path(&session_path, repo),
        created,
    })
}

/// On-disk shape of the session file. Field order is the wire contract —
/// the parity byte-equality check on `.govern.session.toml` depends on it.
/// All keys are kebab-case to match the reader in
/// [`crate::primitives::dashboard`]. Every field is optional: a host-config
/// write (only `cli-config-dir`) against a fresh repo writes just that key,
/// and a target write writes the target block plus the preserved
/// `cli-config-dir`.
#[derive(Serialize)]
struct SessionRecord {
    #[serde(skip_serializing_if = "Option::is_none")]
    feature: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scenario: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "scenario-path")]
    scenario_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "set-at")]
    set_at: Option<String>,
    // Serialized last so the target block (feature/path/scenario/set-at)
    // keeps its byte-for-byte order; absent unless a write recorded it.
    #[serde(skip_serializing_if = "Option::is_none", rename = "cli-config-dir")]
    cli_config_dir: Option<String>,
}

/// The fields of an existing `.govern.session.toml` a write may need to
/// carry forward: a target write preserves `cli-config-dir`; a host-config
/// write preserves the whole target block.
#[derive(Deserialize, Default)]
struct ExistingSession {
    #[serde(default)]
    feature: Option<String>,
    #[serde(default)]
    path: Option<String>,
    #[serde(default)]
    scenario: Option<String>,
    #[serde(default, rename = "scenario-path")]
    scenario_path: Option<String>,
    #[serde(default, rename = "set-at")]
    set_at: Option<String>,
    #[serde(default, rename = "cli-config-dir")]
    cli_config_dir: Option<String>,
}

/// Best-effort read of the current session file at `session_path`. A missing
/// or malformed file yields an empty record so a write simply has nothing to
/// preserve rather than failing.
fn read_existing_session(session_path: &Path) -> ExistingSession {
    let Ok(content) = std::fs::read_to_string(session_path) else {
        return ExistingSession::default();
    };
    toml::from_str::<ExistingSession>(&content).unwrap_or_default()
}

/// Format `now` as an RFC 3339 / ISO 8601 UTC timestamp
/// (`YYYY-MM-DDTHH:MM:SSZ`). Matches the field shape `setAt` used
/// pre-consolidation; the TOML key is now `set-at`, but the value
/// remains an ISO 8601 UTC string.
///
/// Uses Howard Hinnant's date algorithms — the standard branchless
/// civil-from-days computation. Valid for any date the underlying
/// `SystemTime` can represent, including the entire post-1970 range
/// the session file actually sees. A `now` earlier than the epoch
/// falls back to `1970-01-01T00:00:00Z`, which the session file
/// never produces in practice.
fn iso8601_utc(now: SystemTime) -> String {
    let secs = now.duration_since(UNIX_EPOCH).map_or(0, |d| d.as_secs());
    let days = secs / 86_400;
    let tod = secs % 86_400;
    let hour = tod / 3600;
    let min = (tod / 60) % 60;
    let sec = tod % 60;

    // `days` from a post-1970 SystemTime fits in i64 with enormous headroom;
    // dates past year ~9999 are far outside any session file's lifetime.
    #[allow(clippy::cast_possible_wrap)]
    let (year, month, day) = civil_from_days(days as i64);
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{min:02}:{sec:02}Z")
}

/// Convert days-since-1970-01-01 (Gregorian) into `(year, month, day)`.
///
/// Howard Hinnant's standard civil-from-days algorithm. The intermediate
/// casts (`i64` ↔ `u64`, `u64` → `u32`) are part of the algorithm and
/// safe for any input in the post-1970, pre-year-9999 range the session
/// file will ever produce.
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 {
        z / 146_097
    } else {
        (z - 146_096) / 146_097
    };
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = y + i64::from(month <= 2);
    (year, month as u32, day as u32)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::fs;
    use std::time::Duration;
    use tempfile::tempdir;

    fn fixed_now() -> SystemTime {
        // 2026-05-23T12:34:56Z
        UNIX_EPOCH + Duration::from_secs(1_779_539_696)
    }

    fn base_args() -> WriteSessionArgs {
        WriteSessionArgs {
            feature: Some("022-deterministic-runtime".into()),
            path: Some("specs/022-deterministic-runtime".into()),
            scenario: None,
            scenario_path: None,
            cli_config_dir: None,
        }
    }

    #[test]
    fn writes_canonical_shape_without_scenario() {
        let tmp = tempdir().unwrap();
        let result = run_with_now(&base_args(), tmp.path(), fixed_now()).unwrap();
        assert_eq!(result.path, ".govern.session.toml");
        assert!(result.created, "fresh file is reported as created");

        let body = fs::read_to_string(tmp.path().join(".govern.session.toml")).unwrap();
        assert_eq!(
            body,
            "feature = \"022-deterministic-runtime\"\n\
             path = \"specs/022-deterministic-runtime\"\n\
             set-at = \"2026-05-23T12:34:56Z\"\n"
        );
    }

    #[test]
    fn writes_scenario_pair_when_both_supplied() {
        let tmp = tempdir().unwrap();
        let mut args = base_args();
        args.scenario = Some("write-session-primitive".into());
        args.scenario_path =
            Some("specs/022-deterministic-runtime/scenarios/write-session-primitive.md".into());

        let result = run_with_now(&args, tmp.path(), fixed_now()).unwrap();
        assert!(result.created);

        let body = fs::read_to_string(tmp.path().join(".govern.session.toml")).unwrap();
        assert!(
            body.contains("scenario = \"write-session-primitive\""),
            "{body}"
        );
        assert!(
            body.contains("scenario-path = \"specs/022-deterministic-runtime/scenarios/write-session-primitive.md\""),
            "{body}"
        );
        // Key order: feature, path, scenario, scenario-path, set-at.
        let feat = body.find("feature =").unwrap();
        let p = body.find("path =").unwrap();
        let scen = body.find("scenario =").unwrap();
        let scen_path = body.find("scenario-path =").unwrap();
        let set_at = body.find("set-at =").unwrap();
        assert!(feat < p && p < scen && scen < scen_path && scen_path < set_at);
    }

    #[test]
    fn overwrites_existing_file_and_reports_not_created() {
        let tmp = tempdir().unwrap();
        fs::write(
            tmp.path().join(".govern.session.toml"),
            "feature = \"old-feature\"\npath = \"specs/old\"\nset-at = \"2026-01-01T00:00:00Z\"\n",
        )
        .unwrap();

        let result = run_with_now(&base_args(), tmp.path(), fixed_now()).unwrap();
        assert!(!result.created, "existing file is reported as overwritten");

        let body = fs::read_to_string(tmp.path().join(".govern.session.toml")).unwrap();
        assert!(body.contains("022-deterministic-runtime"));
        assert!(!body.contains("old-feature"));
    }

    #[test]
    fn writes_at_repo_root_regardless_of_project_name() {
        // The point of the consolidation: the path doesn't change with
        // project name, AI CLI, or anything else. It is always
        // `.govern.session.toml` at the repo root.
        let tmp = tempdir().unwrap();
        let mut args = base_args();
        args.feature = Some("002-observability".into());
        args.path = Some("specs/002-observability".into());
        let result = run_with_now(&args, tmp.path(), fixed_now()).unwrap();
        assert_eq!(result.path, ".govern.session.toml");
        assert!(tmp.path().join(".govern.session.toml").is_file());
        // No host-specific or project-specific sibling exists.
        assert!(!tmp.path().join(".claude").exists());
        assert!(!tmp.path().join(".claude/gov-session.json").exists());
        assert!(!tmp.path().join(".claude/anvil-session.json").exists());
    }

    #[test]
    fn preserves_cli_config_dir_across_a_target_switch() {
        // `/govern` records the per-contributor `cli-config-dir` in the
        // session file; a later `/{project}:target` rewrites the file for a
        // new feature and must NOT drop it.
        let tmp = tempdir().unwrap();
        fs::write(
            tmp.path().join(".govern.session.toml"),
            "feature = \"001-old\"\npath = \"specs/001-old\"\nset-at = \"2026-01-01T00:00:00Z\"\ncli-config-dir = \".opencode\"\n",
        )
        .unwrap();

        let mut args = base_args();
        args.feature = Some("002-new".into());
        args.path = Some("specs/002-new".into());
        run_with_now(&args, tmp.path(), fixed_now()).unwrap();

        let body = fs::read_to_string(tmp.path().join(".govern.session.toml")).unwrap();
        assert!(body.contains("feature = \"002-new\""), "{body}");
        assert!(
            body.contains("cli-config-dir = \".opencode\""),
            "cli-config-dir must survive the target switch: {body}"
        );
        // Serialized after the target block.
        assert!(body.find("set-at =").unwrap() < body.find("cli-config-dir =").unwrap());
    }

    #[test]
    fn omits_cli_config_dir_when_none_recorded() {
        let tmp = tempdir().unwrap();
        run_with_now(&base_args(), tmp.path(), fixed_now()).unwrap();
        let body = fs::read_to_string(tmp.path().join(".govern.session.toml")).unwrap();
        assert!(!body.contains("cli-config-dir"), "{body}");
    }

    #[test]
    fn host_config_write_sets_cli_config_dir_on_fresh_repo() {
        // `/govern` setting the agent identity before any target is selected:
        // a host-config write (no feature) against a fresh repo writes just
        // `cli-config-dir`.
        let tmp = tempdir().unwrap();
        let args = WriteSessionArgs {
            feature: None,
            path: None,
            scenario: None,
            scenario_path: None,
            cli_config_dir: Some(".opencode".into()),
        };
        let result = run_with_now(&args, tmp.path(), fixed_now()).unwrap();
        assert!(result.created);
        let body = fs::read_to_string(tmp.path().join(".govern.session.toml")).unwrap();
        assert_eq!(body, "cli-config-dir = \".opencode\"\n");
    }

    #[test]
    fn host_config_write_preserves_existing_target() {
        // Setting `cli-config-dir` after a target is already selected must not
        // disturb the target block (feature/path/scenario/set-at).
        let tmp = tempdir().unwrap();
        fs::write(
            tmp.path().join(".govern.session.toml"),
            "feature = \"001-x\"\npath = \"specs/001-x\"\nset-at = \"2026-01-01T00:00:00Z\"\n",
        )
        .unwrap();
        let args = WriteSessionArgs {
            feature: None,
            path: None,
            scenario: None,
            scenario_path: None,
            cli_config_dir: Some(".augment".into()),
        };
        run_with_now(&args, tmp.path(), fixed_now()).unwrap();
        let body = fs::read_to_string(tmp.path().join(".govern.session.toml")).unwrap();
        assert!(body.contains("feature = \"001-x\""), "{body}");
        assert!(body.contains("path = \"specs/001-x\""), "{body}");
        assert!(body.contains("set-at = \"2026-01-01T00:00:00Z\""), "{body}");
        assert!(body.contains("cli-config-dir = \".augment\""), "{body}");
    }

    #[test]
    fn rejects_write_with_neither_target_nor_cli_config_dir() {
        let tmp = tempdir().unwrap();
        let args = WriteSessionArgs {
            feature: None,
            path: None,
            scenario: None,
            scenario_path: None,
            cli_config_dir: None,
        };
        let err = run_with_now(&args, tmp.path(), fixed_now()).unwrap_err();
        assert!(matches!(err, PrimitiveError::MissingArgument { .. }));
        assert!(!tmp.path().join(".govern.session.toml").exists());
    }

    #[test]
    fn rejects_feature_without_path() {
        let tmp = tempdir().unwrap();
        let args = WriteSessionArgs {
            feature: Some("001-x".into()),
            path: None,
            scenario: None,
            scenario_path: None,
            cli_config_dir: None,
        };
        let err = run_with_now(&args, tmp.path(), fixed_now()).unwrap_err();
        match err {
            PrimitiveError::MissingArgument {
                primitive,
                argument,
                ..
            } => {
                assert_eq!(primitive, "write-session");
                assert_eq!(argument, "path");
            }
            other => panic!("expected MissingArgument, got {other:?}"),
        }
    }

    #[test]
    fn rejects_scenario_without_a_target() {
        // A scenario is a sub-selection of the current target, so it requires
        // a target write (feature + path).
        let tmp = tempdir().unwrap();
        let args = WriteSessionArgs {
            feature: None,
            path: None,
            scenario: Some("x".into()),
            scenario_path: Some("specs/x/scenarios/y.md".into()),
            cli_config_dir: Some(".opencode".into()),
        };
        let err = run_with_now(&args, tmp.path(), fixed_now()).unwrap_err();
        match err {
            PrimitiveError::MissingArgument {
                primitive,
                argument,
                ..
            } => {
                assert_eq!(primitive, "write-session");
                assert_eq!(argument, "feature");
            }
            other => panic!("expected MissingArgument, got {other:?}"),
        }
    }

    #[test]
    fn clearing_scenario_omits_both_keys() {
        let tmp = tempdir().unwrap();
        // First write with a scenario set.
        let mut with_scenario = base_args();
        with_scenario.scenario = Some("write-session-primitive".into());
        with_scenario.scenario_path =
            Some("specs/022-deterministic-runtime/scenarios/write-session-primitive.md".into());
        run_with_now(&with_scenario, tmp.path(), fixed_now()).unwrap();

        // Then overwrite without — both keys must vanish.
        run_with_now(&base_args(), tmp.path(), fixed_now()).unwrap();
        let body = fs::read_to_string(tmp.path().join(".govern.session.toml")).unwrap();
        assert!(!body.contains("scenario"), "{body}");
        assert!(!body.contains("scenario-path"), "{body}");
    }

    #[test]
    fn rejects_scenario_without_scenario_path() {
        let tmp = tempdir().unwrap();
        let mut args = base_args();
        args.scenario = Some("orphan".into());
        let err = run_with_now(&args, tmp.path(), fixed_now()).unwrap_err();
        match err {
            PrimitiveError::MissingArgument {
                primitive,
                argument,
                ..
            } => {
                assert_eq!(primitive, "write-session");
                assert_eq!(argument, "scenario-path");
            }
            other => panic!("expected MissingArgument, got {other:?}"),
        }
        // Disk is unchanged (no file created).
        assert!(!tmp.path().join(".govern.session.toml").exists());
    }

    #[test]
    fn rejects_scenario_path_without_scenario() {
        let tmp = tempdir().unwrap();
        let mut args = base_args();
        args.scenario_path = Some("specs/x/scenarios/y.md".into());
        let err = run_with_now(&args, tmp.path(), fixed_now()).unwrap_err();
        match err {
            PrimitiveError::MissingArgument {
                primitive,
                argument,
                ..
            } => {
                assert_eq!(primitive, "write-session");
                assert_eq!(argument, "scenario");
            }
            other => panic!("expected MissingArgument, got {other:?}"),
        }
    }

    #[test]
    fn rejects_path_with_parent_component() {
        let tmp = tempdir().unwrap();
        let mut args = base_args();
        args.path = Some("specs/../escape".into());
        let err = run_with_now(&args, tmp.path(), fixed_now()).unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidPath { .. }));
    }

    #[test]
    fn rejects_absolute_scenario_path() {
        let tmp = tempdir().unwrap();
        let mut args = base_args();
        args.scenario = Some("x".into());
        args.scenario_path = Some("/etc/passwd".into());
        let err = run_with_now(&args, tmp.path(), fixed_now()).unwrap_err();
        assert!(matches!(err, PrimitiveError::InvalidPath { .. }));
    }

    #[test]
    fn dropping_named_tempfile_leaves_existing_session_unchanged() {
        use std::io::Write;
        let tmp = tempdir().unwrap();
        let session_path = tmp.path().join(".govern.session.toml");
        let original = "feature = \"unchanged\"\n";
        fs::write(&session_path, original).unwrap();
        {
            let mut tf = tempfile::NamedTempFile::new_in(tmp.path()).unwrap();
            tf.write_all(b"INTERRUPTED").unwrap();
        }
        assert_eq!(fs::read_to_string(&session_path).unwrap(), original);
    }

    #[test]
    fn iso8601_utc_formats_known_epoch() {
        // 0 → 1970-01-01T00:00:00Z
        assert_eq!(iso8601_utc(UNIX_EPOCH), "1970-01-01T00:00:00Z");
        // 1700000000 → 2023-11-14T22:13:20Z (a well-known epoch).
        assert_eq!(
            iso8601_utc(UNIX_EPOCH + Duration::from_secs(1_700_000_000)),
            "2023-11-14T22:13:20Z"
        );
        // Our fixed test moment.
        assert_eq!(iso8601_utc(fixed_now()), "2026-05-23T12:34:56Z");
    }

    #[test]
    fn civil_from_days_handles_leap_years() {
        // 2024-02-29 — 2024 is a leap year.
        let days = day_count(2024, 2, 29);
        assert_eq!(civil_from_days(days), (2024, 2, 29));
        // 2100-03-01 — 2100 is NOT a leap year (divisible by 100, not 400).
        let days = day_count(2100, 3, 1);
        assert_eq!(civil_from_days(days), (2100, 3, 1));
        // 2000-02-29 — 2000 IS a leap year.
        let days = day_count(2000, 2, 29);
        assert_eq!(civil_from_days(days), (2000, 2, 29));
    }

    /// Round-trip helper: count days since 1970-01-01 for a known date.
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
    fn day_count(year: i64, month: u32, day: u32) -> i64 {
        let y = year - i64::from(month <= 2);
        let era = if y >= 0 { y / 400 } else { (y - 399) / 400 };
        let yoe = (y - era * 400) as u64;
        let m = u64::from(month);
        let d = u64::from(day);
        let doy = (153 * (if m > 2 { m - 3 } else { m + 9 }) + 2) / 5 + d - 1;
        let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
        era * 146_097 + doe as i64 - 719_468
    }
}
