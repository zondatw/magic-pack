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

fn get_file_type(file_path: &std::string::String) -> FileType {
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

fn pack(file_type: FileType, src_path: &std::string::String, dst_path: &std::string::String) {
    match file_type {
        FileType::Zip => {
            println!("Zip");
            let output = Command::new("zip")
                .arg("-r")
                .arg(dst_path)
                .arg(src_path)
                .output()
                .expect("zip command failed");

            if !output.status.success() {
                panic!("zip command failed");
            }
        }
        FileType::Tar => {
            println!("Tar");
            let output = Command::new("tar")
                .arg("cvf")
                .arg(dst_path)
                .arg(src_path)
                .output()
                .expect("tar command failed");

            if !output.status.success() {
                panic!("tar command failed");
            }
        }
        FileType::Tarbz2 => {
            println!("Tarbz2");
            let output = Command::new("tar")
                .arg("jcvf")
                .arg(dst_path)
                .arg(src_path)
                .output()
                .expect("tar.bz2 command failed");

            if !output.status.success() {
                panic!("tar.bz2 command failed");
            }
        }
        FileType::Targz => {
            println!("Targz");
        }
    }
}

fn unpack(file_type: FileType, src_path: &std::string::String, dst_path: &std::string::String) {
    match file_type {
        FileType::Zip => {
            println!("Zip");
            let output = Command::new("unzip")
                .arg(src_path)
                .arg("-d")
                .arg(dst_path)
                .output()
                .expect("unzip command failed");

            if !output.status.success() {
                panic!("unzip command failed");
            }
        }
        FileType::Tar => {
            println!("Tar");
            let output = Command::new("tar")
                .arg("xvf")
                .arg(src_path)
                .arg("-C")
                .arg(dst_path)
                .output()
                .expect("tar command failed");

            if !output.status.success() {
                panic!("tar command failed");
            }
        }
        FileType::Tarbz2 => {
            println!("Tarbz2");
            let output = Command::new("tar")
                .arg("jxvf")
                .arg(src_path)
                .arg("-C")
                .arg(dst_path)
                .output()
                .expect("tar.bz2 command failed");

            if !output.status.success() {
                panic!("tar.bz2 command failed");
            }
        }
        FileType::Targz => {
            println!("Targz");
        }
    }
}

fn main() {
    let args = Args::parse();

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
