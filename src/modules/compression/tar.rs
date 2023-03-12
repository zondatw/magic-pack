use std::fs::File;
use std::io::{Seek, Write};

use walkdir::{DirEntry, WalkDir};

use tar;
use tar::Archive;

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

pub fn compress(src_path: &std::string::String, dst_path: &std::string::String) {
    let tar_file = File::create(dst_path).expect("tar create failed");
    let walkdir = WalkDir::new(src_path);
    let it = walkdir.into_iter();
    tar_dir(&mut it.filter_map(|e| e.ok()), tar_file);
}

pub fn decompress(src_path: &std::string::String, dst_path: &std::string::String) {
    let tar_file = File::open(src_path).expect("tar open failed");
    let mut archive = Archive::new(tar_file);
    archive.unpack(dst_path).expect("tar unpack failed");
}
