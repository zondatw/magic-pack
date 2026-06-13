//! List an archive's contents without extracting.
//!
//! Reads only the central directory / entry headers, so nothing is
//! written to disk. Container formats (zip, tar.*, 7z) yield one
//! [`ArchiveEntry`] per member. The single-file codecs (gz/bz2/xz/zst/
//! lz4) are peeked one layer deep: if they wrap a tar (the common
//! `.tar.gz` case) the tar's members are listed and the effective type
//! is reported (e.g. `tar.gz`); otherwise a single synthetic entry for
//! the wrapped stream. UPX is an executable packer, not an archive.

use std::fs::File;
use std::io::{self, BufReader, Error, ErrorKind, Read};
use std::path::Path;

use flate2::read::GzDecoder;

use crate::contents::enums::FileType;

/// One member of an archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    pub name: String,
    /// Uncompressed size in bytes. `0` for directories and for a
    /// single-file stream (whose payload size isn't recorded up front).
    pub size: u64,
    pub is_dir: bool,
}

fn other<E: std::fmt::Display>(err: E) -> Error {
    Error::other(err.to_string())
}

/// List the contents of `src`, detected as `file_type`. Returns the
/// *effective* type (refined to a `tar.*` variant when a single-file
/// codec is found to wrap a tar) alongside the entries.
pub fn list(file_type: FileType, src: &Path) -> io::Result<(FileType, Vec<ArchiveEntry>)> {
    match file_type {
        FileType::Zip => Ok((file_type, list_zip(src)?)),
        FileType::Tar => Ok((file_type, list_tar(File::open(src)?)?)),
        FileType::Targz => Ok((file_type, list_tar(GzDecoder::new(File::open(src)?))?)),
        FileType::Tarbz2 => Ok((
            file_type,
            list_tar(bzip2::read::BzDecoder::new(File::open(src)?))?,
        )),
        FileType::Tarxz => Ok((
            file_type,
            list_tar(xz2::read::XzDecoder::new(File::open(src)?))?,
        )),
        FileType::Tarzst => Ok((file_type, list_tar(zstd::Decoder::new(File::open(src)?)?)?)),
        FileType::Tarlz4 => Ok((
            file_type,
            list_tar(lz4_flex::frame::FrameDecoder::new(File::open(src)?))?,
        )),
        FileType::SevenZ => Ok((file_type, list_7z(src)?)),
        FileType::Gz | FileType::Bz2 | FileType::Xz | FileType::Zst | FileType::Lz4 => {
            list_single_or_tar(file_type, src)
        }
        FileType::Upx => Err(Error::other(
            "UPX is an executable packer, not an archive; use -d to unpack it",
        )),
    }
}

fn list_zip(src: &Path) -> io::Result<Vec<ArchiveEntry>> {
    let file = File::open(src)?;
    let mut archive = zip::ZipArchive::new(BufReader::new(file)).map_err(other)?;
    let mut entries = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        let entry = archive.by_index(i).map_err(other)?;
        entries.push(ArchiveEntry {
            name: entry.name().to_string(),
            size: entry.size(),
            is_dir: entry.is_dir(),
        });
    }
    Ok(entries)
}

/// Iterate tar entry headers without unpacking to disk.
fn list_tar<R: Read>(reader: R) -> io::Result<Vec<ArchiveEntry>> {
    let mut archive = tar::Archive::new(reader);
    let mut entries = Vec::new();
    for entry in archive.entries()? {
        let entry = entry?;
        let header = entry.header();
        let size = header.size().unwrap_or(0);
        let is_dir = header.entry_type().is_dir();
        let name = entry
            .path()
            .map(|p| p.to_string_lossy().into_owned())
            .unwrap_or_default();
        entries.push(ArchiveEntry { name, size, is_dir });
    }
    Ok(entries)
}

fn list_7z(src: &Path) -> io::Result<Vec<ArchiveEntry>> {
    // `Archive::open` reads headers only (no decompression). Encrypted-
    // header archives need a password and will error here — listing an
    // encrypted 7z is a documented follow-up.
    let archive = sevenz_rust::Archive::open(src).map_err(other)?;
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

/// A single-file codec wraps one stream. Peek one layer: if the decoded
/// stream is a tar (the `.tar.gz` family) list the tar and report the
/// `tar.*` type; otherwise report one entry for the wrapped file.
fn list_single_or_tar(
    file_type: FileType,
    src: &Path,
) -> io::Result<(FileType, Vec<ArchiveEntry>)> {
    let mut head = [0u8; 512];
    let n = read_fill(&mut decoder_for(file_type, src)?, &mut head)?;
    let is_tar = n >= 262 && &head[257..262] == b"ustar";

    if is_tar {
        let entries = list_tar(decoder_for(file_type, src)?)?;
        Ok((tar_variant_of(file_type), entries))
    } else {
        Ok((file_type, vec![single_file_entry(src)]))
    }
}

fn decoder_for(file_type: FileType, src: &Path) -> io::Result<Box<dyn Read>> {
    let file = File::open(src)?;
    Ok(match file_type {
        FileType::Gz => Box::new(GzDecoder::new(file)),
        FileType::Bz2 => Box::new(bzip2::read::BzDecoder::new(file)),
        FileType::Xz => Box::new(xz2::read::XzDecoder::new(file)),
        FileType::Zst => Box::new(zstd::Decoder::new(file)?),
        FileType::Lz4 => Box::new(lz4_flex::frame::FrameDecoder::new(file)),
        _ => return Err(other("not a single-file codec")),
    })
}

fn tar_variant_of(file_type: FileType) -> FileType {
    match file_type {
        FileType::Gz => FileType::Targz,
        FileType::Bz2 => FileType::Tarbz2,
        FileType::Xz => FileType::Tarxz,
        FileType::Zst => FileType::Tarzst,
        FileType::Lz4 => FileType::Tarlz4,
        other => other,
    }
}

/// Read until `buf` is full or the reader hits EOF; returns bytes read.
fn read_fill(reader: &mut dyn Read, buf: &mut [u8]) -> io::Result<usize> {
    let mut filled = 0;
    while filled < buf.len() {
        match reader.read(&mut buf[filled..]) {
            Ok(0) => break,
            Ok(n) => filled += n,
            Err(ref e) if e.kind() == ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(filled)
}

fn single_file_entry(src: &Path) -> ArchiveEntry {
    let name = src
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_else(|| String::from("(stream)"));
    ArchiveEntry {
        name,
        size: 0,
        is_dir: false,
    }
}
