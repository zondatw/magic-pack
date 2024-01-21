use clap::{ArgGroup, Parser};
use std::path;

use magic_pack::contents::enums;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(group(
    ArgGroup::new("functional")
        .required(true)
        .args(["compress", "decompress"]),
))]

pub struct Args {
    // Compress flag
    #[arg(short, long, requires = "file_type")]
    pub compress: bool,

    // file type
    #[arg(short, value_enum)]
    pub file_type: Option<enums::FileType>,

    // Decompress flag
    #[arg(short, long)]
    pub decompress: bool,

    // decompress level
    #[arg(short, long, default_value = "5")]
    pub level: i8,

    // file / directory input path
    pub input: path::PathBuf,

    // file / directory output path
    #[arg(short, default_value = ".")]
    pub output: path::PathBuf,
}

impl Args {
    pub fn new() -> Self {
        Args::parse()
    }
}
