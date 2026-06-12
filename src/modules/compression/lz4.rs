use std::fs::File;
use std::io;

use crate::modules::progress::{CountingReader, Progress};

pub fn compress(src_path: &std::path::Path, dst_path: &std::path::Path, progress: Progress) {
    let dst = File::create(dst_path).expect("lz4 create failed");
    let mut enc = lz4_flex::frame::FrameEncoder::new(dst);
    let src = File::open(src_path).expect("lz4 open failed");
    let mut reader = CountingReader::new(src, progress);
    io::copy(&mut reader, &mut enc).expect("lz4 compress failed");
    enc.finish().expect("lz4 finish failed");
}

pub fn decompress(src_path: &std::path::Path, dst_path: &std::path::Path, progress: Progress) {
    let src = File::open(src_path).expect("lz4 open failed");
    let mut dec = lz4_flex::frame::FrameDecoder::new(CountingReader::new(src, progress));
    let mut dst = File::create(dst_path).expect("lz4 create dst failed");
    io::copy(&mut dec, &mut dst).expect("lz4 unpack failed");
}
