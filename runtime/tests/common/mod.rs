//! Shared helpers for integration test crates under `runtime/tests/`.
//!
//! Each `tests/*.rs` file compiles as its own integration-test binary,
//! so plain top-level helpers are not directly shareable. The
//! `tests/common/mod.rs` shape is the idiomatic Rust workaround: a
//! sub-module path that cargo does NOT auto-build as a test binary
//! (the `tests/foo.rs` shape would). Each integration test that needs
//! a helper does `mod common;` at the top of its file and imports the
//! symbols it uses.

#![allow(dead_code, clippy::unwrap_used, clippy::expect_used)]

use std::fs;
use std::path::Path;

/// Recursively copy `src` into `dst`. Creates `dst` (and any missing
/// parents for nested files) as needed. Used to stage fixtures from
/// `runtime/tests/fixtures/<name>/` into a tempdir for write-side
/// integration tests.
pub fn copy_dir_recursive(src: &Path, dst: &Path) {
    fs::create_dir_all(dst).unwrap();
    for entry in fs::read_dir(src).unwrap() {
        let entry = entry.unwrap();
        let from = entry.path();
        let to = dst.join(entry.file_name());
        if from.is_dir() {
            copy_dir_recursive(&from, &to);
        } else {
            if let Some(parent) = to.parent() {
                fs::create_dir_all(parent).unwrap();
            }
            fs::copy(&from, &to).unwrap();
        }
    }
}
