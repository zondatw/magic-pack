use std::fs::File;
use std::io::{Read, Write};

pub fn compress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    let dst_file = File::create(dst_path).expect("zst create failed");
    let mut enc = zstd::Encoder::new(dst_file, 0).expect("zst encoder failed");

    let mut src_file = File::open(src_path).expect("zst open failed");
    let mut content = Vec::new();
    src_file.read_to_end(&mut content).expect("zst read failed");
    enc.write_all(&content).expect("zst write failed");
    enc.finish().expect("zst finish failed");
}

pub fn decompress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    let src_file = File::open(src_path).expect("zst open failed");
    let mut dec = zstd::Decoder::new(src_file).expect("zst decoder failed");
    let mut content = Vec::new();
    dec.read_to_end(&mut content).expect("zst read failed");
    let mut dst_file = File::create(dst_path).expect("zst create dst failed");
    dst_file.write_all(&content).expect("zst write dst failed");
}
