use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::time::{SystemTime, UNIX_EPOCH};

use magic_pack::contents::enums::FileType;
use magic_pack::modules;
use walkdir::WalkDir;
use std::sync::Once;

fn make_unique_dir(name: &str) -> PathBuf {
    let base = PathBuf::from("target/tests");
    fs::create_dir_all(&base).expect("create base test dir");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let pid = process::id();
    let dir = base.join(format!("{}_{}_{}", name, pid, nanos));
    fs::create_dir_all(&dir).expect("create test dir");
    dir
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, contents).expect("write file");
}

fn prepare_src_dir(root: &Path, name: &str) -> PathBuf {
    let src_dir = root.join(name);
    write_file(&src_dir.join("a.txt"), "alpha");
    write_file(&src_dir.join("sub/b.txt"), "bravo");
    src_dir
}

fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

fn find_file_by_suffix(root: &Path, suffix: &str) -> Option<PathBuf> {
    for entry in WalkDir::new(root).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        if path.is_file() && path.to_string_lossy().ends_with(suffix) {
            return Some(path.to_path_buf());
        }
    }
    None
}

fn tar_available() -> bool {
    Command::new("tar")
        .arg("--version")
        .output()
        .is_ok()
}

fn warn_missing_tar_once() {
    static WARN_ONCE: Once = Once::new();
    WARN_ONCE.call_once(|| {
        eprintln!("warning: `tar` not found in PATH; skipping interop tar tests");
    });
}

#[test]
fn tar_command_compress_tool_decompress() {
    if !tar_available() {
        warn_missing_tar_once();
        return;
    }

    let root = make_unique_dir("interop_tar_cmd_to_tool");
    let _src_dir = prepare_src_dir(&root, "srcdir");
    let archive_name = "from_tar_cmd.tar";
    let archive = root.join(archive_name);

    let status = Command::new("tar")
        .current_dir(&root)
        .arg("-cf")
        .arg(archive_name)
        .arg("srcdir")
        .status()
        .expect("run tar");
    assert!(status.success(), "tar compress failed");

    let unpack = root.join("unpack");
    modules::decompress(FileType::Tar, &archive, &unpack);

    let file_a = find_file_by_suffix(&unpack, "srcdir/a.txt").expect("find a.txt");
    let file_b = find_file_by_suffix(&unpack, "srcdir/sub/b.txt").expect("find b.txt");
    assert_eq!(fs::read_to_string(file_a).expect("read a.txt"), "alpha");
    assert_eq!(fs::read_to_string(file_b).expect("read b.txt"), "bravo");

    cleanup_dir(&root);
}

#[test]
fn tool_compress_tar_command_decompress() {
    if !tar_available() {
        warn_missing_tar_once();
        return;
    }

    let root = make_unique_dir("interop_tool_to_tar_cmd");
    let src_dir = prepare_src_dir(&root, "srcdir");
    let archive_name = "from_tool.tar";
    let archive = root.join(archive_name);

    modules::compress(FileType::Tar, &src_dir, &archive);

    let output = Command::new("tar")
        .current_dir(&root)
        .arg("-tf")
        .arg(archive_name)
        .output()
        .expect("run tar");
    assert!(output.status.success(), "tar list failed");

    let listing = String::from_utf8_lossy(&output.stdout);
    assert!(
        listing.contains("srcdir/a.txt"),
        "tar listing missing srcdir/a.txt"
    );
    assert!(
        listing.contains("srcdir/sub/b.txt"),
        "tar listing missing srcdir/sub/b.txt"
    );

    cleanup_dir(&root);
}
