mod cli;

use std::time::Instant;

use crate::cli::{report, Args};
use magic_pack::service::{self, CompressRequest, DecompressRequest};

fn main() {
    let args = Args::new();

    if args.compress {
        let file_type = args.file_type.unwrap();
        // Resolve the password first — it may prompt — so prompt time
        // isn't counted and the prompt isn't hidden behind the bar.
        let password = args.compress_password();
        let encrypted = password.is_some();

        let total = report::walk_stats(&args.input).1;
        let determinate =
            file_type.reports_compress_progress() && total >= report::PROGRESS_BAR_MIN_BYTES;

        let start = Instant::now();
        let result = report::with_progress(
            "Compressing",
            &args.input,
            total,
            determinate,
            args.quiet,
            |counter| {
                service::compress_with_progress(
                    CompressRequest {
                        file_type,
                        input: args.input.clone(),
                        output: args.output.clone(),
                        password,
                    },
                    counter,
                )
            },
        );
        let elapsed = start.elapsed();
        let result = match result {
            Ok(result) => result,
            Err(err) => exit_with_error(err),
        };
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

        let total = std::fs::metadata(&args.input).map(|m| m.len()).unwrap_or(0);
        // Decompress auto-detects the format, so peek at the type to
        // decide bar vs spinner (zip/7z/upx report no decompress progress).
        // Guard on is_file() — detect_file_type opens the file and would
        // panic on a missing path; let the service surface the clean error.
        let determinate = args.input.is_file()
            && service::detect_file_type(&args.input)
                .map(|t| t.reports_decompress_progress())
                .unwrap_or(false)
            && total >= report::PROGRESS_BAR_MIN_BYTES;

        let start = Instant::now();
        let result = report::with_progress(
            "Extracting",
            &args.input,
            total,
            determinate,
            args.quiet,
            |counter| {
                service::decompress_with_progress(
                    DecompressRequest {
                        input: args.input.clone(),
                        output: args.output.clone(),
                        level: args.level,
                        password,
                    },
                    counter,
                )
            },
        );
        let elapsed = start.elapsed();
        let result = match result {
            Ok(result) => result,
            Err(err) => exit_with_error(err),
        };
        report::print_decompress_summary(&args.input, &result.output_path, elapsed, args.quiet);
    }

    if args.list {
        let result = match service::list(&args.input) {
            Ok(result) => result,
            Err(err) => exit_with_error(err),
        };
        let file_type = magic_pack::contents::enums::get_file_type_string(result.file_type);
        report::print_listing(&args.input, file_type, &result.entries, args.quiet);
    }
}

fn exit_with_error(err: service::MagicPackError) -> ! {
    eprintln!("Error: {}", err);
    std::process::exit(1);
}
