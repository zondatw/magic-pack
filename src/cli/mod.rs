use clap::{ArgGroup, Parser};
use std::path;

use magic_pack::contents::enums;

#[derive(Parser)]
#[command(
    author,
    version,
    about,
    long_about = None,
    after_help =
"Examples:
  magic-pack -c -f zip -o temp/temp.zip src
  magic-pack -d -o temp/. temp/temp.zip
  magic-pack -c -f gz -o temp/file.txt.gz temp/file.txt
  magic-pack -d -o temp/. temp/file.txt.gz
  magic-pack -c -f bz2 -o temp/file.txt.bz2 temp/file.txt
  magic-pack -d -o temp/. temp/file.txt.bz2
  magic-pack -c -f tar -o temp/temp.tar src
  magic-pack -d -o temp/. temp/temp.tar
  magic-pack -c -f tarbz2 -o temp/temp.tar.bz2 src
  magic-pack -d -o temp/. temp/temp.tar.bz2
  magic-pack -c -f targz -o temp/temp.tar.gz src
  magic-pack -d -o temp/. temp/temp.tar.gz
  magic-pack -d -l 3 -o temp/. temp/archive.tar.gz
  magic-pack -d temp/temp.zip
"
)]
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
