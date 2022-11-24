use std::io::{self, Write};
use std::process::Command;

use clap::{ArgGroup, Parser, ValueEnum};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("functional")
        .required(true)
        .args(["compress", "decompress"]),
))]
struct Args {
    /// Compress flag
    #[arg(short, long, requires = "file_type")]
    compress: bool,

    // file type
    #[arg(short, value_enum)]
    file_type: Option<FileType>,

    /// Decompress flag
    #[arg(short, long)]
    decompress: bool,

    // file / directory input path
    #[arg(short)]
    input: String,

    // file / directory output path
    #[arg(short)]
    output: String,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum FileType {
    Zip,
    Tar,
    Tarbz2,
    Targz,
}

fn get_file_type(file_path: String) -> FileType {
    let output = Command::new("file")
        .arg(file_path)
        .output()
        .expect("file command failed");

    if !output.status.success() {
        panic!("file command failed");
    }

    let file_type = String::from_utf8(output.stdout).unwrap();
    match file_type {
        s if s.contains("Zip") => FileType::Zip,
        s if s.contains("POSIX tar archive") => FileType::Tar,
        s if s.contains("gzip") => FileType::Targz,
        s if s.contains("bzip2") => FileType::Tarbz2,
        _ => panic!("no supported"),
    }
}

fn main() {
    let args = Args::parse();

    println!("Input path: {:?}", args.input);
    println!("Output path: {:?}", args.output);

    if args.compress {
        println!("Compress");

        match args.file_type.unwrap() {
            FileType::Zip => {
                println!("Zip");
            }
            FileType::Tar => {
                println!("Tar");
            }
            FileType::Tarbz2 => {
                println!("Tarbz2");
            }
            FileType::Targz => {
                println!("Targz");
            }
        }
    }
    if args.decompress {
        println!("Decompress");
        let file_type = get_file_type(args.input);

        match file_type {
            FileType::Zip => {
                println!("Zip");
            }
            FileType::Tar => {
                println!("Tar");
            }
            FileType::Tarbz2 => {
                println!("Tarbz2");
            }
            FileType::Targz => {
                println!("Targz");
            }
        }
    }

    // TODO: get file type
}
