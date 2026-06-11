//! CLI presentation layer: animated progress bar + a designed result
//! summary. Everything here is cosmetic — it must never turn a
//! successful operation into a failure, so size/count gathering is
//! best-effort and degrades to a minimal line on any error.

use std::io::{IsTerminal, Write};
use std::path::Path;
use std::time::Duration;

use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use walkdir::WalkDir;

/// Start an animated spinner on stderr for the running operation.
/// Returns a handle the caller clears with [`finish_progress`]. When
/// `quiet`, the bar uses a hidden draw target (no output). indicatif
/// also auto-hides when stderr is not a TTY, so pipes stay clean.
pub fn start_progress(verb: &str, input: &Path, quiet: bool) -> ProgressBar {
    if quiet {
        return ProgressBar::hidden();
    }
    let pb = ProgressBar::new_spinner();
    pb.set_draw_target(ProgressDrawTarget::stderr());
    // Pretty braille spinner + message + dim elapsed. Fall back to the
    // default template if this one somehow fails to parse.
    if let Ok(style) = ProgressStyle::with_template("{spinner:.cyan} {msg} {elapsed:.dim}") {
        pb.set_style(style.tick_strings(&["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏", "✓"]));
    }
    pb.set_message(format!("{} {}…", verb, input.display()));
    pb.enable_steady_tick(Duration::from_millis(80));
    pb
}

/// Clear the spinner line. Safe to call on a hidden/finished bar.
pub fn finish_progress(pb: &ProgressBar) {
    pb.finish_and_clear();
}

/// `✓` in green when stdout is a TTY and `NO_COLOR` is unset; plain
/// otherwise.
fn success_glyph() -> String {
    if std::io::stdout().is_terminal() && std::env::var_os("NO_COLOR").is_none() {
        "\x1b[32m✓\x1b[0m".to_string()
    } else {
        "✓".to_string()
    }
}

/// Print the compress result summary to stdout. No-op when `quiet`.
pub fn print_compress_summary(
    input: &Path,
    output: &Path,
    elapsed: Duration,
    encrypted: bool,
    quiet: bool,
) {
    if quiet {
        return;
    }
    let (files, in_bytes) = walk_stats(input);
    let out_bytes = file_size(output);

    let mut line = format!(
        "{} Compressed  {} → {}",
        success_glyph(),
        input.display(),
        output.display()
    );
    line.push('\n');
    line.push_str(&format!(
        "  {} · {} → {} ({}) · {}",
        files_label(files),
        format_bytes(in_bytes),
        format_bytes(out_bytes),
        format_ratio(in_bytes, out_bytes),
        format_duration(elapsed),
    ));
    if encrypted {
        line.push_str(" · encrypted (AES-256)");
    }
    print_line(&line);
}

/// Print the decompress result summary to stdout. No-op when `quiet`.
pub fn print_decompress_summary(input: &Path, output: &Path, elapsed: Duration, quiet: bool) {
    if quiet {
        return;
    }
    let (files, out_bytes) = walk_stats(output);

    let line = format!(
        "{} Extracted  {} → {}\n  {} · {} · {}",
        success_glyph(),
        input.display(),
        output.display(),
        files_label(files),
        format_bytes(out_bytes),
        format_duration(elapsed),
    );
    print_line(&line);
}

/// Write a finished line to stdout, ignoring write errors (a broken
/// pipe must not fail a successful operation).
fn print_line(line: &str) {
    let mut out = std::io::stdout();
    let _ = writeln!(out, "{}", line);
}

fn files_label(n: u64) -> String {
    if n == 1 {
        "1 file".to_string()
    } else {
        format!("{} files", n)
    }
}

/// Best-effort (count, total_bytes) for a path. A file → (1, len). A
/// dir → walked regular-file totals. Anything unreadable/missing →
/// (0, 0). Never panics.
pub fn walk_stats(path: &Path) -> (u64, u64) {
    match std::fs::metadata(path) {
        Ok(meta) if meta.is_file() => (1, meta.len()),
        Ok(meta) if meta.is_dir() => {
            let mut files = 0u64;
            let mut bytes = 0u64;
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                if entry.file_type().is_file() {
                    files += 1;
                    if let Ok(m) = entry.metadata() {
                        bytes += m.len();
                    }
                }
            }
            (files, bytes)
        }
        _ => (0, 0),
    }
}

fn file_size(path: &Path) -> u64 {
    std::fs::metadata(path).map(|m| m.len()).unwrap_or(0)
}

/// Human byte size, 1024-base with familiar `B/KB/MB/GB/TB` labels.
/// 1 decimal place at KB and above (`0 B`, `999 B`, `1.0 KB`, `1.5 MB`).
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    if bytes < 1024 {
        return format!("{} B", bytes);
    }
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{:.1} {}", value, UNITS[unit])
}

/// Compression ratio wording. `input == 0` → "—" (no division). Output
/// smaller → "N% smaller"; larger → "N% larger"; equal → "no size
/// change".
pub fn format_ratio(input: u64, output: u64) -> String {
    if input == 0 {
        return "—".to_string();
    }
    if output == input {
        return "no size change".to_string();
    }
    if output < input {
        let pct = ((input - output) as f64 / input as f64 * 100.0).round() as i64;
        format!("{}% smaller", pct)
    } else {
        let pct = ((output - input) as f64 / input as f64 * 100.0).round() as i64;
        format!("{}% larger", pct)
    }
}

/// Elapsed time in seconds, 2 decimals. Sub-5ms → "<0.01s".
pub fn format_duration(elapsed: Duration) -> String {
    let secs = elapsed.as_secs_f64();
    if secs < 0.005 {
        "<0.01s".to_string()
    } else {
        format!("{:.2}s", secs)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_bytes_table() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(999), "999 B");
        assert_eq!(format_bytes(1023), "1023 B");
        assert_eq!(format_bytes(1024), "1.0 KB");
        assert_eq!(format_bytes(1536), "1.5 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.0 MB");
        assert_eq!(format_bytes(1024 * 1024 * 3 / 2), "1.5 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.0 GB");
    }

    #[test]
    fn format_ratio_table() {
        assert_eq!(format_ratio(0, 0), "—");
        assert_eq!(format_ratio(0, 500), "—");
        assert_eq!(format_ratio(1000, 1000), "no size change");
        assert_eq!(format_ratio(1000, 740), "26% smaller");
        assert_eq!(format_ratio(1000, 80), "92% smaller");
        assert_eq!(format_ratio(1000, 1120), "12% larger");
    }

    #[test]
    fn format_duration_table() {
        assert_eq!(format_duration(Duration::from_millis(0)), "<0.01s");
        assert_eq!(format_duration(Duration::from_millis(2)), "<0.01s");
        assert_eq!(format_duration(Duration::from_millis(300)), "0.30s");
        assert_eq!(format_duration(Duration::from_millis(1450)), "1.45s");
    }

    #[test]
    fn files_label_singular_plural() {
        assert_eq!(files_label(1), "1 file");
        assert_eq!(files_label(0), "0 files");
        assert_eq!(files_label(4), "4 files");
    }
}
