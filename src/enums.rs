
use clap::{ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum FileType {
    Zip,
    Tar,
    Tarbz2,
    Targz,
}