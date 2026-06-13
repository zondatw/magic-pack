use std::ffi::OsString;
use std::fs::File;
use std::io;
use std::path::{Path, PathBuf};

use sevenz_rust::{SevenZArchiveEntry, SevenZWriter};
use walkdir::WalkDir;

use super::ArchiveEntry;

/// List 7z entries from headers only (`Archive::open` reads the header,
/// no decompression). Encrypted-header archives need a password and
/// error here — a documented follow-up.
pub fn list(src_path: &Path) -> io::Result<Vec<ArchiveEntry>> {
    let archive =
        sevenz_rust::Archive::open(src_path).map_err(|e| io::Error::other(e.to_string()))?;
    Ok(archive
        .files
        .iter()
        .map(|e| ArchiveEntry {
            name: e.name().to_string(),
            size: e.size(),
            is_dir: e.is_directory(),
        })
        .collect())
}

fn archive_path(src_root: &Path, entry_path: &Path) -> PathBuf {
    let base: Option<OsString> = src_root.file_name().map(|s| s.to_os_string());
    if entry_path == src_root {
        if let Some(base) = base {
            return PathBuf::from(base);
        }
    }
    match entry_path.strip_prefix(src_root) {
        Ok(rel) => match base {
            Some(base) => PathBuf::from(base).join(rel),
            None => rel.to_path_buf(),
        },
        Err(_) => entry_path.to_path_buf(),
    }
}

pub fn compress(src_path: &Path, dst_path: &Path, password: Option<&str>) {
    let dst_file = File::create(dst_path).expect("7z create failed");
    let mut writer = SevenZWriter::new(dst_file).expect("7z writer init failed");

    match password {
        Some(pw) if !pw.is_empty() => set_encryption(&mut writer, pw),
        _ => {}
    }

    for entry in WalkDir::new(src_path).into_iter().filter_map(|e| e.ok()) {
        let path = entry.path();
        let name = archive_path(src_path, path).to_string_lossy().to_string();

        if path.is_dir() {
            writer
                .push_archive_entry::<File>(SevenZArchiveEntry::from_path(path, name), None)
                .expect("7z add dir failed");
        } else {
            let file = File::open(path).expect("7z open source file failed");
            writer
                .push_archive_entry(SevenZArchiveEntry::from_path(path, name), Some(file))
                .expect("7z add file failed");
        }
    }

    writer.finish().expect("7z finish failed");
}

pub fn decompress(src_path: &Path, dst_path: &Path, password: Option<&str>) {
    match password {
        Some(pw) if !pw.is_empty() => decompress_encrypted(src_path, dst_path, pw),
        // No password: existing path. If the archive turns out to be
        // encrypted, surface an actionable hint instead of a raw debug
        // dump. The sevenz error never contains the password.
        _ => {
            if let Err(err) = sevenz_rust::decompress_file(src_path, dst_path) {
                if is_password_error(&err) {
                    panic!("7z archive is encrypted; provide a password with -p (CLI) or the `password` argument (MCP)");
                }
                panic!("7z decompress failed: {}", err);
            }
        }
    }
}

/// True for sevenz errors that indicate the archive needs a (correct)
/// password. Used to translate raw crate errors into actionable hints.
fn is_password_error(err: &sevenz_rust::Error) -> bool {
    matches!(
        err,
        sevenz_rust::Error::PasswordRequired | sevenz_rust::Error::MaybeBadPassword(_)
    )
}

/// Configure the writer for AES-256 + LZMA2 content encryption. Mirrors
/// sevenz-rust's own `compress_encypted` helper. Header encryption stays
/// at the `SevenZWriter` default (on), so entry filenames are encrypted
/// too. The panic-on-no-support arm keeps the unencrypted path working
/// in lean builds while giving a clear message if a password somehow
/// reaches a non-encryption build.
#[cfg(feature = "encryption")]
fn set_encryption(writer: &mut SevenZWriter<File>, password: &str) {
    use sevenz_rust::{AesEncoderOptions, Password, SevenZMethod};

    writer.set_content_methods(vec![
        AesEncoderOptions::new(Password::from(password)).into(),
        SevenZMethod::LZMA2.into(),
    ]);
}

#[cfg(not(feature = "encryption"))]
fn set_encryption(_writer: &mut SevenZWriter<File>, _password: &str) {
    panic!(
        "this build has no encryption support; rebuild with --features encryption \
         to use a 7z password"
    );
}

#[cfg(feature = "encryption")]
fn decompress_encrypted(src_path: &Path, dst_path: &Path, password: &str) {
    use sevenz_rust::Password;

    if let Err(err) =
        sevenz_rust::decompress_file_with_password(src_path, dst_path, Password::from(password))
    {
        // Never interpolate `password` into the message.
        if is_password_error(&err) {
            panic!("7z decryption failed: wrong password");
        }
        panic!("7z decompress with password failed: {}", err);
    }
}

#[cfg(not(feature = "encryption"))]
fn decompress_encrypted(_src_path: &Path, _dst_path: &Path, _password: &str) {
    panic!(
        "this build has no encryption support; rebuild with --features encryption \
         to decrypt a password-protected 7z"
    );
}
