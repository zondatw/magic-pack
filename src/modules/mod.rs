mod compression;

use crate::enums;

pub fn compress(
    file_type: enums::FileType,
    src_path: &std::string::String,
    dst_path: &std::string::String,
) {
    match file_type {
        enums::FileType::Zip => {
            compression::zip::compress(src_path, dst_path);
        }
        enums::FileType::Tar => {
            compression::tar::compress(src_path, dst_path);
        }
        enums::FileType::Tarbz2 => {
            compression::tar_bz2::compress(src_path, dst_path);
        }
        enums::FileType::Targz => {
            compression::tar_gz::compress(src_path, dst_path);
        }
    }
}

pub fn decompress(
    file_type: enums::FileType,
    src_path: &std::string::String,
    dst_path: &std::string::String,
) {
    match file_type {
        enums::FileType::Zip => {
            println!("zip");
            compression::zip::decompress(src_path, dst_path);
        }
        enums::FileType::Tar => {
            println!("tar");
            compression::tar::decompress(src_path, dst_path);
        }
        enums::FileType::Tarbz2 => {
            println!("tar.bz2");
            compression::tar_bz2::decompress(src_path, dst_path);
        }
        enums::FileType::Targz => {
            println!("tar.gz");
            compression::tar_gz::decompress(src_path, dst_path);
        }
    }
}
