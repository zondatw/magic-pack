use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum FileType {
    Zip,
    Tar,
    Bz2,
    Gz,
    Tarbz2,
    Targz,
    // Canonical CLI token is `seven-z` (kebab-cased variant); `7z` is
    // accepted as an alias so the CLI matches the MCP / README token.
    #[value(alias = "7z")]
    SevenZ,
    Xz,
    Tarxz,
    Zst,
    Tarzst,
    Lz4,
    Tarlz4,
    Upx,
}

impl FileType {
    /// Executable packers wrap a binary in place instead of producing a
    /// new container, so they need bespoke filename handling in the
    /// service layer (preserve the original extension on compress, strip
    /// the `.upx` infix on decompress).
    pub fn is_executable_packer(self) -> bool {
        matches!(self, FileType::Upx)
    }
}

pub fn get_file_type_string(file_type: FileType) -> &'static str {
    match file_type {
        FileType::Zip => "zip",
        FileType::Tar => "tar",
        FileType::Bz2 => "bz2",
        FileType::Gz => "gz",
        FileType::Tarbz2 => "tar.bz2",
        FileType::Targz => "tar.gz",
        FileType::SevenZ => "7z",
        FileType::Xz => "xz",
        FileType::Tarxz => "tar.xz",
        FileType::Zst => "zst",
        FileType::Tarzst => "tar.zst",
        FileType::Lz4 => "lz4",
        FileType::Tarlz4 => "tar.lz4",
        FileType::Upx => "upx",
    }
}
