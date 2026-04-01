mod cli;

use crate::cli::Args;
use magic_pack::service::{self, CompressRequest, DecompressRequest};

fn main() {
    let args = Args::new();

    println!("Input path: {:?}", args.input);
    println!("Output path: {:?}", args.output);

    if args.compress {
        println!("Compress");
        let result = match service::compress(CompressRequest {
            file_type: args.file_type.unwrap(),
            input: args.input.clone(),
            output: args.output.clone(),
        }) {
            Ok(result) => result,
            Err(err) => exit_with_error(err),
        };
        println!("Output file: {:?}", result.output_path);
    }
    if args.decompress {
        println!("Decompress");
        let result = match service::decompress(DecompressRequest {
            input: args.input.clone(),
            output: args.output.clone(),
            level: args.level,
        }) {
            Ok(result) => result,
            Err(err) => exit_with_error(err),
        };
        println!("Output file: {:?}", result.output_path);
    }
}

fn exit_with_error(err: service::MagicPackError) -> ! {
    eprintln!("Error: {}", err);
    std::process::exit(1);
}
