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

fn tar_zst_dir<T>(it: &mut dyn Iterator<Item = DirEntry>, dst_file: T, src_root: &Path)
where
    T: Write,
{
    let enc = zstd::Encoder::new(dst_file, 0).expect("zst encoder failed");
    let mut builder = tar::Builder::new(enc);
    for entry in it {
        let path = entry.path();
        let name = archive_path(src_root, path);
        builder
            .append_path_with_name(path, &name)
            .expect("tar.zst append failed");
    }
    let enc = builder.into_inner().expect("tar.zst finish failed");
    enc.finish().expect("zst finish failed");
}

pub fn compress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    let dst_file = File::create(dst_path).expect("tar.zst create failed");
    let walkdir = WalkDir::new(src_path);
    let it = walkdir.into_iter();
    tar_zst_dir(&mut it.filter_map(|e| e.ok()), dst_file, src_path);
}

pub fn decompress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    let src_file = File::open(src_path).expect("tar.zst open failed");
    let dec = zstd::Decoder::new(src_file).expect("zst decoder failed");
    let mut archive = Archive::new(dec);
    for entry in archive.entries().expect("tar.zst entries failed") {
        let mut entry = entry.expect("tar.zst entry failed");
        let entry_path = entry.path().expect("tar.zst entry path failed");
        if !is_safe_path(&entry_path) {
            panic!("tar.zst entry path traversal detected");
        }
        entry.unpack_in(dst_path).expect("tar.zst unpack failed");
    }
}
