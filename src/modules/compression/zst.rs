use std::fs::File;
use std::io;

use super::ArchiveEntry;
use crate::contents::enums::FileType;
use crate::modules::progress::{CountingReader, Progress};

pub fn list(src_path: &std::path::Path) -> io::Result<(FileType, Vec<ArchiveEntry>)> {
    super::list_single_or_tar(src_path, FileType::Zst, FileType::Tarzst, || {
        zstd::Decoder::new(File::open(src_path)?)
    })
}

pub fn compress(src_path: &std::path::Path, dst_path: &std::path::Path, progress: Progress) {
    let dst = File::create(dst_path).expect("zst create failed");
    let mut enc = zstd::Encoder::new(dst, 0).expect("zst encoder failed");
    let src = File::open(src_path).expect("zst open failed");
    let mut reader = CountingReader::new(src, progress);
    io::copy(&mut reader, &mut enc).expect("zst compress failed");
    enc.finish().expect("zst finish failed");
}

pub fn decompress(src_path: &std::path::Path, dst_path: &std::path::Path, progress: Progress) {
    let src = File::open(src_path).expect("zst open failed");
    let mut dec =
        zstd::Decoder::new(CountingReader::new(src, progress)).expect("zst decoder failed");
    let mut dst = File::create(dst_path).expect("zst create dst failed");
    io::copy(&mut dec, &mut dst).expect("zst unpack failed");
}
