use std::fs::File;
use std::io;

use xz2::read::XzDecoder;
use xz2::write::XzEncoder;

use crate::modules::progress::{CountingReader, Progress};

pub fn compress(src_path: &std::path::Path, dst_path: &std::path::Path, progress: Progress) {
    let dst = File::create(dst_path).expect("xz create failed");
    let mut enc = XzEncoder::new(dst, 6);
    let src = File::open(src_path).expect("xz open failed");
    let mut reader = CountingReader::new(src, progress);
    io::copy(&mut reader, &mut enc).expect("xz compress failed");
    enc.finish().expect("xz finish failed");
}

pub fn decompress(src_path: &std::path::Path, dst_path: &std::path::Path, progress: Progress) {
    let src = File::open(src_path).expect("xz open failed");
    let mut dec = XzDecoder::new(CountingReader::new(src, progress));
    let mut dst = File::create(dst_path).expect("xz create dst failed");
    io::copy(&mut dec, &mut dst).expect("xz unpack failed");
}
