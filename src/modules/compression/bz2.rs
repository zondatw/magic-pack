use std::fs::File;
use std::io;

use bzip2;
use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;

use super::ArchiveEntry;
use crate::contents::enums::FileType;
use crate::modules::progress::{CountingReader, Progress};

pub fn list(src_path: &std::path::Path) -> io::Result<(FileType, Vec<ArchiveEntry>)> {
    super::list_single_or_tar(src_path, FileType::Bz2, FileType::Tarbz2, || {
        Ok(BzDecoder::new(File::open(src_path)?))
    })
}

pub fn compress(src_path: &std::path::Path, dst_path: &std::path::Path, progress: Progress) {
    let dst = File::create(dst_path).expect("bz2 create failed");
    let mut enc = BzEncoder::new(dst, bzip2::Compression::default());
    let src = File::open(src_path).expect("bz2 open failed");
    let mut reader = CountingReader::new(src, progress);
    io::copy(&mut reader, &mut enc).expect("bz2 compress failed");
    enc.finish().expect("bz2 finish failed");
}

pub fn decompress(src_path: &std::path::Path, dst_path: &std::path::Path, progress: Progress) {
    let src = File::open(src_path).expect("bz2 open failed");
    let mut dec = BzDecoder::new(CountingReader::new(src, progress));
    let mut dst = File::create(dst_path).expect("bz2 unpack failed");
    io::copy(&mut dec, &mut dst).expect("bz2 unpack failed");
}
