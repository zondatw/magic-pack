use std::fmt;
use std::fs;
use std::io::ErrorKind;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicU64;
use std::sync::Arc;

use crate::contents::enums::{self, FileType};
use crate::modules;

#[derive(Clone)]
pub struct CompressRequest {
    pub file_type: FileType,
    pub input: PathBuf,
    pub output: PathBuf,
    /// 7z AES-256 password. `None` for unencrypted. Redacted in Debug.
    pub password: Option<String>,
}

#[derive(Clone)]
pub struct DecompressRequest {
    pub input: PathBuf,
    pub output: PathBuf,
    pub level: i8,
    /// 7z AES-256 password. `None` for unencrypted. Redacted in Debug.
    pub password: Option<String>,
}

// Manual Debug impls keep the password out of any `{:?}` output so it
// can never reach a log line or error message via formatting.
impl fmt::Debug for CompressRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CompressRequest")
            .field("file_type", &self.file_type)
            .field("input", &self.input)
            .field("output", &self.output)
            .field("password", &self.password.as_ref().map(|_| "<redacted>"))
            .finish()
    }
}

impl fmt::Debug for DecompressRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DecompressRequest")
            .field("input", &self.input)
            .field("output", &self.output)
            .field("level", &self.level)
            .field("password", &self.password.as_ref().map(|_| "<redacted>"))
            .finish()
    }
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
        "lz4", "tar.lz4", "upx",
    ]
}

pub fn detect_file_type(path: &Path) -> Result<FileType, MagicPackError> {
    modules::get_file_type(&path.to_path_buf()).map_err(MagicPackError::from)
}

pub fn compress(req: CompressRequest) -> Result<OperationResult, MagicPackError> {
    compress_with_progress(req, None)
}

/// Like [`compress`], but increments `progress` (a shared byte counter)
/// as input is consumed so the CLI can draw a progress bar.
pub fn compress_with_progress(
    req: CompressRequest,
    progress: Option<Arc<AtomicU64>>,
) -> Result<OperationResult, MagicPackError> {
    validate_compress_request(&req)?;

    let output_path = if req.output == Path::new(".") {
        default_compress_output_path(&req.input, &req.output, req.file_type)?
    } else {
        req.output.clone()
    };

    let password = req.password.clone();
    run_operation("compress", || {
        modules::compress_with_password(
            req.file_type,
            &req.input,
            &output_path,
            password.as_deref(),
            progress.as_deref(),
        );
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
    decompress_with_progress(req, None)
}

/// Like [`decompress`], but increments `progress` (a shared byte
/// counter) as the archive is consumed so the CLI can draw a bar. The
/// counter tracks the outermost layer for nested archives.
pub fn decompress_with_progress(
    req: DecompressRequest,
    progress: Option<Arc<AtomicU64>>,
) -> Result<OperationResult, MagicPackError> {
    validate_decompress_request(&req)?;

    if req.output != Path::new(".") {
        fs::create_dir_all(&req.output)?;
    }

    // Executable packers (UPX) work file-to-file in a single step. The
    // generic level-loop below assumes archive containers and uses
    // file_stem() to strip extensions, which mangles binary filenames.
    // Detect once up front and take a dedicated path for packers.
    let initial_type = detect_file_type(&req.input)?;
    if initial_type.is_executable_packer() {
        return decompress_executable_packer(&req, initial_type);
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
        let password = req.password.clone();
        let progress_ref = progress.as_deref();
        run_operation("decompress", || {
            modules::decompress_with_password(
                file_type,
                &decompress_input,
                &current_output,
                password.as_deref(),
                progress_ref,
            );
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
    if file_type.is_executable_packer() {
        return default_packer_compress_output_path(src_path, dst_path, file_type);
    }
    let filename = src_path.file_stem().ok_or_else(|| {
        MagicPackError::InvalidInput("input path must include a file name".into())
    })?;
    let mut temp_output = dst_path.join(filename);
    temp_output.set_extension(enums::get_file_type_string(file_type));
    Ok(temp_output)
}

/// `foo.exe` → `foo.upx.exe`. `foo` (no extension) → `foo.upx`. Keeps
/// the original extension so the produced binary stays runnable on the
/// target platform.
fn default_packer_compress_output_path(
    src_path: &Path,
    dst_path: &Path,
    file_type: FileType,
) -> Result<PathBuf, MagicPackError> {
    let filename = src_path.file_name().ok_or_else(|| {
        MagicPackError::InvalidInput("input path must include a file name".into())
    })?;
    let format_str = enums::get_file_type_string(file_type);
    let name_path = Path::new(filename);
    let new_name = match (name_path.file_stem(), name_path.extension()) {
        (Some(stem), Some(ext)) => format!(
            "{}.{}.{}",
            stem.to_string_lossy(),
            format_str,
            ext.to_string_lossy()
        ),
        _ => format!("{}.{}", filename.to_string_lossy(), format_str),
    };
    Ok(dst_path.join(new_name))
}

/// `foo.upx.exe` → `foo.exe`. `foo.exe` (no `.upx` infix) →
/// `foo.unpacked.exe`. `foo` → `foo.unpacked`.
fn derive_packer_decompressed_name(input: &Path) -> Result<String, MagicPackError> {
    let filename = input
        .file_name()
        .ok_or_else(|| MagicPackError::InvalidInput("input path must include a file name".into()))?
        .to_string_lossy()
        .into_owned();

    if let Some(idx) = filename.find(".upx.") {
        let mut result = String::with_capacity(filename.len() - 4);
        result.push_str(&filename[..idx]);
        result.push_str(&filename[idx + 4..]);
        return Ok(result);
    }
    if let Some(stripped) = filename.strip_suffix(".upx") {
        return Ok(stripped.to_string());
    }

    let path = Path::new(&filename);
    match (path.file_stem(), path.extension()) {
        (Some(stem), Some(ext)) => Ok(format!(
            "{}.unpacked.{}",
            stem.to_string_lossy(),
            ext.to_string_lossy()
        )),
        _ => Ok(format!("{}.unpacked", filename)),
    }
}

fn decompress_executable_packer(
    req: &DecompressRequest,
    file_type: FileType,
) -> Result<OperationResult, MagicPackError> {
    let dst_filename = derive_packer_decompressed_name(&req.input)?;
    let dst_path = req.output.join(&dst_filename);
    let dst_clone = dst_path.clone();
    let input = req.input.clone();
    // UPX takes no password; the plain (no-encryption) entry point.
    run_operation("decompress", move || {
        modules::decompress(file_type, &input, &dst_clone);
    })?;
    Ok(OperationResult {
        output_path: dst_path,
        message: String::from("decompressed"),
    })
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

#[cfg(test)]
mod packer_filename_tests {
    use super::*;

    #[test]
    fn compress_default_preserves_extension() {
        let cwd = Path::new(".");
        let cases = [
            ("foo.exe", "./foo.upx.exe"),
            ("foo", "./foo.upx"),
            ("a.b.c.exe", "./a.b.c.upx.exe"),
        ];
        for (input, expected) in cases {
            let got =
                default_packer_compress_output_path(Path::new(input), cwd, FileType::Upx).unwrap();
            assert_eq!(got, PathBuf::from(expected), "input={}", input);
        }
    }

    #[test]
    fn decompress_strips_upx_infix_or_adds_unpacked() {
        let cases = [
            ("foo.upx.exe", "foo.exe"),
            ("foo.upx", "foo"),
            ("a.b.upx.c", "a.b.c"),
            ("foo.exe", "foo.unpacked.exe"),
            ("foo", "foo.unpacked"),
            ("a.b.c.exe", "a.b.c.unpacked.exe"),
        ];
        for (input, expected) in cases {
            let got = derive_packer_decompressed_name(Path::new(input)).unwrap();
            assert_eq!(got, expected, "input={}", input);
        }
    }

    #[test]
    fn is_executable_packer_predicate_table() {
        // Every variant must be classified explicitly — when a new
        // variant is added the compiler-driven match prevents drift.
        for (variant, expected) in [
            (FileType::Zip, false),
            (FileType::Tar, false),
            (FileType::Bz2, false),
            (FileType::Gz, false),
            (FileType::Tarbz2, false),
            (FileType::Targz, false),
            (FileType::SevenZ, false),
            (FileType::Xz, false),
            (FileType::Tarxz, false),
            (FileType::Zst, false),
            (FileType::Tarzst, false),
            (FileType::Lz4, false),
            (FileType::Tarlz4, false),
            (FileType::Upx, true),
        ] {
            assert_eq!(
                variant.is_executable_packer(),
                expected,
                "variant={:?}",
                variant
            );
        }
    }
}
