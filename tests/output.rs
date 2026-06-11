//! Integration tests for the CLI's designed output (progress bar +
//! result summary). Drives the real `magic-pack` binary and asserts on
//! captured stdout/stderr. The animated bar itself (AC-10) can only be
//! seen in a real TTY; a piped test child is never a TTY, so here we
//! assert the *absence* of progress artifacts when piped (AC-11) and
//! the shape of the summary.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_magic-pack")
}

fn tempdir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "magic-pack-output-{}-{}",
        label,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn seed_tree(root: &Path) {
    fs::create_dir_all(root.join("test_dir")).unwrap();
    fs::write(root.join("test_dir/a.txt"), b"hello world\n").unwrap();
    fs::write(root.join("test_dir/b.txt"), vec![b'x'; 4000]).unwrap();
}

#[test]
fn compress_prints_designed_summary() {
    let dir = tempdir("compress");
    seed_tree(&dir);
    let out = Command::new(bin())
        .args(["-c", "-f", "zip", "-o"])
        .arg(dir.join("t.zip"))
        .arg(dir.join("test_dir"))
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("✓ Compressed"), "stdout: {}", stdout);
    assert!(stdout.contains('→'), "stdout: {}", stdout);
    assert!(
        stdout.contains("files") || stdout.contains("file"),
        "stdout: {}",
        stdout
    );
    assert!(
        stdout.contains("KB") || stdout.contains(" B"),
        "stdout: {}",
        stdout
    );
    assert!(stdout.contains('s'), "stdout: {}", stdout);
    // Old debug lines must be gone.
    assert!(!stdout.contains("Input path:"), "stdout: {}", stdout);
    assert!(!stdout.contains("Output file:"), "stdout: {}", stdout);
}

#[test]
fn decompress_prints_designed_summary() {
    let dir = tempdir("decompress");
    seed_tree(&dir);
    Command::new(bin())
        .args(["-c", "-f", "zip", "-o"])
        .arg(dir.join("t.zip"))
        .arg(dir.join("test_dir"))
        .output()
        .unwrap();

    let out = Command::new(bin())
        .args(["-d", "-o"])
        .arg(dir.join("out"))
        .arg(dir.join("t.zip"))
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("✓ Extracted"), "stdout: {}", stdout);
    assert!(!stdout.contains("Decompress\n"), "stdout: {}", stdout);
}

#[test]
fn quiet_suppresses_all_success_output() {
    let dir = tempdir("quiet");
    seed_tree(&dir);
    let out = Command::new(bin())
        .args(["-c", "-f", "zip", "-q", "-o"])
        .arg(dir.join("t.zip"))
        .arg(dir.join("test_dir"))
        .output()
        .unwrap();

    assert!(out.status.success());
    assert!(
        out.stdout.is_empty(),
        "stdout should be empty: {:?}",
        out.stdout
    );
    // Artifact still produced.
    assert!(dir.join("t.zip").exists());
}

#[test]
fn piped_output_has_no_ansi_or_progress_artifacts() {
    let dir = tempdir("ansi");
    seed_tree(&dir);
    let out = Command::new(bin())
        .args(["-c", "-f", "zip", "-o"])
        .arg(dir.join("t.zip"))
        .arg(dir.join("test_dir"))
        .output()
        .unwrap();

    // No ESC (0x1b) anywhere — non-TTY child means no color on stdout
    // and no spinner frames on stderr.
    assert!(!out.stdout.contains(&0x1b), "stdout has ANSI escape");
    assert!(
        !out.stderr.contains(&0x1b),
        "stderr has ANSI escape (progress leaked)"
    );
    // stderr carries no summary either.
    assert!(
        out.stderr.is_empty(),
        "stderr should be empty: {:?}",
        out.stderr
    );
}

#[test]
fn error_still_exits_nonzero_with_stderr() {
    let dir = tempdir("err");
    let out = Command::new(bin())
        .args(["-d", "-o"])
        .arg(dir.join("out"))
        .arg(dir.join("does-not-exist.zip"))
        .output()
        .unwrap();

    assert!(!out.status.success());
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(stderr.contains("Error:"), "stderr: {}", stderr);
}

#[cfg(feature = "encryption")]
#[test]
fn encrypted_compress_shows_aes_note() {
    let dir = tempdir("enc");
    seed_tree(&dir);
    let out = Command::new(bin())
        .args(["-c", "-f", "seven-z", "-p", "pw123", "-o"])
        .arg(dir.join("enc.7z"))
        .arg(dir.join("test_dir"))
        .output()
        .unwrap();

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("encrypted (AES-256)"), "stdout: {}", stdout);
}
