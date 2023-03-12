use std::fs::File;
use std::io::{Seek, Write};

use flate2;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use tar;
use tar::Archive;
use walkdir::{DirEntry, WalkDir};

fn tar_gz_dir<T>(it: &mut dyn Iterator<Item = DirEntry>, tar_gz_file: T)
where
    T: Write + Seek,
{
    let enc = GzEncoder::new(tar_gz_file, flate2::Compression::default());
    let mut tar_gz_builder = tar::Builder::new(enc);
    for entry in it {
        let path = entry.path();
        tar_gz_builder
            .append_path(path)
            .expect("tar.gz append failed");
    }
}

pub fn compress(src_path: &std::string::String, dst_path: &std::string::String) {
    let tar_gz_file = File::create(dst_path).expect("tar.gz create failed");
    let walkdir = WalkDir::new(src_path);
    let it = walkdir.into_iter();
    tar_gz_dir(&mut it.filter_map(|e| e.ok()), tar_gz_file);
}

pub fn decompress(src_path: &std::string::String, dst_path: &std::string::String) {
    let tar_gz_file = File::open(src_path).expect("tar.gz open failed");
    let dec = GzDecoder::new(tar_gz_file);
    let mut archive = Archive::new(dec);
    archive.unpack(dst_path).expect("tar.gz unpack failed");
}
