//! UPX shell-out + detection.
//!
//! UPX is the only format magic-pack supports that requires an external
//! binary on PATH; we never link or bundle it. Both compress and
//! decompress route through `Command::new("upx")`. Detection is a
//! byte-scan that combines a structural exec-header check with the
//! UPX `UPX!` marker.

use std::ffi::{OsStr, OsString};
use std::io::ErrorKind;
use std::path::Path;
use std::process::Command;

const UPX_MARKER: &[u8] = b"UPX!";
/// Section names UPX writes into PE binaries; they're a corroborating
/// signal but not load-bearing — `UPX!` plus a valid exec header is the
/// primary check.
#[allow(dead_code)]
const UPX_PE_SECTION_NAMES: [&[u8]; 3] = [b"UPX0", b"UPX1", b"UPX2"];
/// 16 KB is enough to cover the typical UPX stub area without paying
/// for a full file read on every detect call.
const SCAN_LIMIT: usize = 16 * 1024;
/// At least one `UPX!` marker plus a structurally valid PE / ELF /
/// Mach-O header is enough to call a binary UPX-packed. Real packed
/// binaries on every platform we checked (Linux ELF, macOS Mach-O,
/// Windows PE) leave only one `UPX!` magic near the header — a higher
/// threshold rejects real-world packed inputs. The structural exec
/// header check (PE `e_lfanew → PE\0\0`, ELF magic + EI_CLASS /
/// EI_DATA, six specific Mach-O magics) is the load-bearing
/// gatekeeper against false positives.
const MIN_MARKER_COUNT: usize = 1;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Action {
    Compress,
    Decompress,
}

/// True when `buf` begins with a recognised executable header AND
/// contains at least [`MIN_MARKER_COUNT`] UPX markers within
/// [`SCAN_LIMIT`] bytes.
pub fn detect(buf: &[u8]) -> bool {
    if !has_executable_header(buf) {
        return false;
    }
    let scan_window = &buf[..buf.len().min(SCAN_LIMIT)];
    count_marker_occurrences(scan_window, UPX_MARKER) >= MIN_MARKER_COUNT
}

fn has_executable_header(buf: &[u8]) -> bool {
    is_pe(buf) || is_elf(buf) || is_macho(buf)
}

fn is_pe(buf: &[u8]) -> bool {
    if buf.len() < 0x40 || &buf[..2] != b"MZ" {
        return false;
    }
    // e_lfanew at offset 0x3c → must point inside the buffer at PE\0\0
    let e_lfanew = u32::from_le_bytes([buf[0x3c], buf[0x3d], buf[0x3e], buf[0x3f]]) as usize;
    e_lfanew + 4 <= buf.len() && &buf[e_lfanew..e_lfanew + 4] == b"PE\0\0"
}

fn is_elf(buf: &[u8]) -> bool {
    if buf.len() < 16 || &buf[..4] != b"\x7fELF" {
        return false;
    }
    // EI_CLASS at offset 4 (1 = 32-bit, 2 = 64-bit) and EI_DATA at
    // offset 5 (1 = LE, 2 = BE) must hold sane values; rules out
    // arbitrary `\x7fELF` prefixes in non-binary data.
    matches!(buf[4], 1 | 2) && matches!(buf[5], 1 | 2)
}

fn is_macho(buf: &[u8]) -> bool {
    if buf.len() < 4 {
        return false;
    }
    let magic = [buf[0], buf[1], buf[2], buf[3]];
    matches!(
        magic,
        // Thin LE / BE 32-bit and 64-bit
        [0xCE, 0xFA, 0xED, 0xFE]
            | [0xCF, 0xFA, 0xED, 0xFE]
            | [0xFE, 0xED, 0xFA, 0xCE]
            | [0xFE, 0xED, 0xFA, 0xCF]
            // FAT (universal) LE / BE
            | [0xCA, 0xFE, 0xBA, 0xBE]
            | [0xBE, 0xBA, 0xFE, 0xCA]
    )
}

fn count_marker_occurrences(haystack: &[u8], needle: &[u8]) -> usize {
    if needle.is_empty() || haystack.len() < needle.len() {
        return 0;
    }
    haystack
        .windows(needle.len())
        .filter(|w| *w == needle)
        .count()
}

/// Build the argv that gets handed to `upx`. Always emits a `--`
/// separator before path operands so paths starting with `-` cannot be
/// reinterpreted as flags.
pub fn build_args<S: AsRef<OsStr>, D: AsRef<OsStr>>(
    action: Action,
    src: S,
    dst: D,
) -> Vec<OsString> {
    let mut args: Vec<OsString> = Vec::with_capacity(6);
    if action == Action::Decompress {
        args.push("-d".into());
    }
    args.push("-q".into());
    args.push("-o".into());
    args.push(dst.as_ref().to_os_string());
    args.push("--".into());
    args.push(src.as_ref().to_os_string());
    args
}

pub fn compress(src_path: &Path, dst_path: &Path) {
    run_upx(Action::Compress, src_path, dst_path);
}

pub fn decompress(src_path: &Path, dst_path: &Path) {
    run_upx(Action::Decompress, src_path, dst_path);
}

fn run_upx(action: Action, src: &Path, dst: &Path) {
    let args = build_args(action, src, dst);
    let output = match Command::new("upx").args(&args).output() {
        Ok(out) => out,
        Err(err) if err.kind() == ErrorKind::NotFound => {
            panic!("{}", missing_binary_message());
        }
        Err(err) => panic!("upx failed to start: {}", err),
    };

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let label = match action {
            Action::Compress => "compress",
            Action::Decompress => "decompress",
        };
        panic!(
            "upx {} failed (exit {}): {}",
            label,
            output
                .status
                .code()
                .map(|c| c.to_string())
                .unwrap_or_else(|| String::from("?")),
            stderr.trim()
        );
    }
}

fn missing_binary_message() -> String {
    String::from(
        "upx binary not found on PATH. Install via one of:\n  \
         - Debian / Ubuntu: sudo apt install upx-ucl\n  \
         - macOS (Homebrew): brew install upx\n  \
         - Arch Linux: sudo pacman -S upx\n  \
         - Windows (Scoop): scoop install upx",
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_pe_buffer(marker_count: usize) -> Vec<u8> {
        let mut buf = vec![0u8; 4096];
        buf[0..2].copy_from_slice(b"MZ");
        // e_lfanew = 0x80
        buf[0x3c..0x40].copy_from_slice(&0x80u32.to_le_bytes());
        buf[0x80..0x84].copy_from_slice(b"PE\0\0");
        for i in 0..marker_count {
            let offset = 0x100 + i * 8;
            buf[offset..offset + 4].copy_from_slice(UPX_MARKER);
        }
        buf
    }

    fn build_elf_buffer(marker_count: usize) -> Vec<u8> {
        let mut buf = vec![0u8; 4096];
        buf[0..4].copy_from_slice(b"\x7fELF");
        buf[4] = 2; // 64-bit
        buf[5] = 1; // little-endian
        for i in 0..marker_count {
            let offset = 0x200 + i * 8;
            buf[offset..offset + 4].copy_from_slice(UPX_MARKER);
        }
        buf
    }

    fn build_macho_buffer(magic: [u8; 4], marker_count: usize) -> Vec<u8> {
        let mut buf = vec![0u8; 4096];
        buf[0..4].copy_from_slice(&magic);
        for i in 0..marker_count {
            let offset = 0x300 + i * 8;
            buf[offset..offset + 4].copy_from_slice(UPX_MARKER);
        }
        buf
    }

    #[test]
    fn detect_pe_with_two_markers_returns_true() {
        assert!(detect(&build_pe_buffer(2)));
    }

    #[test]
    fn detect_elf_with_two_markers_returns_true() {
        assert!(detect(&build_elf_buffer(2)));
    }

    #[test]
    fn detect_macho_thin_64_le_with_two_markers_returns_true() {
        assert!(detect(&build_macho_buffer([0xCF, 0xFA, 0xED, 0xFE], 2)));
    }

    #[test]
    fn detect_macho_fat_returns_true() {
        assert!(detect(&build_macho_buffer([0xCA, 0xFE, 0xBA, 0xBE], 3)));
    }

    #[test]
    fn detect_macho_with_single_marker_returns_true() {
        // Real UPX-packed Mach-O binaries leave only one `UPX!` marker
        // near the header. The Mach-O structural magic is itself a
        // strong-enough gatekeeper that a single marker is reliable.
        assert!(detect(&build_macho_buffer([0xCF, 0xFA, 0xED, 0xFE], 1)));
    }

    #[test]
    fn detect_macho_with_zero_markers_returns_false() {
        assert!(!detect(&build_macho_buffer([0xCF, 0xFA, 0xED, 0xFE], 0)));
    }

    #[test]
    fn detect_text_with_marker_returns_false() {
        let mut buf = b"plaintext UPX! data UPX! more".to_vec();
        buf.resize(2048, 0);
        assert!(!detect(&buf));
    }

    #[test]
    fn detect_plain_pe_without_markers_returns_false() {
        assert!(!detect(&build_pe_buffer(0)));
    }

    #[test]
    fn detect_pe_with_single_marker_returns_true() {
        // Real UPX-packed PE / ELF binaries also leave only one `UPX!`
        // marker near the header (verified against Linux ELF on CI and
        // macOS Mach-O via dogfood). The structural PE header check is
        // the real gatekeeper, so a single marker is sufficient.
        assert!(detect(&build_pe_buffer(1)));
    }

    #[test]
    fn detect_truncated_buffer_returns_false_no_panic() {
        assert!(!detect(b"MZ"));
        assert!(!detect(b""));
    }

    #[test]
    fn detect_pe_with_invalid_e_lfanew_returns_false() {
        // MZ present but e_lfanew points outside the buffer.
        let mut buf = vec![0u8; 64];
        buf[0..2].copy_from_slice(b"MZ");
        buf[0x3c..0x40].copy_from_slice(&0xFFFF_FFFFu32.to_le_bytes());
        // even with markers it should fail header check
        assert!(!detect(&buf));
    }

    #[test]
    fn detect_marker_count_caps_at_scan_limit() {
        // Place markers BEYOND SCAN_LIMIT — should not be counted.
        let mut buf = vec![0u8; SCAN_LIMIT + 256];
        buf[0..2].copy_from_slice(b"MZ");
        buf[0x3c..0x40].copy_from_slice(&0x80u32.to_le_bytes());
        buf[0x80..0x84].copy_from_slice(b"PE\0\0");
        // markers placed at offsets after SCAN_LIMIT
        buf[SCAN_LIMIT..SCAN_LIMIT + 4].copy_from_slice(UPX_MARKER);
        buf[SCAN_LIMIT + 8..SCAN_LIMIT + 12].copy_from_slice(UPX_MARKER);
        assert!(!detect(&buf));
    }

    #[test]
    fn build_args_decompress_emits_dash_dash_separator() {
        let args = build_args(Action::Decompress, "/tmp/in", "/tmp/out");
        let strs: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(strs, vec!["-d", "-q", "-o", "/tmp/out", "--", "/tmp/in"]);
    }

    #[test]
    fn build_args_compress_emits_dash_dash_separator() {
        let args = build_args(Action::Compress, "/tmp/in", "/tmp/out");
        let strs: Vec<String> = args
            .iter()
            .map(|a| a.to_string_lossy().into_owned())
            .collect();
        assert_eq!(strs, vec!["-q", "-o", "/tmp/out", "--", "/tmp/in"]);
    }

    #[test]
    fn build_args_path_with_leading_dash_passes_after_separator() {
        let args = build_args(Action::Decompress, "-malicious.exe", "/tmp/out");
        let separator_idx = args
            .iter()
            .position(|a| a == "--")
            .expect("-- separator must be present");
        let src_idx = args
            .iter()
            .position(|a| a == "-malicious.exe")
            .expect("input path must be present");
        assert!(
            src_idx > separator_idx,
            "input path must be passed after the -- separator"
        );
    }

    #[test]
    fn missing_binary_message_lists_install_hints() {
        let msg = missing_binary_message();
        assert!(msg.contains("apt install upx-ucl"));
        assert!(msg.contains("brew install upx"));
        assert!(msg.contains("pacman -S upx"));
        assert!(msg.contains("scoop install upx"));
    }
}
