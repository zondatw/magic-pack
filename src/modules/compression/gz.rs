use std::fs::File;
use std::io;

use flate2;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

use super::ArchiveEntry;
use crate::contents::enums::FileType;
use crate::modules::progress::{CountingReader, Progress};

pub fn list(src_path: &std::path::Path) -> io::Result<(FileType, Vec<ArchiveEntry>)> {
    super::list_single_or_tar(src_path, FileType::Gz, FileType::Targz, || {
        Ok(GzDecoder::new(File::open(src_path)?))
    })
}

pub fn compress(src_path: &std::path::Path, dst_path: &std::path::Path, progress: Progress) {
    let dst = File::create(dst_path).expect("gz create failed");
    let mut enc = GzEncoder::new(dst, flate2::Compression::default());
    let src = File::open(src_path).expect("gz open failed");
    let mut reader = CountingReader::new(src, progress);
    io::copy(&mut reader, &mut enc).expect("gz compress failed");
    enc.finish().expect("gz finish failed");
}

pub fn decompress(src_path: &std::path::Path, dst_path: &std::path::Path, progress: Progress) {
    let src = File::open(src_path).expect("gz open failed");
    let mut dec = GzDecoder::new(CountingReader::new(src, progress));
    let mut dst = File::create(dst_path).expect("gz unpack failed");
    io::copy(&mut dec, &mut dst).expect("gz unpack failed");
}
