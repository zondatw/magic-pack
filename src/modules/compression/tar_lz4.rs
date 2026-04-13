use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

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

fn tar_lz4_dir<T>(it: &mut dyn Iterator<Item = DirEntry>, dst_file: T, src_root: &Path)
where
    T: Write,
{
    let enc = lz4_flex::frame::FrameEncoder::new(dst_file);
    let mut builder = tar::Builder::new(enc);
    for entry in it {
        let path = entry.path();
        let name = archive_path(src_root, path);
        builder
            .append_path_with_name(path, &name)
            .expect("tar.lz4 append failed");
    }
    let enc = builder.into_inner().expect("tar.lz4 finish failed");
    enc.finish().expect("lz4 finish failed");
}

pub fn compress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    let dst_file = File::create(dst_path).expect("tar.lz4 create failed");
    let walkdir = WalkDir::new(src_path);
    let it = walkdir.into_iter();
    tar_lz4_dir(&mut it.filter_map(|e| e.ok()), dst_file, src_path);
}

pub fn decompress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    std::fs::create_dir_all(dst_path).expect("tar.lz4 create dst dir failed");
    let src_file = File::open(src_path).expect("tar.lz4 open failed");
    let dec = lz4_flex::frame::FrameDecoder::new(src_file);
    let mut archive = Archive::new(dec);
    for entry in archive.entries().expect("tar.lz4 entries failed") {
        let mut entry = entry.expect("tar.lz4 entry failed");
        let entry_path = entry.path().expect("tar.lz4 entry path failed");
        if !is_safe_path(&entry_path) {
            panic!("tar.lz4 entry path traversal detected");
        }
        entry.unpack_in(dst_path).expect("tar.lz4 unpack failed");
    }
}
