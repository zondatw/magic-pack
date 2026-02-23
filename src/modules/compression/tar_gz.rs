use std::ffi::OsString;
use std::fs::File;
use std::io::{Seek, Write};
use std::path::{Path, PathBuf};

use flate2;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use tar;
use tar::Archive;
use walkdir::{DirEntry, WalkDir};

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

fn tar_gz_dir<T>(it: &mut dyn Iterator<Item = DirEntry>, tar_gz_file: T, src_root: &Path)
where
    T: Write + Seek,
{
    let enc = GzEncoder::new(tar_gz_file, flate2::Compression::default());
    let mut tar_gz_builder = tar::Builder::new(enc);
    for entry in it {
        let path = entry.path();
        let name = archive_path(src_root, path);
        tar_gz_builder
            .append_path_with_name(path, &name)
            .expect("tar.gz append failed");
    }
}

pub fn compress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    let tar_gz_file = File::create(dst_path).expect("tar.gz create failed");
    let walkdir = WalkDir::new(src_path);
    let it = walkdir.into_iter();
    tar_gz_dir(&mut it.filter_map(|e| e.ok()), tar_gz_file, src_path);
}

pub fn decompress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    let tar_gz_file = File::open(src_path).expect("tar.gz open failed");
    let dec = GzDecoder::new(tar_gz_file);
    let mut archive = Archive::new(dec);
    for entry in archive.entries().expect("tar.gz entries failed") {
        let mut entry = entry.expect("tar.gz entry failed");
        let entry_path = entry.path().expect("tar.gz entry path failed");
        if !is_safe_path(&entry_path) {
            panic!("tar.gz entry path traversal detected");
        }
        entry
            .unpack_in(dst_path)
            .expect("tar.gz unpack failed");
    }
}
