use std::fs::File;
use std::io::{Read, Write};

use xz2::read::XzDecoder;
use xz2::write::XzEncoder;

pub fn compress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    let xz_file = File::create(dst_path).expect("xz create failed");
    let mut enc = XzEncoder::new(xz_file, 6);

    let mut src_file = File::open(src_path).expect("xz open failed");
    let mut content = Vec::new();
    src_file.read_to_end(&mut content).expect("xz read failed");
    enc.write_all(content.as_slice()).expect("xz write failed");
    enc.finish().expect("xz finish failed");
}

pub fn decompress(src_path: &std::path::Path, dst_path: &std::path::Path) {
    let xz_file = File::open(src_path).expect("xz open failed");
    let dec = XzDecoder::new(xz_file);
    let mut reader = std::io::BufReader::new(dec);
    let mut content = Vec::new();
    reader.read_to_end(&mut content).expect("xz unpack failed");
    let mut dst_file = File::create(dst_path).expect("xz create dst failed");
    dst_file
        .write_all(content.as_slice())
        .expect("xz write dst failed");
}
