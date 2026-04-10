use std::ffi::OsString;
use std::fs::File;
use std::path::{Path, PathBuf};

use sevenz_rust::{SevenZArchiveEntry, SevenZWriter};
use walkdir::WalkDir;

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

pub fn compress(src_path: &Path, dst_path: &Path) {
    let dst_file = File::create(dst_path).expect("7z create failed");
    let mut writer = SevenZWriter::new(dst_file).expect("7z writer init failed");

    for entry in WalkDir::new(src_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = archive_path(src_path, path).to_string_lossy().to_string();

        if path.is_dir() {
            writer
                .push_archive_entry::<File>(SevenZArchiveEntry::from_path(path, name), None)
                .expect("7z add dir failed");
        } else {
            let file = File::open(path).expect("7z open source file failed");
            writer
                .push_archive_entry(SevenZArchiveEntry::from_path(path, name), Some(file))
                .expect("7z add file failed");
        }
    }

    writer.finish().expect("7z finish failed");
}

pub fn decompress(src_path: &Path, dst_path: &Path) {
    sevenz_rust::decompress_file(src_path, dst_path).expect("7z decompress failed");
}
