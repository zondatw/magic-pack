use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use magic_pack::contents::enums::FileType;
use magic_pack::modules;

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
    write_file(&src_dir.join("a.txt"), "hello");
    write_file(&src_dir.join("sub/b.txt"), "world");
    src_dir
}

fn cleanup_dir(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

#[test]
fn roundtrip_gz() {
    let root = make_unique_dir("roundtrip_gz");
    let src = root.join("src.txt");
    write_file(&src, "hello gz");

    let compressed = root.join("out.gz");
    modules::compress(FileType::Gz, &src, &compressed);

    let decompressed = root.join("out.txt");
    modules::decompress(FileType::Gz, &compressed, &decompressed);

    let contents = fs::read_to_string(&decompressed).expect("read decompressed");
    assert_eq!(contents, "hello gz");
    cleanup_dir(&root);
}

#[test]
fn roundtrip_bz2() {
    let root = make_unique_dir("roundtrip_bz2");
    let src = root.join("src.txt");
    write_file(&src, "hello bz2");

    let compressed = root.join("out.bz2");
    modules::compress(FileType::Bz2, &src, &compressed);

    let decompressed = root.join("out.txt");
    modules::decompress(FileType::Bz2, &compressed, &decompressed);

    let contents = fs::read_to_string(&decompressed).expect("read decompressed");
    assert_eq!(contents, "hello bz2");
    cleanup_dir(&root);
}

#[test]
fn roundtrip_zip_dir() {
    let root = make_unique_dir("roundtrip_zip");
    let src_dir = prepare_src_dir(&root, "srcdir");

    let compressed = root.join("out.zip");
    modules::compress(FileType::Zip, &src_dir, &compressed);

    let unpack = root.join("unpack");
    modules::decompress(FileType::Zip, &compressed, &unpack);

    let file_a = unpack.join("srcdir/a.txt");
    let file_b = unpack.join("srcdir/sub/b.txt");

    assert_eq!(fs::read_to_string(file_a).expect("read a.txt"), "hello");
    assert_eq!(fs::read_to_string(file_b).expect("read b.txt"), "world");
    cleanup_dir(&root);
}

#[test]
fn roundtrip_tar_dir() {
    let root = make_unique_dir("roundtrip_tar");
    let src_dir = prepare_src_dir(&root, "srcdir");

    let compressed = root.join("out.tar");
    modules::compress(FileType::Tar, &src_dir, &compressed);

    let unpack = root.join("unpack");
    fs::create_dir_all(&unpack).expect("create unpack dir");
    modules::decompress(FileType::Tar, &compressed, &unpack);

    let file_a = unpack.join("srcdir/a.txt");
    let file_b = unpack.join("srcdir/sub/b.txt");

    assert_eq!(fs::read_to_string(file_a).expect("read a.txt"), "hello");
    assert_eq!(fs::read_to_string(file_b).expect("read b.txt"), "world");
    cleanup_dir(&root);
}

#[test]
fn roundtrip_tar_gz_dir() {
    let root = make_unique_dir("roundtrip_targz");
    let src_dir = prepare_src_dir(&root, "srcdir");

    let compressed = root.join("out.tar.gz");
    modules::compress(FileType::Targz, &src_dir, &compressed);

    let unpack = root.join("unpack");
    fs::create_dir_all(&unpack).expect("create unpack dir");
    modules::decompress(FileType::Targz, &compressed, &unpack);

    let file_a = unpack.join("srcdir/a.txt");
    let file_b = unpack.join("srcdir/sub/b.txt");

    assert_eq!(fs::read_to_string(file_a).expect("read a.txt"), "hello");
    assert_eq!(fs::read_to_string(file_b).expect("read b.txt"), "world");
    cleanup_dir(&root);
}

#[test]
fn roundtrip_tar_bz2_dir() {
    let root = make_unique_dir("roundtrip_tarbz2");
    let src_dir = prepare_src_dir(&root, "srcdir");

    let compressed = root.join("out.tar.bz2");
    modules::compress(FileType::Tarbz2, &src_dir, &compressed);

    let unpack = root.join("unpack");
    fs::create_dir_all(&unpack).expect("create unpack dir");
    modules::decompress(FileType::Tarbz2, &compressed, &unpack);

    let file_a = unpack.join("srcdir/a.txt");
    let file_b = unpack.join("srcdir/sub/b.txt");

    assert_eq!(fs::read_to_string(file_a).expect("read a.txt"), "hello");
    assert_eq!(fs::read_to_string(file_b).expect("read b.txt"), "world");
    cleanup_dir(&root);
}

#[test]
fn roundtrip_7z_dir() {
    let root = make_unique_dir("roundtrip_7z");
    let src_dir = prepare_src_dir(&root, "srcdir");

    let compressed = root.join("out.7z");
    modules::compress(FileType::SevenZ, &src_dir, &compressed);

    let unpack = root.join("unpack");
    fs::create_dir_all(&unpack).expect("create unpack dir");
    modules::decompress(FileType::SevenZ, &compressed, &unpack);

    let file_a = unpack.join("srcdir/a.txt");
    let file_b = unpack.join("srcdir/sub/b.txt");

    assert_eq!(fs::read_to_string(file_a).expect("read a.txt"), "hello");
    assert_eq!(fs::read_to_string(file_b).expect("read b.txt"), "world");
    cleanup_dir(&root);
}

#[test]
fn roundtrip_xz() {
    let root = make_unique_dir("roundtrip_xz");
    let src = root.join("src.txt");
    write_file(&src, "hello xz");

    let compressed = root.join("out.xz");
    modules::compress(FileType::Xz, &src, &compressed);

    let decompressed = root.join("out.txt");
    modules::decompress(FileType::Xz, &compressed, &decompressed);

    let contents = fs::read_to_string(&decompressed).expect("read decompressed");
    assert_eq!(contents, "hello xz");
    cleanup_dir(&root);
}

#[test]
fn roundtrip_tar_xz_dir() {
    let root = make_unique_dir("roundtrip_tarxz");
    let src_dir = prepare_src_dir(&root, "srcdir");

    let compressed = root.join("out.tar.xz");
    modules::compress(FileType::Tarxz, &src_dir, &compressed);

    let unpack = root.join("unpack");
    fs::create_dir_all(&unpack).expect("create unpack dir");
    modules::decompress(FileType::Tarxz, &compressed, &unpack);

    let file_a = unpack.join("srcdir/a.txt");
    let file_b = unpack.join("srcdir/sub/b.txt");

    assert_eq!(fs::read_to_string(file_a).expect("read a.txt"), "hello");
    assert_eq!(fs::read_to_string(file_b).expect("read b.txt"), "world");
    cleanup_dir(&root);
}

#[test]
fn roundtrip_zst() {
    let root = make_unique_dir("roundtrip_zst");
    let src = root.join("src.txt");
    write_file(&src, "hello zst");

    let compressed = root.join("out.zst");
    modules::compress(FileType::Zst, &src, &compressed);

    let decompressed = root.join("out.txt");
    modules::decompress(FileType::Zst, &compressed, &decompressed);

    let contents = fs::read_to_string(&decompressed).expect("read decompressed");
    assert_eq!(contents, "hello zst");
    cleanup_dir(&root);
}

#[test]
fn roundtrip_tar_zst_dir() {
    let root = make_unique_dir("roundtrip_tarzst");
    let src_dir = prepare_src_dir(&root, "srcdir");

    let compressed = root.join("out.tar.zst");
    modules::compress(FileType::Tarzst, &src_dir, &compressed);

    let unpack = root.join("unpack");
    fs::create_dir_all(&unpack).expect("create unpack dir");
    modules::decompress(FileType::Tarzst, &compressed, &unpack);

    let file_a = unpack.join("srcdir/a.txt");
    let file_b = unpack.join("srcdir/sub/b.txt");

    assert_eq!(fs::read_to_string(file_a).expect("read a.txt"), "hello");
    assert_eq!(fs::read_to_string(file_b).expect("read b.txt"), "world");
    cleanup_dir(&root);
}

#[test]
fn detect_file_types() {
    let root = make_unique_dir("detect_file_types");

    let bz2_file = root.join("sample.bz2");
    fs::write(&bz2_file, b"BZh9").expect("write bz2");
    assert_eq!(modules::get_file_type(&bz2_file).unwrap(), FileType::Bz2);

    let gz_file = root.join("sample.gz");
    fs::write(&gz_file, [0x1f, 0x8b, 0x08, 0x00]).expect("write gz");
    assert_eq!(modules::get_file_type(&gz_file).unwrap(), FileType::Gz);

    let zip_file = root.join("sample.zip");
    fs::write(&zip_file, [0x50, 0x4b, 0x03, 0x04]).expect("write zip");
    assert_eq!(modules::get_file_type(&zip_file).unwrap(), FileType::Zip);

    let tar_file = root.join("sample.tar");
    fs::write(&tar_file, b"ustar").expect("write tar");
    assert_eq!(modules::get_file_type(&tar_file).unwrap(), FileType::Tar);

    let sevenz_file = root.join("sample.7z");
    fs::write(&sevenz_file, [0x37, 0x7a, 0xbc, 0xaf]).expect("write 7z");
    assert_eq!(
        modules::get_file_type(&sevenz_file).unwrap(),
        FileType::SevenZ
    );

    let xz_file = root.join("sample.xz");
    fs::write(&xz_file, [0xfd, 0x37, 0x7a, 0x58]).expect("write xz");
    assert_eq!(modules::get_file_type(&xz_file).unwrap(), FileType::Xz);

    let zst_file = root.join("sample.zst");
    fs::write(&zst_file, [0x28, 0xb5, 0x2f, 0xfd]).expect("write zst");
    assert_eq!(modules::get_file_type(&zst_file).unwrap(), FileType::Zst);

    cleanup_dir(&root);
}
