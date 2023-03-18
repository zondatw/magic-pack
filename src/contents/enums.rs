use clap::ValueEnum;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum FileType {
    Zip,
    Tar,
    Bz2,
    Gz,
    Tarbz2,
    Targz,
}

pub fn get_file_type_string(file_type: FileType) -> &'static str {
    match file_type {
        FileType::Zip => "zip",
        FileType::Tar => "tar",
        FileType::Bz2 => "bz2",
        FileType::Gz => "gz",
        FileType::Tarbz2 => "tar.bz2",
        FileType::Targz => "tar.gz",
    }
}
