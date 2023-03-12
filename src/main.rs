use std::fs;
use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::{Read, Seek, Write};
use std::path::PathBuf;
use std::process::Command;

use bzip2;
use bzip2::read::BzDecoder;
use bzip2::write::BzEncoder;
use clap::Parser;
use flate2;
use flate2::read::GzDecoder;
use flate2::write::GzEncoder;
use tar;
use tar::Archive;
use walkdir::{DirEntry, WalkDir};
use zip;
use zip::write::FileOptions;

mod cli;
mod enums;

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

fn zip_dir<T>(
    it: &mut dyn Iterator<Item = DirEntry>,
    writer: T,
    method: zip::CompressionMethod,
) -> zip::result::ZipResult<()>
where
    T: Write + Seek,
{
    let mut zip = zip::ZipWriter::new(writer);
    let options = FileOptions::default()
        .compression_method(method)
        .unix_permissions(0o755);

    let mut buffer = Vec::new();
    for entry in it {
        let path = entry.path();

        if path.is_file() {
            zip.start_file(
                path.to_owned().into_os_string().into_string().unwrap(),
                options,
            )
            .expect("zip start file from path failed");
            let mut f = File::open(path).expect("zip open compressing-file failed");

            f.read_to_end(&mut buffer)
                .expect("zip read compressing-file failed");
            zip.write_all(&buffer).expect("zip compress file failed");
            buffer.clear();
        } else if !path.as_os_str().is_empty() {
            zip.add_directory(
                path.to_owned().into_os_string().into_string().unwrap(),
                options,
            )
            .expect("zip add dir from path failed");
        }
    }
    zip.finish().expect("zip compress failed");
    Result::Ok(())
}

fn tar_dir<T>(it: &mut dyn Iterator<Item = DirEntry>, tar_file: T)
where
    T: Write + Seek,
{
    let mut tar_builder = tar::Builder::new(tar_file);
    for entry in it {
        let path = entry.path();
        tar_builder.append_path(path).expect("tar append failed");
    }
}

fn tar_bz2_dir<T>(it: &mut dyn Iterator<Item = DirEntry>, tar_bz2_file: T)
where
    T: Write + Seek,
{
    let enc = BzEncoder::new(tar_bz2_file, bzip2::Compression::default());
    let mut tar_bz2_builder = tar::Builder::new(enc);
    for entry in it {
        let path = entry.path();
        tar_bz2_builder
            .append_path(path)
            .expect("tar.bz2 append failed");
    }
}

fn tar_gz_dir<T>(it: &mut dyn Iterator<Item = DirEntry>, tar_gz_file: T)
where
    T: Write + Seek,
{
    let enc = GzEncoder::new(tar_gz_file, flate2::Compression::default());
    let mut tar_gz_builder = tar::Builder::new(enc);
    for entry in it {
        let path = entry.path();
        tar_gz_builder
            .append_path(path)
            .expect("tar.gz append failed");
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
            let zip_file = File::create(dst_path).expect("zip create failed");
            let walkdir = WalkDir::new(src_path);
            let it = walkdir.into_iter();
            zip_dir(
                &mut it.filter_map(|e| e.ok()),
                zip_file,
                zip::CompressionMethod::Stored,
            )
            .expect("zip compress dir failed");
        }
        enums::FileType::Tar => {
            println!("Tar");
            let tar_file = File::create(dst_path).expect("tar create failed");
            let walkdir = WalkDir::new(src_path);
            let it = walkdir.into_iter();
            tar_dir(&mut it.filter_map(|e| e.ok()), tar_file);
        }
        enums::FileType::Tarbz2 => {
            println!("Tarbz2");
            let tar_bz2_file = File::create(dst_path).expect("tar.bz2 create failed");
            let walkdir = WalkDir::new(src_path);
            let it = walkdir.into_iter();
            tar_bz2_dir(&mut it.filter_map(|e| e.ok()), tar_bz2_file);
        }
        enums::FileType::Targz => {
            println!("Targz");
            let tar_gz_file = File::create(dst_path).expect("tar.gz create failed");
            let walkdir = WalkDir::new(src_path);
            let it = walkdir.into_iter();
            tar_gz_dir(&mut it.filter_map(|e| e.ok()), tar_gz_file);
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
            let zip_file = File::open(src_path).expect("zip open failed");
            let mut zip_archive =
                zip::ZipArchive::new(BufReader::new(zip_file)).expect("zip open to archive failed");

            for i in 0..zip_archive.len() {
                let mut file = zip_archive.by_index(i).expect("zip index not exist");
                let outpath = &PathBuf::from(dst_path).join(file.mangled_name());

                if (&*file.name()).ends_with('/') {
                    fs::create_dir_all(&outpath).expect("zip create dir all failed");
                } else {
                    if let Some(p) = outpath.parent() {
                        if !p.exists() {
                            fs::create_dir_all(&p).expect("zip create dir all failed");
                        }
                    }
                    let mut outfile = fs::File::create(&outpath).expect("zip create file failed");
                    io::copy(&mut file, &mut outfile).expect("zip file copy failed");
                }
            }
        }
        enums::FileType::Tar => {
            println!("Tar");
            let tar_file = File::open(src_path).expect("tar open failed");
            let mut archive = Archive::new(tar_file);
            archive.unpack(dst_path).expect("tar unpack failed");
        }
        enums::FileType::Tarbz2 => {
            println!("Tarbz2");
            let tar_bz2_file = File::open(src_path).expect("tar.bz2 open failed");
            let dec = BzDecoder::new(tar_bz2_file);
            let mut archive = Archive::new(dec);
            archive.unpack(dst_path).expect("tar.bz2 unpack failed");
        }
        enums::FileType::Targz => {
            println!("Targz");
            let tar_gz_file = File::open(src_path).expect("tar.gz open failed");
            let dec = GzDecoder::new(tar_gz_file);
            let mut archive = Archive::new(dec);
            archive.unpack(dst_path).expect("tar.gz unpack failed");
        }
    }
}

fn main() {
    let args = cli::Args::parse();

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
