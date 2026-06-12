//! Progress-reporting tests. The visible bar can only be confirmed in a
//! real TTY (manual dogfood), so here we deterministically assert the
//! shared byte counter the bar is driven by: it must reach the input
//! size on compress (covers both the single-file `io::copy` path and the
//! tar per-file path) and the archive size on decompress.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

use magic_pack::contents::enums::FileType;
use magic_pack::service::{self, CompressRequest, DecompressRequest};

fn tempdir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "magic-pack-progress-{}-{}",
        label,
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn gz_compress_counter_reaches_input_size() {
    let dir = tempdir("gz-c");
    let input = dir.join("data.bin");
    let bytes = vec![7u8; 200_000];
    fs::write(&input, &bytes).unwrap();

    let counter = Arc::new(AtomicU64::new(0));
    service::compress_with_progress(
        CompressRequest {
            file_type: FileType::Gz,
            input: input.clone(),
            output: dir.join("data.gz"),
            password: None,
        },
        Some(counter.clone()),
    )
    .unwrap();

    // io::copy reads the whole input → counter == input size exactly.
    assert_eq!(counter.load(Ordering::Relaxed), 200_000);
}

#[test]
fn gz_decompress_counter_reaches_archive_size() {
    let dir = tempdir("gz-d");
    let input = dir.join("data.bin");
    fs::write(&input, vec![3u8; 500_000]).unwrap();
    let archive = dir.join("data.gz");
    service::compress_with_progress(
        CompressRequest {
            file_type: FileType::Gz,
            input,
            output: archive.clone(),
            password: None,
        },
        None,
    )
    .unwrap();
    let archive_size = fs::metadata(&archive).unwrap().len();

    let counter = Arc::new(AtomicU64::new(0));
    service::decompress_with_progress(
        DecompressRequest {
            input: archive,
            output: dir.join("out"),
            level: 5,
            password: None,
        },
        Some(counter.clone()),
    )
    .unwrap();

    // The decoder consumes the whole compressed stream (incl. trailer).
    let counted = counter.load(Ordering::Relaxed);
    assert!(
        counted >= archive_size * 95 / 100 && counted <= archive_size,
        "counted {} vs archive {}",
        counted,
        archive_size
    );
}

#[test]
fn tar_zst_compress_counter_sums_file_sizes() {
    let dir = tempdir("tzst-c");
    let src = dir.join("src");
    fs::create_dir_all(src.join("sub")).unwrap();
    fs::write(src.join("a.bin"), vec![1u8; 100_000]).unwrap();
    fs::write(src.join("b.bin"), vec![2u8; 250_000]).unwrap();
    fs::write(src.join("sub/c.bin"), vec![3u8; 50_000]).unwrap();
    let total: u64 = 100_000 + 250_000 + 50_000;

    let counter = Arc::new(AtomicU64::new(0));
    service::compress_with_progress(
        CompressRequest {
            file_type: FileType::Tarzst,
            input: src,
            output: dir.join("src.tar.zst"),
            password: None,
        },
        Some(counter.clone()),
    )
    .unwrap();

    // Per-file increments sum to the total regular-file bytes.
    assert_eq!(counter.load(Ordering::Relaxed), total);
}

#[test]
fn reports_progress_capability_table() {
    use FileType::*;
    // compress: everything except 7z + UPX.
    for ft in [
        Zip, Tar, Bz2, Gz, Tarbz2, Targz, Xz, Tarxz, Zst, Tarzst, Lz4, Tarlz4,
    ] {
        assert!(
            ft.reports_compress_progress(),
            "{:?} should report compress",
            ft
        );
    }
    for ft in [SevenZ, Upx] {
        assert!(!ft.reports_compress_progress(), "{:?} should not", ft);
    }
    // decompress: same, minus zip.
    for ft in [
        Tar, Bz2, Gz, Tarbz2, Targz, Xz, Tarxz, Zst, Tarzst, Lz4, Tarlz4,
    ] {
        assert!(
            ft.reports_decompress_progress(),
            "{:?} should report decompress",
            ft
        );
    }
    for ft in [Zip, SevenZ, Upx] {
        assert!(!ft.reports_decompress_progress(), "{:?} should not", ft);
    }
}
