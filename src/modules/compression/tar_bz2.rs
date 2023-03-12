use std::fs::File;
use std::io::{Seek, Write};

use bzip2;
use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;
use tar;
use tar::Archive;
use walkdir::{DirEntry, WalkDir};

fn tar_bz2_dir<T>(it: &mut dyn Iterator<Item = DirEntry>, tar_bz2_file: T)
where
    T: Write + Seek,
{
    let enc = BzEncoder::new(tar_bz2_file, bzip2::Compression::default());
    let mut tar_bz2_builder = tar::Builder::new(enc);
    for entry in it {
        let path = entry.path();
        tar_bz2_builder
            .append_path(path)
            .expect("tar.bz2 append failed");
    }
}

pub fn compress(src_path: &std::string::String, dst_path: &std::string::String) {
    let tar_bz2_file = File::create(dst_path).expect("tar.bz2 create failed");
    let walkdir = WalkDir::new(src_path);
    let it = walkdir.into_iter();
    tar_bz2_dir(&mut it.filter_map(|e| e.ok()), tar_bz2_file);
}

pub fn decompress(src_path: &std::string::String, dst_path: &std::string::String) {
    let tar_bz2_file = File::open(src_path).expect("tar.bz2 open failed");
    let dec = BzDecoder::new(tar_bz2_file);
    let mut archive = Archive::new(dec);
    archive.unpack(dst_path).expect("tar.bz2 unpack failed");
}
