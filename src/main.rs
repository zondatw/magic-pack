mod cli;

use std::time::Instant;

use crate::cli::{report, Args};
use magic_pack::service::{self, CompressRequest, DecompressRequest};

fn main() {
    let args = Args::new();

    if args.compress {
        // Resolve the password first — it may prompt — so prompt time
        // isn't counted and the prompt isn't hidden behind the bar.
        let password = args.compress_password();
        let encrypted = password.is_some();

        let pb = report::start_progress("Compressing", &args.input, args.quiet);
        let start = Instant::now();
        let result = match service::compress(CompressRequest {
            file_type: args.file_type.unwrap(),
            input: args.input.clone(),
            output: args.output.clone(),
            password,
        }) {
            Ok(result) => result,
            Err(err) => {
                report::finish_progress(&pb);
                exit_with_error(err);
            }
        };
        let elapsed = start.elapsed();
        report::finish_progress(&pb);
        report::print_compress_summary(
            &args.input,
            &result.output_path,
            elapsed,
            encrypted,
            args.quiet,
        );
    }

    if args.decompress {
        let password = args.decompress_password();

        let pb = report::start_progress("Extracting", &args.input, args.quiet);
        let start = Instant::now();
        let result = match service::decompress(DecompressRequest {
            input: args.input.clone(),
            output: args.output.clone(),
            level: args.level,
            password,
        }) {
            Ok(result) => result,
            Err(err) => {
                report::finish_progress(&pb);
                exit_with_error(err);
            }
        };
        let elapsed = start.elapsed();
        report::finish_progress(&pb);
        report::print_decompress_summary(&args.input, &result.output_path, elapsed, args.quiet);
    }
}

fn exit_with_error(err: service::MagicPackError) -> ! {
    eprintln!("Error: {}", err);
    std::process::exit(1);
}
