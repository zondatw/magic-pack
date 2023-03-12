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

fn main() {
    let args = Args::new();

    println!("Input path: {:?}", args.input);
    println!("Output path: {:?}", args.output);

    if args.compress {
        println!("Compress");
        modules::compress(args.file_type.unwrap(), &args.input, &args.output);
    }
    if args.decompress {
        println!("Decompress");
        let file_type = get_file_type(&args.input);
        modules::decompress(file_type, &args.input, &args.output);
    }
}
