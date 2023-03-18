mod compression;

use std::io::{Error, ErrorKind};
use std::process::Command;

use crate::enums;

pub fn get_file_type(file_path: &std::string::String) -> Result<enums::FileType, std::io::Error> {
    let output = Command::new("file")
        .arg(file_path)
        .output()
        .expect("file command failed");

    if !output.status.success() {
        panic!("file command failed");
    }

    let file_type = String::from_utf8(output.stdout).unwrap();
    match file_type {
        s if s.contains("Zip") => Ok(enums::FileType::Zip),
        s if s.contains("POSIX tar archive") => Ok(enums::FileType::Tar),
        s if s.contains("gzip") => Ok(enums::FileType::Gz),
        s if s.contains("bzip2") => Ok(enums::FileType::Bz2),
        _ => Err(Error::from(ErrorKind::Unsupported)),
    }
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
