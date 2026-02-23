use std::fs::File;
use std::io::{Seek, Write};

use walkdir::{DirEntry, WalkDir};

use tar;
use tar::Archive;

use crate::utils::is_safe_path;

fn tar_dir<T>(it: &mut dyn Iterator<Item = DirEntry>, tar_file: T)
where
    T: Write + Seek,
{
    let mut tar_builder = tar::Builder::new(tar_file);
    for entry in it {
        let path = entry.path();
        tar_builder.append_path(path).expect("tar append failed");
    }
}

pub fn compress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    let tar_file = File::create(dst_path).expect("tar create failed");
    let walkdir = WalkDir::new(src_path);
    let it = walkdir.into_iter();
    tar_dir(&mut it.filter_map(|e| e.ok()), tar_file);
}

pub fn decompress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    let tar_file = File::open(src_path).expect("tar open failed");
    let mut archive = Archive::new(tar_file);
    for entry in archive.entries().expect("tar entries failed") {
        let mut entry = entry.expect("tar entry failed");
        let entry_path = entry.path().expect("tar entry path failed");
        if !is_safe_path(&entry_path) {
            panic!("tar entry path traversal detected");
        }
        entry
            .unpack_in(dst_path)
            .expect("tar unpack failed");
    }
}
