//! 7z AES-256 password tests. The whole file only compiles/runs when
//! the `encryption` feature is enabled.
//!
//! Happy-path roundtrip + header-hiding use the low-level
//! `modules::*_with_password` API (panic-on-error is fine there).
//! Wrong/missing-password and no-leak assertions go through
//! `service::decompress`, which converts the panic into a typed
//! `MagicPackError` we can inspect.
#![cfg(feature = "encryption")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use magic_pack::contents::enums::FileType;
use magic_pack::modules;
use magic_pack::service::{self, CompressRequest, DecompressRequest, MagicPackError};

const PASSWORD: &str = "correct horse battery staple";
const SECRET_ENTRY: &str = "topsecret_filename.txt";

fn make_unique_dir(name: &str) -> PathBuf {
    let base = PathBuf::from("target/tests");
    fs::create_dir_all(&base).expect("create base test dir");
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time")
        .as_nanos();
    let dir = base.join(format!("{}_{}_{}", name, process::id(), nanos));
    fs::create_dir_all(&dir).expect("create test dir");
    dir
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent");
    }
    fs::write(path, contents).expect("write file");
}

/// Build `srcdir/` with a distinctively-named file so we can later
/// assert the encrypted header hides it.
fn prepare_src_dir(root: &Path) -> PathBuf {
    let src_dir = root.join("srcdir");
    write_file(&src_dir.join(SECRET_ENTRY), "hello encrypted");
    write_file(&src_dir.join("sub/b.txt"), "world");
    src_dir
}

fn cleanup(path: &Path) {
    let _ = fs::remove_dir_all(path);
}

#[test]
fn encrypt_roundtrip_byte_equal() {
    let root = make_unique_dir("enc_roundtrip");
    let src_dir = prepare_src_dir(&root);

    let archive = root.join("out.7z");
    modules::compress_with_password(FileType::SevenZ, &src_dir, &archive, Some(PASSWORD), None);

    let unpack = root.join("unpack");
    fs::create_dir_all(&unpack).expect("create unpack dir");
    modules::decompress_with_password(FileType::SevenZ, &archive, &unpack, Some(PASSWORD), None);

    assert_eq!(
        fs::read_to_string(unpack.join("srcdir").join(SECRET_ENTRY)).expect("read secret"),
        "hello encrypted"
    );
    assert_eq!(
        fs::read_to_string(unpack.join("srcdir/sub/b.txt")).expect("read b"),
        "world"
    );
    cleanup(&root);
}

#[test]
fn encrypted_header_hides_filenames() {
    // AC-8: header encryption (the SevenZWriter default) means entry
    // names are not recoverable without the password. We assert two
    // things:
    //   1. the plaintext entry name never appears in the raw bytes, and
    //   2. the archive cannot even be opened/listed without the password
    //      (proven by a no-password decompress failing).
    // Note: a plain (unencrypted) 7z also LZMA-compresses its header, so
    // a raw-byte grep alone wouldn't distinguish compression from
    // encryption — hence the second, stronger check.
    let root = make_unique_dir("enc_header");
    let src_dir = prepare_src_dir(&root);

    let archive = root.join("out.7z");
    modules::compress_with_password(FileType::SevenZ, &src_dir, &archive, Some(PASSWORD), None);

    let bytes = fs::read(&archive).expect("read archive");
    let needle = SECRET_ENTRY.as_bytes();
    assert!(
        !bytes.windows(needle.len()).any(|w| w == needle),
        "encrypted 7z must not contain the plaintext entry name {:?}",
        SECRET_ENTRY
    );

    // Stronger: without the password the archive can't be read at all.
    let unpack = root.join("unpack");
    let no_pw = service::decompress(DecompressRequest {
        input: archive,
        output: unpack,
        level: 5,
        password: None,
    });
    assert!(
        matches!(no_pw, Err(MagicPackError::OperationFailed(_))),
        "encrypted archive must not be readable without the password, got {:?}",
        no_pw
    );
    cleanup(&root);
}

#[test]
fn wrong_password_fails_cleanly() {
    let root = make_unique_dir("enc_wrongpw");
    let src_dir = prepare_src_dir(&root);
    let archive = root.join("out.7z");
    modules::compress_with_password(FileType::SevenZ, &src_dir, &archive, Some(PASSWORD), None);

    let unpack = root.join("unpack");
    let result = service::decompress(DecompressRequest {
        input: archive,
        output: unpack,
        level: 5,
        password: Some(String::from("definitely-wrong")),
    });

    match result {
        Err(MagicPackError::OperationFailed(msg)) => {
            assert!(
                !msg.contains(PASSWORD) && !msg.contains("definitely-wrong"),
                "error must not leak any password, got: {}",
                msg
            );
        }
        other => panic!("expected OperationFailed, got {:?}", other),
    }
    cleanup(&root);
}

#[test]
fn missing_password_on_encrypted_fails() {
    let root = make_unique_dir("enc_nopw");
    let src_dir = prepare_src_dir(&root);
    let archive = root.join("out.7z");
    modules::compress_with_password(FileType::SevenZ, &src_dir, &archive, Some(PASSWORD), None);

    let unpack = root.join("unpack");
    let result = service::decompress(DecompressRequest {
        input: archive,
        output: unpack,
        level: 5,
        password: None,
    });

    match result {
        Err(MagicPackError::OperationFailed(msg)) => {
            assert!(
                msg.to_lowercase().contains("password") || msg.to_lowercase().contains("encrypt"),
                "missing-password error should hint at a password, got: {}",
                msg
            );
        }
        other => panic!("expected OperationFailed, got {:?}", other),
    }
    cleanup(&root);
}

#[test]
fn service_compress_then_decompress_roundtrips() {
    // Exercises the full service path (filename dance + password) for
    // both directions, the way the CLI/MCP actually call it.
    let root = make_unique_dir("enc_service");
    let src_dir = prepare_src_dir(&root);

    let archive = root.join("out.7z");
    service::compress(CompressRequest {
        file_type: FileType::SevenZ,
        input: src_dir,
        output: archive.clone(),
        password: Some(String::from(PASSWORD)),
    })
    .expect("encrypted compress should succeed");

    let unpack = root.join("unpack");
    service::decompress(DecompressRequest {
        input: archive,
        output: unpack.clone(),
        level: 5,
        password: Some(String::from(PASSWORD)),
    })
    .expect("encrypted decompress should succeed");

    // service decompress nests under <stem>/ — find the secret file.
    let secret = unpack.join("out").join("srcdir").join(SECRET_ENTRY);
    assert_eq!(
        fs::read_to_string(&secret).unwrap_or_else(|_| panic!("read {}", secret.display())),
        "hello encrypted"
    );
    cleanup(&root);
}

#[test]
fn unencrypted_7z_unchanged() {
    // AC-5 regression: the no-password path still works (also covered
    // by tests/roundtrip.rs::roundtrip_7z_dir under default features).
    let root = make_unique_dir("enc_plain");
    let src_dir = prepare_src_dir(&root);
    let archive = root.join("out.7z");
    modules::compress(FileType::SevenZ, &src_dir, &archive);

    let unpack = root.join("unpack");
    fs::create_dir_all(&unpack).expect("create unpack dir");
    modules::decompress(FileType::SevenZ, &archive, &unpack);

    assert_eq!(
        fs::read_to_string(unpack.join("srcdir").join(SECRET_ENTRY)).expect("read secret"),
        "hello encrypted"
    );
    cleanup(&root);
}
