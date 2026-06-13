use std::ffi::OsString;
use std::fs::File;
use std::io::{self, Read, Seek, Write};
use std::path::{Path, PathBuf};

use walkdir::{DirEntry, WalkDir};

use tar;
use tar::Archive;

use super::ArchiveEntry;
use crate::modules::progress::{add, CountingReader, Progress};
use crate::utils::is_safe_path;

/// List tar entry headers from an already-decompressed reader, without
/// unpacking to disk. Shared by `tar` and every `tar.*` variant.
pub(super) fn list_reader<R: Read>(reader: R) -> io::Result<Vec<ArchiveEntry>> {
    let mut archive = Archive::new(reader);
    let mut entries = Vec::new();
    for entry in archive.entries()? {
        let entry = entry?;
        let header = entry.header();
        let size = header.size().unwrap_or(0);
        let is_dir = header.entry_type().is_dir();
        let name = entry
            .path()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        entries.push(ArchiveEntry { name, size, is_dir });
    }
    Ok(entries)
}

pub fn list(src_path: &Path) -> io::Result<Vec<ArchiveEntry>> {
    list_reader(File::open(src_path)?)
}

fn archive_path(src_root: &Path, entry_path: &Path) -> PathBuf {
    let base: Option<OsString> = src_root.file_name().map(|s| s.to_os_string());
    if entry_path == src_root {
        if let Some(base) = base {
            return PathBuf::from(base);
        }
    }
    match entry_path.strip_prefix(src_root) {
        Ok(rel) => match base {
            Some(base) => PathBuf::from(base).join(rel),
            None => rel.to_path_buf(),
        },
        Err(_) => entry_path.to_path_buf(),
    }
}

fn tar_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    tar_file: T,
    src_root: &Path,
    progress: Progress,
) where
    T: Write + Seek,
{
    let mut tar_builder = tar::Builder::new(tar_file);
    for entry in it {
        let path = entry.path();
        let name = archive_path(src_root, path);
        tar_builder
            .append_path_with_name(path, &name)
            .expect("tar append failed");
        if entry.file_type().is_file() {
            if let Ok(meta) = entry.metadata() {
                add(progress, meta.len());
            }
        }
    }
}

pub fn compress(src_path: &std::path::Path, dst_path: &std::path::Path, progress: Progress) {
    let tar_file = File::create(dst_path).expect("tar create failed");
    let walkdir = WalkDir::new(src_path);
    let it = walkdir.into_iter();
    tar_dir(&mut it.filter_map(|e| e.ok()), tar_file, src_path, progress);
}

pub fn decompress(src_path: &std::path::Path, dst_path: &std::path::Path, progress: Progress) {
    std::fs::create_dir_all(dst_path).expect("tar create dst dir failed");
    let tar_file = File::open(src_path).expect("tar open failed");
    let mut archive = Archive::new(CountingReader::new(tar_file, progress));
    for entry in archive.entries().expect("tar entries failed") {
        let mut entry = entry.expect("tar entry failed");
        let entry_path = entry.path().expect("tar entry path failed");
        if !is_safe_path(&entry_path) {
            panic!("tar entry path traversal detected");
        }
        entry.unpack_in(dst_path).expect("tar unpack failed");
    }
}
