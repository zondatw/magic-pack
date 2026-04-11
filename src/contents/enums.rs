use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
pub enum FileType {
    Zip,
    Tar,
    Bz2,
    Gz,
    Tarbz2,
    Targz,
    SevenZ,
    Xz,
    Tarxz,
    Zst,
    Tarzst,
    Lz4,
    Tarlz4,
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
    }
}
