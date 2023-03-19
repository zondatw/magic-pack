mod cli;
mod contents;
mod modules;

use std::fs;
use std::io::ErrorKind;
use std::path;

use crate::cli::Args;
use crate::contents::enums;

fn compress(
    file_type: &enums::FileType,
    src_path: &std::path::PathBuf,
    dst_path: &std::path::PathBuf,
) {
    println!("Compress");

    let filename = path::Path::new(&src_path).file_stem().unwrap();
    let mut temp_output = dst_path.join(filename);

    temp_output.set_extension(enums::get_file_type_string(file_type.to_owned()));
    if dst_path == path::Path::new(".") {
        modules::compress(file_type.to_owned(), &src_path, &temp_output);
    } else {
        modules::compress(file_type.to_owned(), &src_path, &dst_path);
    }
}

fn decompress(src_path: &std::path::PathBuf, dst_path: &std::path::PathBuf) {
    let filename = path::Path::new(src_path).file_stem().unwrap();
    let temp_output = dst_path.join(filename);
    println!("Decompress");
    if dst_path != path::Path::new(".") {
        fs::create_dir_all(&dst_path).expect("Create dir failed");
    }
    let mut decompress_output = temp_output;
    let mut decompress_input = src_path.to_owned();
    let _filename = path::Path::new(&decompress_output).file_name().unwrap();
    let mg_filename = format!("{}{}", "mg_", _filename.to_str().unwrap());
    decompress_output.set_file_name(mg_filename);

    for index in 0..5 {
        let file_type = match modules::get_file_type(
            &decompress_input
                .to_owned()
                .into_os_string()
                .into_string()
                .unwrap(),
        ) {
            Ok(file_type) => file_type,
            Err(e) if e.kind() == ErrorKind::Unsupported && index != 0 => {
                let final_filename = decompress_input
                    .file_name()
                    .unwrap()
                    .to_os_string()
                    .into_string()
                    .unwrap()
                    .replace("mg_", "");
                let mut decompress_final_output = path::PathBuf::from(&decompress_input);
                decompress_final_output.set_file_name(final_filename);
                fs::rename(decompress_input, decompress_final_output).expect("Rename failed");
                break;
            }
            Err(e) => panic!("{}", e),
        };
        println!("Decompress output: {:?}", decompress_output.to_owned());
        modules::decompress(file_type, &decompress_input, &decompress_output);
        decompress_input = decompress_output;
        let temp_filename = path::Path::new(&decompress_input).file_stem().unwrap();
        decompress_output = path::PathBuf::from(&decompress_input);
        decompress_output.set_file_name(temp_filename);
    }
}

fn main() {
    let args = Args::new();

    println!("Input path: {:?}", args.input);
    println!("Output path: {:?}", args.output);

    if args.compress {
        compress(&args.file_type.unwrap(), &args.input, &args.output);
    }
    if args.decompress {
        decompress(&args.input, &args.output);
    }
}
