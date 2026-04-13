use std::ffi::OsString;
use std::fs::File;
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};

use tar;
use tar::Archive;
use walkdir::{DirEntry, WalkDir};
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;

use crate::utils::is_safe_path;

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

fn tar_xz_dir<T>(it: &mut dyn Iterator<Item = DirEntry>, tar_xz_file: T, src_root: &Path)
where
    T: Write + Seek,
{
    let enc = XzEncoder::new(tar_xz_file, 6);
    let mut tar_xz_builder = tar::Builder::new(enc);
    for entry in it {
        let path = entry.path();
        let name = archive_path(src_root, path);
        tar_xz_builder
            .append_path_with_name(path, &name)
            .expect("tar.xz append failed");
    }
}

pub fn compress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    let tar_xz_file = File::create(dst_path).expect("tar.xz create failed");
    let walkdir = WalkDir::new(src_path);
    let it = walkdir.into_iter();
    tar_xz_dir(&mut it.filter_map(|e| e.ok()), tar_xz_file, src_path);
}

pub fn decompress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    std::fs::create_dir_all(dst_path).expect("tar.xz create dst dir failed");
    let tar_xz_file = File::open(src_path).expect("tar.xz open failed");
    let dec = XzDecoder::new(tar_xz_file);
    let mut archive = Archive::new(dec);
    for entry in archive.entries().expect("tar.xz entries failed") {
        let mut entry = entry.expect("tar.xz entry failed");
        let entry_path = entry.path().expect("tar.xz entry path failed");
        if !is_safe_path(&entry_path) {
            panic!("tar.xz entry path traversal detected");
        }
        entry.unpack_in(dst_path).expect("tar.xz unpack failed");
    }
}
