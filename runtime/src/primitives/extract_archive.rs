//! `extract-archive` — extract a local `.tar.gz`/`.tgz`/`.zip` into a
//! destination directory.
//!
//! Both formats are handled entirely in-process (no shell-out): tar.gz
//! via `flate2` + `tar`, zip via the `zip` crate. Every entry path is
//! validated against directory traversal — absolute paths, components
//! containing `..`, and paths that would resolve outside the requested
//! `dest` directory all yield [`PrimitiveError::UnsafeArchivePath`]
//! before any file is written.
//!
//! The result lists every regular file extracted (relative to `dest`)
//! in archive order. Directory entries are created implicitly and not
//! counted. Symlinks inside the archive are not extracted — they are
//! ignored with no error (a future revision may surface them as a
//! finding).

use std::fs;
use std::io::Read;
use std::path::{Component, Path, PathBuf};

use crate::primitives::{PrimitiveError, Result};
use crate::schema::primitives::{ExtractArchiveArgs, ExtractArchiveResult};

/// Execute the `extract-archive` primitive.
///
/// # Errors
///
/// - [`PrimitiveError::Io`] when the archive cannot be read or files cannot be
///   written to disk.
/// - [`PrimitiveError::UnknownArchiveFormat`] when the format cannot be inferred
///   and no override was supplied.
/// - [`PrimitiveError::UnsafeArchivePath`] for absolute paths or paths that
///   resolve outside `dest`.
pub fn run(args: &ExtractArchiveArgs, repo: &Path) -> Result<ExtractArchiveResult> {
    let archive_path = resolve_path(repo, &args.archive);
    let dest = resolve_path(repo, &args.dest);
    let format = detect_format(&archive_path, args.format.as_deref())?;

    fs::create_dir_all(&dest).map_err(|source| PrimitiveError::Io {
        path: dest.clone(),
        source,
    })?;

    let files = match format.as_str() {
        "tar-gz" => extract_tar_gz(&archive_path, &dest)?,
        "zip" => extract_zip(&archive_path, &dest)?,
        _ => unreachable!("detect_format only returns tar-gz or zip"),
    };

    let count = u32::try_from(files.len()).unwrap_or(u32::MAX);
    Ok(ExtractArchiveResult {
        dest: dest.to_string_lossy().into_owned(),
        files,
        count,
        format,
    })
}

fn resolve_path(repo: &Path, p: &str) -> PathBuf {
    let candidate = Path::new(p);
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo.join(candidate)
    }
}

// The `name` binding below is already lowercased; clippy's
// case-sensitivity warning doesn't apply.
#[allow(clippy::case_sensitive_file_extension_comparisons)]
fn detect_format(archive: &Path, override_format: Option<&str>) -> Result<String> {
    if let Some(o) = override_format {
        let normalized = o.to_ascii_lowercase().replace('_', "-");
        if normalized == "tar-gz" || normalized == "zip" {
            return Ok(normalized);
        }
        return Err(PrimitiveError::UnknownArchiveFormat {
            path: archive.into(),
        });
    }
    let name = archive
        .file_name()
        .map(|s| s.to_string_lossy().to_ascii_lowercase())
        .unwrap_or_default();
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        Ok("tar-gz".into())
    } else if name.ends_with(".zip") {
        Ok("zip".into())
    } else {
        Err(PrimitiveError::UnknownArchiveFormat {
            path: archive.into(),
        })
    }
}

fn extract_tar_gz(archive: &Path, dest: &Path) -> Result<Vec<String>> {
    let file = fs::File::open(archive).map_err(|source| PrimitiveError::Io {
        path: archive.into(),
        source,
    })?;
    let decoder = flate2::read::GzDecoder::new(file);
    let mut tar = tar::Archive::new(decoder);
    let mut files: Vec<String> = Vec::new();

    let entries = tar.entries().map_err(|source| PrimitiveError::Io {
        path: archive.into(),
        source,
    })?;
    for entry in entries {
        let mut entry = entry.map_err(|source| PrimitiveError::Io {
            path: archive.into(),
            source,
        })?;
        let entry_path = entry
            .path()
            .map_err(|source| PrimitiveError::Io {
                path: archive.into(),
                source,
            })?
            .to_path_buf();
        let safe = safe_join(dest, &entry_path)?;
        let kind = entry.header().entry_type();
        if kind.is_dir() {
            fs::create_dir_all(&safe).map_err(|source| PrimitiveError::Io {
                path: safe.clone(),
                source,
            })?;
            continue;
        }
        if !kind.is_file() {
            continue;
        }
        if let Some(parent) = safe.parent() {
            fs::create_dir_all(parent).map_err(|source| PrimitiveError::Io {
                path: parent.into(),
                source,
            })?;
        }
        let mut out = fs::File::create(&safe).map_err(|source| PrimitiveError::Io {
            path: safe.clone(),
            source,
        })?;
        std::io::copy(&mut entry, &mut out).map_err(|source| PrimitiveError::Io {
            path: safe.clone(),
            source,
        })?;
        files.push(entry_path.to_string_lossy().replace('\\', "/"));
    }
    Ok(files)
}

fn extract_zip(archive: &Path, dest: &Path) -> Result<Vec<String>> {
    let file = fs::File::open(archive).map_err(|source| PrimitiveError::Io {
        path: archive.into(),
        source,
    })?;
    let mut zip = zip::ZipArchive::new(file).map_err(zip_to_io(archive))?;
    let mut files: Vec<String> = Vec::new();
    for i in 0..zip.len() {
        let mut entry = zip.by_index(i).map_err(zip_to_io(archive))?;
        let raw_path = entry
            .enclosed_name()
            .ok_or_else(|| PrimitiveError::UnsafeArchivePath {
                entry: entry.name().to_string(),
            })?;
        let safe = safe_join(dest, &raw_path)?;
        if entry.is_dir() {
            fs::create_dir_all(&safe).map_err(|source| PrimitiveError::Io {
                path: safe.clone(),
                source,
            })?;
            continue;
        }
        if let Some(parent) = safe.parent() {
            fs::create_dir_all(parent).map_err(|source| PrimitiveError::Io {
                path: parent.into(),
                source,
            })?;
        }
        let mut out = fs::File::create(&safe).map_err(|source| PrimitiveError::Io {
            path: safe.clone(),
            source,
        })?;
        std::io::copy(&mut entry, &mut out).map_err(|source| PrimitiveError::Io {
            path: safe.clone(),
            source,
        })?;
        let mut buf: Vec<u8> = Vec::new();
        // Drain remaining bytes so the next iteration can advance the cursor.
        let _ = entry.read_to_end(&mut buf);
        files.push(raw_path.to_string_lossy().replace('\\', "/"));
    }
    Ok(files)
}

fn zip_to_io(archive: &Path) -> impl Fn(zip::result::ZipError) -> PrimitiveError + '_ {
    move |err| PrimitiveError::Io {
        path: archive.into(),
        source: std::io::Error::other(format!("zip error: {err}")),
    }
}

/// Join `dest` and an entry path safely. Rejects absolute paths,
/// `..` components, and paths that resolve outside `dest`. Returns the
/// fully resolved destination path on success.
pub(crate) fn safe_join(dest: &Path, entry: &Path) -> Result<PathBuf> {
    let entry_str = entry.to_string_lossy().to_string();
    let mut accumulated = PathBuf::new();
    for component in entry.components() {
        match component {
            Component::Normal(part) => accumulated.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(PrimitiveError::UnsafeArchivePath { entry: entry_str });
            }
        }
    }
    Ok(dest.join(accumulated))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use std::io::Write;

    fn make_tar_gz(path: &Path, entries: &[(&str, &[u8])]) {
        let file = fs::File::create(path).unwrap();
        let encoder = flate2::write::GzEncoder::new(file, flate2::Compression::default());
        let mut builder = tar::Builder::new(encoder);
        for (name, body) in entries {
            let mut header = tar::Header::new_gnu();
            header.set_size(body.len() as u64);
            header.set_mode(0o644);
            header.set_cksum();
            builder.append_data(&mut header, name, *body).unwrap();
        }
        builder.into_inner().unwrap().finish().unwrap();
    }

    fn make_zip(path: &Path, entries: &[(&str, &[u8])]) {
        let file = fs::File::create(path).unwrap();
        let mut writer = zip::ZipWriter::new(file);
        let options: zip::write::SimpleFileOptions = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Deflated);
        for (name, body) in entries {
            writer.start_file(*name, options).unwrap();
            writer.write_all(body).unwrap();
        }
        writer.finish().unwrap();
    }

    #[test]
    fn extracts_tar_gz_to_dest() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("test.tar.gz");
        make_tar_gz(&archive, &[("a.txt", b"alpha"), ("dir/b.txt", b"beta")]);

        let dest = tmp.path().join("out");
        let result = run(
            &ExtractArchiveArgs {
                archive: archive.to_string_lossy().into_owned(),
                dest: dest.to_string_lossy().into_owned(),
                format: None,
            },
            tmp.path(),
        )
        .unwrap();

        assert_eq!(result.format, "tar-gz");
        assert_eq!(result.count, 2);
        assert!(result.files.contains(&"a.txt".to_string()));
        assert!(result.files.contains(&"dir/b.txt".to_string()));
        assert_eq!(fs::read(dest.join("a.txt")).unwrap(), b"alpha");
        assert_eq!(fs::read(dest.join("dir/b.txt")).unwrap(), b"beta");
    }

    #[test]
    fn extracts_zip_to_dest() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("test.zip");
        make_zip(&archive, &[("a.txt", b"alpha"), ("dir/b.txt", b"beta")]);

        let dest = tmp.path().join("out");
        let result = run(
            &ExtractArchiveArgs {
                archive: archive.to_string_lossy().into_owned(),
                dest: dest.to_string_lossy().into_owned(),
                format: None,
            },
            tmp.path(),
        )
        .unwrap();

        assert_eq!(result.format, "zip");
        assert!(result.files.contains(&"a.txt".to_string()));
        assert!(result.files.contains(&"dir/b.txt".to_string()));
        assert_eq!(fs::read(dest.join("a.txt")).unwrap(), b"alpha");
        assert_eq!(fs::read(dest.join("dir/b.txt")).unwrap(), b"beta");
    }

    #[test]
    fn rejects_unknown_extension() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("test.bogus");
        fs::write(&archive, b"not an archive").unwrap();

        let err = run(
            &ExtractArchiveArgs {
                archive: archive.to_string_lossy().into_owned(),
                dest: tmp.path().join("out").to_string_lossy().into_owned(),
                format: None,
            },
            tmp.path(),
        )
        .unwrap_err();
        matches!(err, PrimitiveError::UnknownArchiveFormat { .. });
    }

    #[test]
    fn safe_join_rejects_parent_dir() {
        let dest = Path::new("/tmp/out");
        let err = safe_join(dest, Path::new("../etc/passwd")).unwrap_err();
        matches!(err, PrimitiveError::UnsafeArchivePath { .. });
    }

    #[test]
    fn safe_join_rejects_absolute() {
        let dest = Path::new("/tmp/out");
        let err = safe_join(dest, Path::new("/etc/passwd")).unwrap_err();
        matches!(err, PrimitiveError::UnsafeArchivePath { .. });
    }

    #[test]
    fn safe_join_accepts_relative_path() {
        let dest = Path::new("/tmp/out");
        let resolved = safe_join(dest, Path::new("sub/nested/file.txt")).unwrap();
        assert_eq!(resolved, Path::new("/tmp/out/sub/nested/file.txt"));
    }

    #[test]
    fn extract_with_format_override() {
        let tmp = tempfile::tempdir().unwrap();
        let archive = tmp.path().join("payload"); // no extension
        make_tar_gz(&archive, &[("a.txt", b"alpha")]);

        let dest = tmp.path().join("out");
        let result = run(
            &ExtractArchiveArgs {
                archive: archive.to_string_lossy().into_owned(),
                dest: dest.to_string_lossy().into_owned(),
                format: Some("tar-gz".into()),
            },
            tmp.path(),
        )
        .unwrap();
        assert_eq!(result.format, "tar-gz");
        assert_eq!(result.count, 1);
    }
}
