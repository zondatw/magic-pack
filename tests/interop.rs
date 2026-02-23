use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::time::{SystemTime, UNIX_EPOCH};

use magic_pack::contents::enums::FileType;
use magic_pack::modules;
use std::sync::Once;
use walkdir::WalkDir;

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
    Command::new("tar").arg("--version").output().is_ok()
}

fn warn_missing_tar_once() {
    static WARN_ONCE: Once = Once::new();
    WARN_ONCE.call_once(|| {
        eprintln!("warning: `tar` not found in PATH; skipping interop tar tests");
    });
}

fn gzip_available() -> bool {
    Command::new("gzip").arg("--version").output().is_ok()
}

fn warn_missing_gzip_once() {
    static WARN_ONCE: Once = Once::new();
    WARN_ONCE.call_once(|| {
        eprintln!("warning: `gzip` not found in PATH; skipping gzip interop tests");
    });
}

fn bzip2_available() -> bool {
    Command::new("bzip2").arg("--version").output().is_ok()
}

fn warn_missing_bzip2_once() {
    static WARN_ONCE: Once = Once::new();
    WARN_ONCE.call_once(|| {
        eprintln!("warning: `bzip2` not found in PATH; skipping bzip2 interop tests");
    });
}

fn zip_available() -> bool {
    Command::new("zip").arg("-v").output().is_ok()
        && Command::new("unzip").arg("-v").output().is_ok()
}

fn warn_missing_zip_once() {
    static WARN_ONCE: Once = Once::new();
    WARN_ONCE.call_once(|| {
        eprintln!("warning: `zip`/`unzip` not found in PATH; skipping zip interop tests");
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
    fs::create_dir_all(&unpack).expect("create unpack dir");
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

#[test]
fn gzip_command_compress_tool_decompress() {
    if !gzip_available() {
        warn_missing_gzip_once();
        return;
    }

    let root = make_unique_dir("interop_gzip_cmd_to_tool");
    let src = root.join("src.txt");
    write_file(&src, "hello gzip");
    let archive = root.join("from_gzip_cmd.gz");

    let output = Command::new("gzip")
        .arg("-c")
        .arg(&src)
        .output()
        .expect("run gzip");
    assert!(output.status.success(), "gzip compress failed");
    fs::write(&archive, output.stdout).expect("write gzip output");

    let decompressed = root.join("out.txt");
    modules::decompress(FileType::Gz, &archive, &decompressed);
    let contents = fs::read_to_string(&decompressed).expect("read decompressed");
    assert_eq!(contents, "hello gzip");

    cleanup_dir(&root);
}

#[test]
fn tool_compress_gzip_command_decompress() {
    if !gzip_available() {
        warn_missing_gzip_once();
        return;
    }

    let root = make_unique_dir("interop_tool_to_gzip_cmd");
    let src = root.join("src.txt");
    write_file(&src, "hello gzip");
    let archive = root.join("from_tool.gz");

    modules::compress(FileType::Gz, &src, &archive);

    let output = Command::new("gzip")
        .arg("-dc")
        .arg(&archive)
        .output()
        .expect("run gzip");
    assert!(output.status.success(), "gzip decompress failed");
    let contents = String::from_utf8_lossy(&output.stdout);
    assert_eq!(contents, "hello gzip");

    cleanup_dir(&root);
}

#[test]
fn bzip2_command_compress_tool_decompress() {
    if !bzip2_available() {
        warn_missing_bzip2_once();
        return;
    }

    let root = make_unique_dir("interop_bzip2_cmd_to_tool");
    let src = root.join("src.txt");
    write_file(&src, "hello bzip2");
    let archive = root.join("from_bzip2_cmd.bz2");

    let output = Command::new("bzip2")
        .arg("-c")
        .arg(&src)
        .output()
        .expect("run bzip2");
    assert!(output.status.success(), "bzip2 compress failed");
    fs::write(&archive, output.stdout).expect("write bzip2 output");

    let decompressed = root.join("out.txt");
    modules::decompress(FileType::Bz2, &archive, &decompressed);
    let contents = fs::read_to_string(&decompressed).expect("read decompressed");
    assert_eq!(contents, "hello bzip2");

    cleanup_dir(&root);
}

#[test]
fn tool_compress_bzip2_command_decompress() {
    if !bzip2_available() {
        warn_missing_bzip2_once();
        return;
    }

    let root = make_unique_dir("interop_tool_to_bzip2_cmd");
    let src = root.join("src.txt");
    write_file(&src, "hello bzip2");
    let archive = root.join("from_tool.bz2");

    modules::compress(FileType::Bz2, &src, &archive);

    let output = Command::new("bzip2")
        .arg("-dc")
        .arg(&archive)
        .output()
        .expect("run bzip2");
    assert!(output.status.success(), "bzip2 decompress failed");
    let contents = String::from_utf8_lossy(&output.stdout);
    assert_eq!(contents, "hello bzip2");

    cleanup_dir(&root);
}

#[test]
fn zip_command_compress_tool_decompress() {
    if !zip_available() {
        warn_missing_zip_once();
        return;
    }

    let root = make_unique_dir("interop_zip_cmd_to_tool");
    let _src_dir = prepare_src_dir(&root, "srcdir");
    let archive_name = "from_zip_cmd.zip";
    let archive = root.join(archive_name);

    let status = Command::new("zip")
        .current_dir(&root)
        .arg("-r")
        .arg(archive_name)
        .arg("srcdir")
        .status()
        .expect("run zip");
    assert!(status.success(), "zip compress failed");

    let unpack = root.join("unpack");
    modules::decompress(FileType::Zip, &archive, &unpack);

    let file_a = find_file_by_suffix(&unpack, "srcdir/a.txt").expect("find a.txt");
    let file_b = find_file_by_suffix(&unpack, "srcdir/sub/b.txt").expect("find b.txt");
    assert_eq!(fs::read_to_string(file_a).expect("read a.txt"), "alpha");
    assert_eq!(fs::read_to_string(file_b).expect("read b.txt"), "bravo");

    cleanup_dir(&root);
}

#[test]
fn tool_compress_zip_command_list() {
    if !zip_available() {
        warn_missing_zip_once();
        return;
    }

    let root = make_unique_dir("interop_tool_to_zip_cmd");
    let _src_dir = prepare_src_dir(&root, "srcdir");
    let archive_name = "from_tool.zip";
    let archive = root.join(archive_name);

    modules::compress(FileType::Zip, &root.join("srcdir"), &archive);

    let output = Command::new("unzip")
        .current_dir(&root)
        .arg("-l")
        .arg(archive_name)
        .output()
        .expect("run unzip");
    assert!(output.status.success(), "unzip list failed");
    let listing = String::from_utf8_lossy(&output.stdout);
    assert!(
        listing.contains("srcdir/a.txt"),
        "zip listing missing srcdir/a.txt"
    );
    assert!(
        listing.contains("srcdir/sub/b.txt"),
        "zip listing missing srcdir/sub/b.txt"
    );

    cleanup_dir(&root);
}

#[test]
fn tar_gz_command_compress_tool_decompress() {
    if !tar_available() {
        warn_missing_tar_once();
        return;
    }

    let root = make_unique_dir("interop_targz_cmd_to_tool");
    let _src_dir = prepare_src_dir(&root, "srcdir");
    let archive_name = "from_tar_cmd.tar.gz";
    let archive = root.join(archive_name);

    let status = Command::new("tar")
        .current_dir(&root)
        .arg("-czf")
        .arg(archive_name)
        .arg("srcdir")
        .status()
        .expect("run tar");
    assert!(status.success(), "tar.gz compress failed");

    let unpack = root.join("unpack");
    fs::create_dir_all(&unpack).expect("create unpack dir");
    modules::decompress(FileType::Targz, &archive, &unpack);

    let file_a = find_file_by_suffix(&unpack, "srcdir/a.txt").expect("find a.txt");
    let file_b = find_file_by_suffix(&unpack, "srcdir/sub/b.txt").expect("find b.txt");
    assert_eq!(fs::read_to_string(file_a).expect("read a.txt"), "alpha");
    assert_eq!(fs::read_to_string(file_b).expect("read b.txt"), "bravo");

    cleanup_dir(&root);
}

#[test]
fn tar_bz2_command_compress_tool_decompress() {
    if !tar_available() {
        warn_missing_tar_once();
        return;
    }

    let root = make_unique_dir("interop_tarbz2_cmd_to_tool");
    let _src_dir = prepare_src_dir(&root, "srcdir");
    let archive_name = "from_tar_cmd.tar.bz2";
    let archive = root.join(archive_name);

    let status = Command::new("tar")
        .current_dir(&root)
        .arg("-cjf")
        .arg(archive_name)
        .arg("srcdir")
        .status()
        .expect("run tar");
    assert!(status.success(), "tar.bz2 compress failed");

    let unpack = root.join("unpack");
    fs::create_dir_all(&unpack).expect("create unpack dir");
    modules::decompress(FileType::Tarbz2, &archive, &unpack);

    let file_a = find_file_by_suffix(&unpack, "srcdir/a.txt").expect("find a.txt");
    let file_b = find_file_by_suffix(&unpack, "srcdir/sub/b.txt").expect("find b.txt");
    assert_eq!(fs::read_to_string(file_a).expect("read a.txt"), "alpha");
    assert_eq!(fs::read_to_string(file_b).expect("read b.txt"), "bravo");

    cleanup_dir(&root);
}

#[test]
fn tool_compress_targz_command_list() {
    if !tar_available() {
        warn_missing_tar_once();
        return;
    }

    let root = make_unique_dir("interop_tool_to_targz_cmd");
    let _src_dir = prepare_src_dir(&root, "srcdir");
    let archive_name = "from_tool.tar.gz";
    let archive = root.join(archive_name);

    modules::compress(FileType::Targz, &root.join("srcdir"), &archive);

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

#[test]
fn tool_compress_tarbz2_command_list() {
    if !tar_available() {
        warn_missing_tar_once();
        return;
    }

    let root = make_unique_dir("interop_tool_to_tarbz2_cmd");
    let _src_dir = prepare_src_dir(&root, "srcdir");
    let archive_name = "from_tool.tar.bz2";
    let archive = root.join(archive_name);

    modules::compress(FileType::Tarbz2, &root.join("srcdir"), &archive);

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
