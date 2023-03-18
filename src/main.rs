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
        let mut decompress_output = args.output;
        let mut decompress_input = args.input;
        let file_type = modules::get_file_type(&decompress_input.to_owned().into_os_string().into_string().unwrap());
        if decompress_output == path::Path::new(".") {
            decompress_output = temp_output;
        }
        println!("Decompress output: {:?}", decompress_output.to_owned());
        modules::decompress(file_type, &decompress_input, &decompress_output);
    }
}
