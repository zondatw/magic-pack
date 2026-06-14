//! Integration tests for `service::list` (archive listing without
//! extraction). Builds fixtures with `service::compress`, then lists
//! them and checks entries, sizes, and the effective file type.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use magic_pack::contents::enums::FileType;
use magic_pack::service::{self, CompressRequest};

fn tempdir(label: &str) -> PathBuf {
    let dir =
        std::env::temp_dir().join(format!("magic-pack-list-{}-{}", label, std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn seed(root: &Path) {
    fs::create_dir_all(root.join("src/sub")).unwrap();
    fs::write(root.join("src/a.txt"), b"hello world\n").unwrap(); // 12
    fs::write(root.join("src/b.txt"), vec![b'x'; 5000]).unwrap(); // 5000
    fs::write(root.join("src/sub/c.txt"), b"nested\n").unwrap(); // 7
}

fn compress(dir: &Path, ft: FileType, out: &str) -> PathBuf {
    let archive = dir.join(out);
    service::compress(CompressRequest {
        file_type: ft,
        input: dir.join("src"),
        output: archive.clone(),
        password: None,
    })
    .unwrap();
    archive
}

/// Map of file-name → size for the non-dir entries.
fn file_sizes(entries: &[service::ArchiveEntry]) -> HashMap<String, u64> {
    entries
        .iter()
        .filter(|e| !e.is_dir)
        .map(|e| {
            // normalize: take the basename for robustness across formats
            let base = e.name.rsplit('/').next().unwrap_or(&e.name).to_string();
            (base, e.size)
        })
        .collect()
}

#[test]
fn list_zip_reports_entries_and_sizes() {
    let dir = tempdir("zip");
    seed(&dir);
    let archive = compress(&dir, FileType::Zip, "s.zip");

    let result = service::list(&archive).unwrap();
    assert_eq!(result.file_type, FileType::Zip);
    let sizes = file_sizes(&result.entries);
    assert_eq!(sizes.get("a.txt"), Some(&12));
    assert_eq!(sizes.get("b.txt"), Some(&5000));
    assert_eq!(sizes.get("c.txt"), Some(&7));
}

#[test]
fn list_tar_gz_lists_inner_tar_and_reports_targz() {
    let dir = tempdir("targz");
    seed(&dir);
    let archive = compress(&dir, FileType::Targz, "s.tar.gz");

    let result = service::list(&archive).unwrap();
    // Detected as gz, refined to tar.gz because it wraps a tar.
    assert_eq!(result.file_type, FileType::Targz);
    let sizes = file_sizes(&result.entries);
    assert_eq!(sizes.get("a.txt"), Some(&12));
    assert_eq!(sizes.get("b.txt"), Some(&5000));
    assert_eq!(sizes.get("c.txt"), Some(&7));
}

#[test]
fn list_7z_reports_entries() {
    let dir = tempdir("7z");
    seed(&dir);
    let archive = compress(&dir, FileType::SevenZ, "s.7z");

    let result = service::list(&archive).unwrap();
    assert_eq!(result.file_type, FileType::SevenZ);
    let sizes = file_sizes(&result.entries);
    assert_eq!(sizes.get("a.txt"), Some(&12));
    assert_eq!(sizes.get("b.txt"), Some(&5000));
}

#[test]
fn list_single_file_gz_reports_one_entry() {
    let dir = tempdir("gz");
    fs::write(dir.join("report.csv"), b"a,b,c\n1,2,3\n").unwrap();
    let archive = service::compress(CompressRequest {
        file_type: FileType::Gz,
        input: dir.join("report.csv"),
        output: dir.join("report.csv.gz"),
        password: None,
    })
    .unwrap();

    let result = service::list(&archive.output_path).unwrap();
    assert_eq!(result.file_type, FileType::Gz);
    assert_eq!(result.entries.len(), 1);
    // Inner logical name = input minus the .gz extension.
    assert_eq!(result.entries[0].name, "report.csv");
    assert!(!result.entries[0].is_dir);
}

#[test]
fn list_missing_file_errors_cleanly() {
    let dir = tempdir("missing");
    let result = service::list(&dir.join("nope.zip"));
    assert!(result.is_err());
}
