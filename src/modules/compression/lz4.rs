use std::fs::File;
use std::io::{Read, Write};

pub fn compress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    let dst_file = File::create(dst_path).expect("lz4 create failed");
    let mut enc = lz4_flex::frame::FrameEncoder::new(dst_file);

    let mut src_file = File::open(src_path).expect("lz4 open failed");
    let mut content = Vec::new();
    src_file.read_to_end(&mut content).expect("lz4 read failed");
    enc.write_all(&content).expect("lz4 write failed");
    enc.finish().expect("lz4 finish failed");
}

pub fn decompress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    let src_file = File::open(src_path).expect("lz4 open failed");
    let mut dec = lz4_flex::frame::FrameDecoder::new(src_file);
    let mut content = Vec::new();
    dec.read_to_end(&mut content).expect("lz4 read failed");
    let mut dst_file = File::create(dst_path).expect("lz4 create dst failed");
    dst_file.write_all(&content).expect("lz4 write dst failed");
}
