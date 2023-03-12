mod cli;
mod contents;
mod modules;

use std::process::Command;

use crate::cli::Args;
use crate::contents::enums;

fn get_file_type(file_path: &std::string::String) -> enums::FileType {
    let output = Command::new("file")
        .arg(file_path)
        .output()
        .expect("file command failed");

    if !output.status.success() {
        panic!("file command failed");
    }

    let file_type = String::from_utf8(output.stdout).unwrap();
    match file_type {
        s if s.contains("Zip") => enums::FileType::Zip,
        s if s.contains("POSIX tar archive") => enums::FileType::Tar,
        s if s.contains("gzip") => enums::FileType::Targz,
        s if s.contains("bzip2") => enums::FileType::Tarbz2,
        _ => panic!("no supported"),
    }
}

fn pack(
    file_type: enums::FileType,
    src_path: &std::string::String,
    dst_path: &std::string::String,
) {
    match file_type {
        enums::FileType::Zip => {
            println!("Zip");
            modules::compression::zip::compress(src_path, dst_path);
        }
        enums::FileType::Tar => {
            println!("Tar");
            modules::compression::tar::compress(src_path, dst_path);
        }
        enums::FileType::Tarbz2 => {
            println!("Tarbz2");
            modules::compression::tar_bz2::compress(src_path, dst_path);
        }
        enums::FileType::Targz => {
            println!("Targz");
            modules::compression::tar_gz::compress(src_path, dst_path);
        }
    }
}

fn unpack(
    file_type: enums::FileType,
    src_path: &std::string::String,
    dst_path: &std::string::String,
) {
    match file_type {
        enums::FileType::Zip => {
            println!("Zip");
            modules::compression::zip::decompress(src_path, dst_path);
        }
        enums::FileType::Tar => {
            println!("Tar");
            modules::compression::tar::decompress(src_path, dst_path);
        }
        enums::FileType::Tarbz2 => {
            println!("Tarbz2");
            modules::compression::tar_bz2::decompress(src_path, dst_path);
        }
        enums::FileType::Targz => {
            println!("Targz");
            modules::compression::tar_gz::decompress(src_path, dst_path);
        }
    }
}

fn main() {
    let args = Args::new();

    println!("Input path: {:?}", args.input);
    println!("Output path: {:?}", args.output);

    if args.compress {
        println!("Compress");
        pack(args.file_type.unwrap(), &args.input, &args.output);
    }
    if args.decompress {
        println!("Decompress");
        let file_type = get_file_type(&args.input);
        unpack(file_type, &args.input, &args.output);
    }
}
