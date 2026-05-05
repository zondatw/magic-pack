//! Test-fixture generator. Best-effort: never fails the build.
//!
//! Produces three artefacts under `OUT_DIR`:
//!
//! - `upx_unpacked_host.bin`  — a tiny host-target hello-world binary.
//! - `upx_packed_host.bin`    — the same binary after `upx` packing.
//! - `upx_missing` (sentinel) — written instead when `upx` is not on
//!   PATH or the host-target build / pack step fails. Tests check for
//!   this sentinel and skip the roundtrip integration assertions.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=PATH");

    let out_dir = match env::var_os("OUT_DIR") {
        Some(v) => PathBuf::from(v),
        None => return,
    };

    let unpacked = out_dir.join("upx_unpacked_host.bin");
    let packed = out_dir.join("upx_packed_host.bin");
    let sentinel = out_dir.join("upx_missing");

    // Clean any prior artefacts so we don't surface stale fixtures.
    let _ = fs::remove_file(&unpacked);
    let _ = fs::remove_file(&packed);
    let _ = fs::remove_file(&sentinel);

    if let Err(reason) = build_fixture(&out_dir, &unpacked, &packed) {
        let _ = fs::write(&sentinel, reason.as_bytes());
    }
}

fn build_fixture(out_dir: &Path, unpacked: &Path, packed: &Path) -> Result<(), String> {
    if which("upx").is_none() {
        return Err(String::from("upx not on PATH"));
    }

    // Source for the tiny host binary: a no-std-ish hello world is
    // overkill, a normal hello world is fine. UPX rejects very small
    // binaries (< ~10 KB on most platforms), so rely on rustc's default
    // output which is usually large enough.
    let src_path = out_dir.join("upx_fixture_src.rs");
    let src_body = r#"fn main() { println!("upx-fixture"); }"#;
    fs::write(&src_path, src_body).map_err(|e| format!("write src: {}", e))?;

    let rustc = env::var_os("RUSTC").unwrap_or_else(|| "rustc".into());
    let status = Command::new(&rustc)
        .arg(&src_path)
        .arg("-O")
        .arg("-o")
        .arg(unpacked)
        .status()
        .map_err(|e| format!("invoke rustc: {}", e))?;
    if !status.success() {
        return Err(format!("rustc exited with {}", status));
    }

    // Copy unpacked → temp file → run upx in-place pack into `packed`.
    fs::copy(unpacked, packed).map_err(|e| format!("copy unpacked: {}", e))?;
    let status = Command::new("upx")
        .arg("-q")
        .arg("--")
        .arg(packed)
        .status()
        .map_err(|e| format!("invoke upx: {}", e))?;
    if !status.success() {
        // upx sometimes refuses tiny binaries — clean up partial output
        // and fall back to sentinel.
        let _ = fs::remove_file(packed);
        return Err(format!("upx exited with {}", status));
    }
    Ok(())
}

fn which(prog: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;
    for dir in env::split_paths(&path) {
        let candidate = dir.join(prog);
        if candidate.is_file() {
            return Some(candidate);
        }
        // Windows .exe fallback
        let candidate_exe = dir.join(format!("{}.exe", prog));
        if candidate_exe.is_file() {
            return Some(candidate_exe);
        }
    }
    None
}
