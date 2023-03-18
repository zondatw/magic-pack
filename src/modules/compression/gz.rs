use std::fs::File;
use std::io::{Write, Read};

use flate2;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;

pub fn compress(src_path: &std::string::String, dst_path: &std::string::String) {
    let gz_file = File::create(dst_path).expect("gz create failed");
    let mut enc = GzEncoder::new(gz_file, flate2::Compression::default());

    let mut src_file = File::open(src_path).expect("gz open failed");
    let mut content = Vec::new();
    src_file.read_to_end(&mut content).expect("gz open failed");
    enc.write_all(content.as_slice()).expect("gz open failed");
    enc.finish().expect("gz open failed");
}

pub fn decompress(src_path: &std::string::String, dst_path: &std::string::String) {
    let gz_file = File::open(src_path).expect("gz open failed");
    let dec = GzDecoder::new(gz_file);
    let mut reader = std::io::BufReader::new(dec);
    let mut content = Vec::new();
    reader.read_to_end(&mut content).expect("gz unpack failed");
    let mut dst_file = File::create(dst_path).expect("gz unpack failed");
    dst_file.write_all(content.as_slice()).expect("gz unpack failed");
}
