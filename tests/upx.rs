//! Integration tests for the UPX support path. Exercises the public
//! `service` API and the magic-pack CLI binary. Detector unit tests
//! (handcrafted byte buffers + path-injection guard) live next to the
//! implementation in `src/modules/compression/upx.rs`; filename UX
//! tests live in `src/service.rs`.
//!
//! Roundtrip tests skip themselves when the build-time fixture
//! generator could not find `upx` on PATH (sentinel file written by
//! `build.rs`).

use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

use magic_pack::contents::enums::FileType;
use magic_pack::service::{self, CompressRequest, DecompressRequest, MagicPackError};

fn fixture_dir() -> PathBuf {
    PathBuf::from(env!("OUT_DIR"))
}

fn fixture_available() -> bool {
    !fixture_dir().join("upx_missing").exists()
}

fn skip_if_no_fixture(test_name: &str) -> bool {
    if fixture_available() {
        return false;
    }
    eprintln!(
        "[skip] {}: build.rs sentinel present (upx not on PATH at build time)",
        test_name
    );
    true
}

fn tempdir(label: &str) -> PathBuf {
    let dir = env::temp_dir().join(format!("magic-pack-upx-{}-{}", label, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("create tempdir");
    dir
}

#[test]
fn detect_real_packed_fixture_returns_upx() {
    if skip_if_no_fixture("detect_real_packed_fixture_returns_upx") {
        return;
    }
    let packed = fixture_dir().join("upx_packed_host.bin");
    let detected = service::detect_file_type(&packed).expect("detect should succeed");
    assert_eq!(detected, FileType::Upx);
}

#[test]
fn detect_unpacked_fixture_returns_unsupported() {
    if skip_if_no_fixture("detect_unpacked_fixture_returns_unsupported") {
        return;
    }
    let unpacked = fixture_dir().join("upx_unpacked_host.bin");
    let result = service::detect_file_type(&unpacked);
    match result {
        Err(MagicPackError::UnsupportedFileType) => (),
        other => panic!("expected UnsupportedFileType, got {:?}", other),
    }
}

#[test]
fn decompress_packed_fixture_byte_equals_original() {
    if skip_if_no_fixture("decompress_packed_fixture_byte_equals_original") {
        return;
    }
    let dir = tempdir("roundtrip");
    let staged = dir.join("hello.upx.bin");
    fs::copy(fixture_dir().join("upx_packed_host.bin"), &staged).unwrap();

    let result = service::decompress(DecompressRequest {
        input: staged,
        output: dir.clone(),
        level: 5,
        password: None,
    })
    .expect("decompress should succeed");

    // `.upx.bin` infix stripped → `hello.bin`
    assert_eq!(result.output_path, dir.join("hello.bin"));
    let unpacked = fs::read(&result.output_path).unwrap();
    let original = fs::read(fixture_dir().join("upx_unpacked_host.bin")).unwrap();
    assert_eq!(
        unpacked, original,
        "decompressed bytes must match original binary"
    );
}

#[test]
fn decompress_filename_with_no_upx_infix_gets_unpacked_suffix() {
    if skip_if_no_fixture("decompress_filename_with_no_upx_infix_gets_unpacked_suffix") {
        return;
    }
    let dir = tempdir("no-infix");
    let staged = dir.join("hello.bin");
    fs::copy(fixture_dir().join("upx_packed_host.bin"), &staged).unwrap();

    let result = service::decompress(DecompressRequest {
        input: staged,
        output: dir.clone(),
        level: 5,
        password: None,
    })
    .expect("decompress should succeed");

    assert_eq!(result.output_path, dir.join("hello.unpacked.bin"));
}

#[test]
fn decompress_level_loop_stops_after_one_packer_iteration() {
    // AC-8: dedicated executable-packer fast path bypasses the
    // level loop entirely; high level value must not over-decompress.
    if skip_if_no_fixture("decompress_level_loop_stops_after_one_packer_iteration") {
        return;
    }
    let dir = tempdir("level-loop");
    let staged = dir.join("hello.upx.bin");
    fs::copy(fixture_dir().join("upx_packed_host.bin"), &staged).unwrap();

    let result = service::decompress(DecompressRequest {
        input: staged,
        output: dir.clone(),
        level: 100,
        password: None,
    })
    .expect("decompress should succeed");

    let bytes = fs::read(&result.output_path).unwrap();
    let original = fs::read(fixture_dir().join("upx_unpacked_host.bin")).unwrap();
    assert_eq!(bytes, original);
}

#[test]
fn service_compress_then_decompress_roundtrips_byte_equal() {
    // AC-5: explicitly exercises `service::compress` (the build-time
    // fixture comes from build.rs's direct upx invocation, not our
    // compress path).
    if skip_if_no_fixture("service_compress_then_decompress_roundtrips_byte_equal") {
        return;
    }
    let dir = tempdir("compress-roundtrip");
    let unpacked_src = fixture_dir().join("upx_unpacked_host.bin");
    let staged = dir.join("hello.bin");
    fs::copy(&unpacked_src, &staged).unwrap();

    // `service::compress` treats `output != "."` as the explicit
    // archive file path (per the SKILL.md gotcha), not a directory.
    let pack_target = dir.join("hello.upx.bin");
    let pack_result = service::compress(CompressRequest {
        file_type: FileType::Upx,
        input: staged.clone(),
        output: pack_target.clone(),
        password: None,
    })
    .expect("compress should succeed");

    assert_eq!(pack_result.output_path, pack_target);

    // Stage the unpack into a fresh sub-dir so it doesn't collide with
    // the `hello.bin` we used as compress input.
    let unpack_dir = dir.join("unpack-out");
    let unpack_result = service::decompress(DecompressRequest {
        input: pack_result.output_path,
        output: unpack_dir.clone(),
        level: 5,
        password: None,
    })
    .expect("decompress should succeed");

    // .upx.bin → .bin
    assert_eq!(unpack_result.output_path, unpack_dir.join("hello.bin"));
    let original = fs::read(&unpacked_src).unwrap();
    let unpacked = fs::read(&unpack_result.output_path).unwrap();
    assert_eq!(unpacked, original);
}

#[test]
fn cli_decompress_smoke() {
    // AC-3: shell out to the magic-pack binary and verify the unpack.
    if skip_if_no_fixture("cli_decompress_smoke") {
        return;
    }
    let dir = tempdir("cli-smoke");
    let staged = dir.join("hello.upx.bin");
    fs::copy(fixture_dir().join("upx_packed_host.bin"), &staged).unwrap();

    let bin = env!("CARGO_BIN_EXE_magic-pack");
    let output = Command::new(bin)
        .arg("-d")
        .arg("-o")
        .arg(&dir)
        .arg(&staged)
        .output()
        .expect("invoke magic-pack");

    assert!(
        output.status.success(),
        "magic-pack -d failed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let expected = dir.join("hello.bin");
    assert!(
        expected.exists(),
        "expected {} to exist",
        expected.display()
    );
    let original = fs::read(fixture_dir().join("upx_unpacked_host.bin")).unwrap();
    assert_eq!(fs::read(&expected).unwrap(), original);
}
