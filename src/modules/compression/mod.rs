pub mod bz2;
pub mod gz;
pub mod lz4;
pub mod sevenz;
pub mod tar;
pub mod tar_bz2;
pub mod tar_gz;
pub mod tar_lz4;
pub mod tar_xz;
pub mod tar_zst;
pub mod upx;
pub mod xz;
pub mod zip;
pub mod zst;

use std::io::{self, Read};
use std::path::Path;

use crate::contents::enums::FileType;

/// One member of an archive, produced by each codec's `list`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveEntry {
    pub name: String,
    /// Uncompressed size in bytes. `0` for directories and for a
    /// single-file stream (whose payload size isn't recorded up front).
    pub size: u64,
    pub is_dir: bool,
}

/// True if the start of a (decompressed) stream looks like a tar — the
/// POSIX `ustar` magic sits at offset 257.
pub(crate) fn looks_like_tar(head: &[u8]) -> bool {
    head.len() >= 262 && &head[257..262] == b"ustar"
}

/// Read until `buf` is full or the reader hits EOF; returns bytes read.
pub(crate) fn read_fill(reader: &mut dyn Read, buf: &mut [u8]) -> io::Result<usize> {
    let mut filled = 0;
    while filled < buf.len() {
        match reader.read(&mut buf[filled..]) {
            Ok(0) => break,
            Ok(n) => filled += n,
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
    Ok(filled)
}

/// One synthetic entry for a single-file stream: the logical inner name
/// is the input filename minus its compression extension.
pub(crate) fn single_file_entry(src: &Path) -> ArchiveEntry {
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

/// Shared listing for the single-file codecs. Peeks one layer through
/// `make_decoder`: if the decoded stream is a tar (the `.tar.gz` family)
/// the inner tar is listed and `tar_type` reported; otherwise a single
/// entry for the wrapped stream and `single_type`.
pub(crate) fn list_single_or_tar<F, R>(
    src: &Path,
    single_type: FileType,
    tar_type: FileType,
    make_decoder: F,
) -> io::Result<(FileType, Vec<ArchiveEntry>)>
where
    F: Fn() -> io::Result<R>,
    R: Read,
{
    let mut head = [0u8; 512];
    let n = read_fill(&mut make_decoder()?, &mut head)?;
    if looks_like_tar(&head[..n]) {
        Ok((tar_type, tar::list_reader(make_decoder()?)?))
    } else {
        Ok((single_type, vec![single_file_entry(src)]))
    }
}
