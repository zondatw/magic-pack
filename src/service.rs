use std::fmt;
use std::fs;
use std::io::ErrorKind;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

use crate::contents::enums::{self, FileType};
use crate::modules;

#[derive(Debug, Clone)]
pub struct CompressRequest {
    pub file_type: FileType,
    pub input: PathBuf,
    pub output: PathBuf,
}

#[derive(Debug, Clone)]
pub struct DecompressRequest {
    pub input: PathBuf,
    pub output: PathBuf,
    pub level: i8,
}

#[derive(Debug, Clone)]
pub struct OperationResult {
    pub output_path: PathBuf,
    pub message: String,
}

#[derive(Debug)]
pub enum MagicPackError {
    Io(std::io::Error),
    UnsupportedFileType,
    InvalidInput(String),
    OperationFailed(String),
}

impl fmt::Display for MagicPackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MagicPackError::Io(err) => write!(f, "{}", err),
            MagicPackError::UnsupportedFileType => write!(f, "unsupported file type"),
            MagicPackError::InvalidInput(message) => write!(f, "{}", message),
            MagicPackError::OperationFailed(message) => write!(f, "{}", message),
        }
    }
}

impl std::error::Error for MagicPackError {}

impl From<std::io::Error> for MagicPackError {
    fn from(err: std::io::Error) -> Self {
        match err.kind() {
            ErrorKind::Unsupported => MagicPackError::UnsupportedFileType,
            _ => MagicPackError::Io(err),
        }
    }
}

pub fn supported_formats() -> Vec<&'static str> {
    vec![
        "zip", "tar", "bz2", "gz", "tar.bz2", "tar.gz", "7z", "xz", "tar.xz", "zst", "tar.zst",
        "lz4", "tar.lz4",
    ]
}

pub fn detect_file_type(path: &Path) -> Result<FileType, MagicPackError> {
    modules::get_file_type(&path.to_path_buf()).map_err(MagicPackError::from)
}

pub fn compress(req: CompressRequest) -> Result<OperationResult, MagicPackError> {
    validate_compress_request(&req)?;

    let output_path = if req.output == Path::new(".") {
        default_compress_output_path(&req.input, &req.output, req.file_type)?
    } else {
        req.output.clone()
    };

    run_operation("compress", || {
        modules::compress(req.file_type, &req.input, &output_path);
    })?;

    Ok(OperationResult {
        output_path,
        message: format!(
            "compressed as {}",
            enums::get_file_type_string(req.file_type)
        ),
    })
}

pub fn decompress(req: DecompressRequest) -> Result<OperationResult, MagicPackError> {
    validate_decompress_request(&req)?;

    if req.output != Path::new(".") {
        fs::create_dir_all(&req.output)?;
    }

    let src_filename = req.input.file_stem().ok_or_else(|| {
        MagicPackError::InvalidInput("input path must include a file name".into())
    })?;

    let mut decompress_output = req.output.join(src_filename);
    let mut decompress_input = req.input.clone();
    let filename = decompress_output.file_name().ok_or_else(|| {
        MagicPackError::InvalidInput("output path must include a file name".into())
    })?;
    let mg_filename = format!("mg_{}", filename.to_string_lossy());
    decompress_output.set_file_name(mg_filename);

    for index in 0..req.level {
        let file_type = match detect_file_type(&decompress_input) {
            Ok(file_type) => file_type,
            Err(MagicPackError::UnsupportedFileType) if index != 0 => break,
            Err(err) => return Err(err),
        };

        let current_output = decompress_output.clone();
        run_operation("decompress", || {
            modules::decompress(file_type, &decompress_input, &current_output);
        })?;
        decompress_input = current_output;
        let temp_filename = decompress_input.file_stem().ok_or_else(|| {
            MagicPackError::InvalidInput("decompressed output must include a file name".into())
        })?;
        decompress_output.set_file_name(temp_filename);
    }

    let final_filename = decompress_input
        .file_name()
        .ok_or_else(|| {
            MagicPackError::InvalidInput("decompressed output must include a file name".into())
        })?
        .to_string_lossy()
        .replace("mg_", "");
    let mut final_output = decompress_input.clone();
    final_output.set_file_name(final_filename);
    fs::rename(&decompress_input, &final_output)?;

    Ok(OperationResult {
        output_path: final_output,
        message: String::from("decompressed"),
    })
}

fn validate_compress_request(req: &CompressRequest) -> Result<(), MagicPackError> {
    if !req.input.exists() {
        return Err(MagicPackError::InvalidInput(format!(
            "input path does not exist: {}",
            req.input.display()
        )));
    }

    if req.output == Path::new(".") {
        return Ok(());
    }

    if let Some(parent) = req.output.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }

    Ok(())
}

fn validate_decompress_request(req: &DecompressRequest) -> Result<(), MagicPackError> {
    if !req.input.exists() {
        return Err(MagicPackError::InvalidInput(format!(
            "input path does not exist: {}",
            req.input.display()
        )));
    }

    if req.level <= 0 {
        return Err(MagicPackError::InvalidInput(
            "decompress level must be greater than 0".into(),
        ));
    }

    Ok(())
}

fn default_compress_output_path(
    src_path: &Path,
    dst_path: &Path,
    file_type: FileType,
) -> Result<PathBuf, MagicPackError> {
    let filename = src_path.file_stem().ok_or_else(|| {
        MagicPackError::InvalidInput("input path must include a file name".into())
    })?;
    let mut temp_output = dst_path.join(filename);
    temp_output.set_extension(enums::get_file_type_string(file_type));
    Ok(temp_output)
}

fn run_operation<F>(label: &str, operation: F) -> Result<(), MagicPackError>
where
    F: FnOnce(),
{
    catch_unwind(AssertUnwindSafe(operation)).map_err(|panic_payload| {
        let message = if let Some(message) = panic_payload.downcast_ref::<&str>() {
            (*message).to_string()
        } else if let Some(message) = panic_payload.downcast_ref::<String>() {
            message.clone()
        } else {
            format!("{} failed", label)
        };
        MagicPackError::OperationFailed(message)
    })
}
