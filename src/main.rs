mod cli;
mod contents;
mod modules;

use std::path;

use crate::cli::Args;
use crate::contents::enums;

fn main() {
    let args = Args::new();

    println!("Input path: {:?}", args.input);
    println!("Output path: {:?}", args.output);

    let filename = path::Path::new(&args.input).file_stem().unwrap();
    let mut temp_output = args.output.join(filename);

    if args.compress {
        println!("Compress");
        temp_output.set_extension(enums::get_file_type_string(args.file_type.unwrap()));
        if args.output == path::Path::new(".") {
            modules::compress(args.file_type.unwrap(), &args.input, &temp_output);
        } else {
            modules::compress(args.file_type.unwrap(), &args.input, &args.output);
        }
    }
    if args.decompress {
        println!("Decompress");
        let file_type = modules::get_file_type(&args.input);
        if args.output == path::Path::new(".") {
            modules::decompress(file_type, &args.input, &temp_output);
        } else {
            modules::decompress(file_type, &args.input, &args.output);
        }
    }
}
