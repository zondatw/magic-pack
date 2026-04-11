use std::fs::File;
use std::io::{Read, Write};

pub fn compress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    let dst_file = File::create(dst_path).expect("lz4 create failed");
    let mut enc = lz4::EncoderBuilder::new()
        .build(dst_file)
        .expect("lz4 encoder failed");

    let mut src_file = File::open(src_path).expect("lz4 open failed");
    let mut content = Vec::new();
    src_file.read_to_end(&mut content).expect("lz4 read failed");
    enc.write_all(&content).expect("lz4 write failed");
    let (_, result) = enc.finish();
    result.expect("lz4 finish failed");
}

pub fn decompress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    let src_file = File::open(src_path).expect("lz4 open failed");
    let mut dec = lz4::Decoder::new(src_file).expect("lz4 decoder failed");
    let mut content = Vec::new();
    dec.read_to_end(&mut content).expect("lz4 read failed");
    let mut dst_file = File::create(dst_path).expect("lz4 create dst failed");
    dst_file.write_all(&content).expect("lz4 write dst failed");
}
