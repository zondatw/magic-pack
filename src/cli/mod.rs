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
  magic-pack -c -f seven-z -o temp/temp.7z src
  magic-pack -d -o temp/. temp/temp.7z
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

    // 7z AES-256 password. `-p <value>` uses it inline; `-p` alone
    // prompts interactively (no echo). Only present in encryption builds.
    #[cfg(feature = "encryption")]
    #[arg(short = 'p', long = "password", num_args = 0..=1, value_name = "PASSWORD")]
    pub password: Option<Option<String>>,
}

impl Args {
    pub fn new() -> Self {
        Args::parse()
    }

    /// Password to use when compressing (prompts with confirmation when
    /// `-p` is given without a value). `None` in non-encryption builds.
    #[cfg(feature = "encryption")]
    pub fn compress_password(&self) -> Option<String> {
        self.resolve_password(true)
    }

    #[cfg(not(feature = "encryption"))]
    pub fn compress_password(&self) -> Option<String> {
        None
    }

    /// Password to use when decompressing (single prompt when `-p` is
    /// given without a value). `None` in non-encryption builds.
    #[cfg(feature = "encryption")]
    pub fn decompress_password(&self) -> Option<String> {
        self.resolve_password(false)
    }

    #[cfg(not(feature = "encryption"))]
    pub fn decompress_password(&self) -> Option<String> {
        None
    }

    #[cfg(feature = "encryption")]
    fn resolve_password(&self, confirm: bool) -> Option<String> {
        match &self.password {
            None => None,
            Some(Some(inline)) => {
                eprintln!(
                    "warning: a password passed on the command line is visible in `ps` and \
                     shell history; prefer `-p` alone to be prompted"
                );
                Some(inline.clone())
            }
            Some(None) => Some(prompt_password(confirm)),
        }
    }
}

#[cfg(feature = "encryption")]
fn prompt_password(confirm: bool) -> String {
    let pw = rpassword::prompt_password("Password: ").unwrap_or_else(|err| {
        eprintln!("Error: failed to read password: {}", err);
        std::process::exit(1);
    });
    if confirm {
        let again = rpassword::prompt_password("Confirm password: ").unwrap_or_else(|err| {
            eprintln!("Error: failed to read password: {}", err);
            std::process::exit(1);
        });
        if pw != again {
            eprintln!("Error: passwords do not match");
            std::process::exit(1);
        }
    }
    pw
}
