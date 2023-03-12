mod cli;
mod contents;
mod modules;

use crate::cli::Args;
use crate::contents::enums;

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
        let file_type = modules::get_file_type(&args.input);
        modules::decompress(file_type, &args.input, &args.output);
    }
}
