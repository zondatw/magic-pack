mod compression;

use std::fs::File;
use std::io::{Error, ErrorKind, Read};

use crate::enums;

pub fn get_file_type(file_path: &std::path::PathBuf) -> Result<enums::FileType, std::io::Error> {
    struct CompressMagic {
        magic_number: &'static [u8],
        length: usize,
        file_type: enums::FileType,
    }

    static COMPRESS_MAGIC_LIST: [CompressMagic; 3] = [
        CompressMagic {
            magic_number: b"BZh",
            length: 3,
            file_type: enums::FileType::Bz2,
        },
        CompressMagic {
            magic_number: &[0x1f, 0x8b],
            length: 2,
            file_type: enums::FileType::Gz,
        },
        CompressMagic {
            magic_number: &[0x50, 0x4b, 0x03, 0x04],
            length: 4,
            file_type: enums::FileType::Zip,
        },
    ];

    let mut buffer = [0u8; 4];
    let mut file = File::open(file_path).expect("file open failed");
    file.read_exact(&mut buffer).expect("read file failed");

    for compress_magic in COMPRESS_MAGIC_LIST.iter() {
        if buffer.get(..compress_magic.length).unwrap() == compress_magic.magic_number {
            return Ok(compress_magic.file_type);
        }
    }
    Err(Error::from(ErrorKind::Unsupported))
}

pub fn compress(
    file_type: enums::FileType,
    src_path: &std::path::PathBuf,
    dst_path: &std::path::PathBuf,
) {
    let src_path_str = src_path.to_owned().into_os_string().into_string().unwrap();
    let dst_path_str = dst_path.to_owned().into_os_string().into_string().unwrap();
    match file_type {
        enums::FileType::Zip => {
            compression::zip::compress(&src_path_str, &dst_path_str);
        }
        enums::FileType::Tar => {
            compression::tar::compress(&src_path_str, &dst_path_str);
        }
        enums::FileType::Bz2 => {
            compression::bz2::compress(&src_path_str, &dst_path_str);
        }
        enums::FileType::Gz => {
            compression::gz::compress(&src_path_str, &dst_path_str);
        }
        enums::FileType::Tarbz2 => {
            compression::tar_bz2::compress(&src_path_str, &dst_path_str);
        }
        enums::FileType::Targz => {
            compression::tar_gz::compress(&src_path_str, &dst_path_str);
        }
    }
}

pub fn decompress(
    file_type: enums::FileType,
    src_path: &std::path::PathBuf,
    dst_path: &std::path::PathBuf,
) {
    let src_path_str = src_path.to_owned().into_os_string().into_string().unwrap();
    let dst_path_str = dst_path.to_owned().into_os_string().into_string().unwrap();
    match file_type {
        enums::FileType::Zip => {
            println!("zip");
            compression::zip::decompress(&src_path_str, &dst_path_str);
        }
        enums::FileType::Tar => {
            println!("tar");
            compression::tar::decompress(&src_path_str, &dst_path_str);
        }
        enums::FileType::Tarbz2 => {
            println!("tar.bz2");
            compression::tar_bz2::decompress(&src_path_str, &dst_path_str);
        }
        enums::FileType::Targz => {
            println!("tar.gz");
            compression::tar_gz::decompress(&src_path_str, &dst_path_str);
        }
        enums::FileType::Bz2 => {
            println!("bz2");
            compression::bz2::decompress(&src_path_str, &dst_path_str);
        }
        enums::FileType::Gz => {
            println!("gz");
            compression::gz::decompress(&src_path_str, &dst_path_str);
        }
    }
}
