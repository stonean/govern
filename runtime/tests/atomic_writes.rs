//! Integration tests that verify the write primitives' rename atomicity by
//! reading the target file from a parallel thread while writes occur.
//!
//! POSIX (macOS, Linux) guarantees `rename(2)` is atomic when source and
//! destination share a filesystem. We assert that property by reading the
//! file repeatedly from a reader thread while a writer thread invokes a
//! write primitive many times: every observed snapshot must be either the
//! pre-write content or the post-write content, never a partial blend or
//! a missing-file error.
//!
//! Skipped on non-Unix targets where rename semantics are weaker (the
//! runtime documents the Windows gap in its README).

#![cfg(unix)]
#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::{Duration, Instant};

use gvrn::primitives;
use gvrn::schema::primitives::{MarkCriterionArgs, MarkTaskArgs, SetStatusArgs};

fn write_feature(tmp: &std::path::Path) {
    let feature_dir = tmp.join("specs/atom");
    fs::create_dir_all(&feature_dir).unwrap();
    fs::write(
        feature_dir.join("spec.md"),
        "---\nstatus: in-progress\ndependencies: []\n---\n\n# atom\n\n## Acceptance Criteria\n\n- [ ] First.\n- [ ] Second.\n",
    )
    .unwrap();
    fs::write(
        feature_dir.join("tasks.md"),
        "# atom\n\n## 1. Single task\n\n- [ ] Subtask one.\n- [ ] Subtask two.\n- **Done when**: both subtasks check.\n",
    )
    .unwrap();
}

fn parallel_read_assertion<P>(target: &Path, allowed: &[String], iterations: u32, mut writer: P)
where
    P: FnMut(u32),
{
    let stop = Arc::new(AtomicBool::new(false));
    let reader_stop = Arc::clone(&stop);
    let reader_target = target.to_path_buf();
    let reader_allowed: Vec<String> = allowed.to_vec();
    let reader = thread::spawn(move || {
        let mut observed: Vec<String> = Vec::new();
        while !reader_stop.load(Ordering::SeqCst) {
            match fs::read_to_string(&reader_target) {
                Ok(content) => {
                    if !reader_allowed.iter().any(|allowed| allowed == &content) {
                        return Err(format!(
                            "observed unexpected content (len={}): {content:?}",
                            content.len()
                        ));
                    }
                    observed.push(content);
                }
                Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                    return Err(format!(
                        "target {} vanished mid-rename — rename was not atomic",
                        reader_target.display()
                    ));
                }
                Err(err) => return Err(format!("read error: {err}")),
            }
        }
        Ok(observed.len())
    });

    let deadline = Instant::now() + Duration::from_secs(10);
    for i in 0..iterations {
        if Instant::now() > deadline {
            break;
        }
        writer(i);
    }
    stop.store(true, Ordering::SeqCst);
    let observed_count = reader.join().unwrap().unwrap();
    assert!(observed_count > 0, "reader thread never observed the file");
}

#[test]
fn mark_task_rename_is_atomic_under_parallel_reads() {
    let tmp = tempfile::tempdir().unwrap();
    write_feature(tmp.path());
    let tasks_path = tmp.path().join("specs/atom/tasks.md");
    let unchecked = fs::read_to_string(&tasks_path).unwrap();
    let checked = unchecked.replacen("- [ ] Subtask one.", "- [x] Subtask one.", 1);
    let allowed = vec![unchecked.clone(), checked.clone()];
    let repo = tmp.path().to_path_buf();

    parallel_read_assertion(&tasks_path, &allowed, 200, |i| {
        let checked = i % 2 == 0;
        primitives::mark_task::run(
            &MarkTaskArgs {
                feature: "atom".into(),
                task_number: "1".into(),
                subtask_index: 0,
                checked,
            },
            &repo,
        )
        .unwrap();
    });
}

#[test]
fn mark_criterion_rename_is_atomic_under_parallel_reads() {
    let tmp = tempfile::tempdir().unwrap();
    write_feature(tmp.path());
    let spec_path = tmp.path().join("specs/atom/spec.md");
    let unchecked = fs::read_to_string(&spec_path).unwrap();
    let checked = unchecked.replacen("- [ ] First.", "- [x] First.", 1);
    let allowed = vec![unchecked.clone(), checked.clone()];
    let repo = tmp.path().to_path_buf();

    parallel_read_assertion(&spec_path, &allowed, 200, |i| {
        let checked = i % 2 == 0;
        primitives::mark_criterion::run(
            &MarkCriterionArgs {
                feature: "atom".into(),
                criterion_index: 0,
                checked,
            },
            &repo,
        )
        .unwrap();
    });
}

#[test]
fn set_status_rename_is_atomic_under_parallel_reads() {
    let tmp = tempfile::tempdir().unwrap();
    write_feature(tmp.path());
    let spec_path = tmp.path().join("specs/atom/spec.md");
    let starting = fs::read_to_string(&spec_path).unwrap();
    let advanced = starting.replacen("status: in-progress", "status: done", 1);
    let allowed = vec![starting.clone(), advanced.clone()];
    let repo = tmp.path().to_path_buf();

    parallel_read_assertion(&spec_path, &allowed, 200, |i| {
        let (from, to) = if i % 2 == 0 {
            ("in-progress", "done")
        } else {
            ("done", "in-progress")
        };
        primitives::set_status::run(
            &SetStatusArgs {
                feature: "atom".into(),
                from: from.into(),
                to: to.into(),
            },
            &repo,
        )
        .unwrap();
    });
}
