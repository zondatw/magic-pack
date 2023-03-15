use std::fs::File;
use std::io::{Write, Read};

use bzip2;
use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;

pub fn compress(src_path: &std::string::String, dst_path: &std::string::String) {
    let bz2_file = File::create(dst_path).expect("bz2 create failed");
    let mut enc = BzEncoder::new(bz2_file, bzip2::Compression::default());

    let mut src_file = File::open(src_path).expect("bz2 open failed");
    let mut content = Vec::new();
    src_file.read_to_end(&mut content).expect("bz2 open failed");
    enc.write_all(content.as_slice()).expect("bz2 open failed");
    enc.finish().expect("bz2 open failed");
}

pub fn decompress(src_path: &std::string::String, dst_path: &std::string::String) {
    let bz2_file = File::open(src_path).expect("bz2 open failed");
    let dec = BzDecoder::new(bz2_file);
    let mut reader = std::io::BufReader::new(dec);
    let mut content = Vec::new();
    reader.read_to_end(&mut content).expect("bz2 unpack failed");
    let mut dst_file = File::create(dst_path).expect("bz2 unpack failed");
    dst_file.write_all(content.as_slice());
}
