use std::env;
use std::io::{self, BufRead, Write};
use std::path::{Component, Path, PathBuf};

use magic_pack::contents::enums::FileType;
use magic_pack::service::{self, CompressRequest, DecompressRequest};
use serde_json::{json, Map, Value};

const JSONRPC_VERSION: &str = "2.0";
const LATEST_PROTOCOL_VERSION: &str = "2025-11-25";
const SUPPORTED_PROTOCOL_VERSIONS: &[&str] =
    &["2024-11-05", "2025-03-26", "2025-06-18", "2025-11-25"];

fn main() -> io::Result<()> {
    let stdin = io::stdin();
    let stdout = io::stdout();
    let mut writer = stdout.lock();
    let mut state = ServerState::new()?;

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }

        let response = match serde_json::from_str::<Value>(&line) {
            Ok(value) => handle_incoming(value, &mut state),
            Err(err) => Some(error_response(
                Value::Null,
                -32700,
                format!("parse error: {}", err),
            )),
        };

        if let Some(response) = response {
            serde_json::to_writer(&mut writer, &response)?;
            writer.write_all(b"\n")?;
            writer.flush()?;
        }
    }

    Ok(())
}

struct ServerState {
    initialize_seen: bool,
    initialized: bool,
    allowed_root: Option<PathBuf>,
    cwd: PathBuf,
}

impl ServerState {
    fn new() -> io::Result<Self> {
        let cwd = env::current_dir()?;
        let allowed_root = env::var_os("MAGIC_PACK_MCP_ALLOWED_ROOT")
            .map(PathBuf::from)
            .map(|path| absolutize_path(&cwd, &path));
        Ok(Self {
            initialize_seen: false,
            initialized: false,
            allowed_root,
            cwd,
        })
    }
}

fn handle_incoming(value: Value, state: &mut ServerState) -> Option<Value> {
    match value {
        Value::Array(items) => handle_batch(items, state),
        Value::Object(object) => handle_message(&Value::Object(object), state),
        _ => Some(error_response(
            Value::Null,
            -32600,
            String::from("invalid request"),
        )),
    }
}

fn handle_batch(items: Vec<Value>, state: &mut ServerState) -> Option<Value> {
    if items.is_empty() {
        return Some(error_response(
            Value::Null,
            -32600,
            String::from("invalid request"),
        ));
    }

    let mut responses = Vec::new();
    for item in items {
        if let Some(response) = handle_message(&item, state) {
            responses.push(response);
        }
    }

    if responses.is_empty() {
        None
    } else {
        Some(Value::Array(responses))
    }
}

fn handle_message(message: &Value, state: &mut ServerState) -> Option<Value> {
    let object = match message.as_object() {
        Some(object) => object,
        None => {
            return Some(error_response(
                Value::Null,
                -32600,
                String::from("invalid request"),
            ))
        }
    };

    if object.get("jsonrpc").and_then(Value::as_str) != Some(JSONRPC_VERSION) {
        return Some(error_response(
            object.get("id").cloned().unwrap_or(Value::Null),
            -32600,
            String::from("invalid request"),
        ));
    }

    let method = match object.get("method").and_then(Value::as_str) {
        Some(method) => method,
        None => {
            return Some(error_response(
                object.get("id").cloned().unwrap_or(Value::Null),
                -32600,
                String::from("invalid request"),
            ))
        }
    };

    let id = object.get("id").cloned();
    let params = object.get("params");

    match id {
        Some(id) => Some(handle_request(id, method, params, state)),
        None => handle_notification(method, params, state),
    }
}

fn handle_request(
    id: Value,
    method: &str,
    params: Option<&Value>,
    state: &mut ServerState,
) -> Value {
    match method {
        "initialize" => initialize(id, params, state),
        "ping" => success_response(id, json!({})),
        "tools/list" => {
            if let Err(err) = ensure_initialized(state, &id) {
                return err;
            }
            success_response(id, json!({ "tools": tool_definitions() }))
        }
        "tools/call" => {
            if let Err(err) = ensure_initialized(state, &id) {
                return err;
            }
            call_tool(id, params, state)
        }
        _ => error_response(id, -32601, format!("method not found: {}", method)),
    }
}

fn handle_notification(
    method: &str,
    _params: Option<&Value>,
    state: &mut ServerState,
) -> Option<Value> {
    match method {
        "notifications/initialized" => {
            if state.initialize_seen {
                state.initialized = true;
            }
            None
        }
        _ => None,
    }
}

fn initialize(id: Value, params: Option<&Value>, state: &mut ServerState) -> Value {
    let params = match params.and_then(Value::as_object) {
        Some(params) => params,
        None => {
            return error_response(
                id,
                -32602,
                String::from("initialize params must be an object"),
            );
        }
    };

    let requested_version = match params.get("protocolVersion").and_then(Value::as_str) {
        Some(version) => version,
        None => {
            return error_response(
                id,
                -32602,
                String::from("initialize.params.protocolVersion must be a string"),
            );
        }
    };

    let protocol_version = if SUPPORTED_PROTOCOL_VERSIONS.contains(&requested_version) {
        requested_version
    } else {
        LATEST_PROTOCOL_VERSION
    };

    state.initialize_seen = true;
    state.initialized = false;

    success_response(
        id,
        json!({
            "protocolVersion": protocol_version,
            "capabilities": {
                "tools": {
                    "listChanged": false
                }
            },
            "serverInfo": {
                "name": "magic-pack",
                "version": env!("CARGO_PKG_VERSION")
            }
        }),
    )
}

fn call_tool(id: Value, params: Option<&Value>, state: &ServerState) -> Value {
    let params = match params.and_then(Value::as_object) {
        Some(params) => params,
        None => return error_response(id, -32602, String::from("tool params must be an object")),
    };

    let name = match params.get("name").and_then(Value::as_str) {
        Some(name) => name,
        None => return error_response(id, -32602, String::from("tool name must be a string")),
    };

    let empty_arguments = Map::new();
    let arguments = match params.get("arguments") {
        Some(Value::Object(arguments)) => arguments,
        Some(Value::Null) | None => &empty_arguments,
        Some(_) => {
            return error_response(id, -32602, String::from("tool arguments must be an object"))
        }
    };

    match dispatch_tool(name, arguments, state) {
        Ok(result) => success_response(
            id,
            json!({
                "content": [
                    {
                        "type": "text",
                        "text": result
                    }
                ],
                "isError": false
            }),
        ),
        Err(ToolCallError::Protocol { code, message }) => error_response(id, code, message),
        Err(ToolCallError::Tool(message)) => success_response(
            id,
            json!({
                "content": [
                    {
                        "type": "text",
                        "text": message
                    }
                ],
                "isError": true
            }),
        ),
    }
}

fn dispatch_tool(
    name: &str,
    arguments: &Map<String, Value>,
    state: &ServerState,
) -> Result<String, ToolCallError> {
    match name {
        "compress" => {
            let input = required_path(arguments, "input_path", state)?;
            let output = optional_path(arguments, "output_path", state)?
                .unwrap_or_else(|| PathBuf::from("."));
            let file_type = required_file_type(arguments, "file_type")?;
            ensure_allowed_path(&input, state)?;
            ensure_allowed_path(&output, state)?;

            let result = service::compress(CompressRequest {
                file_type,
                input,
                output,
            })
            .map_err(|err| ToolCallError::Tool(err.to_string()))?;

            Ok(json!({
                "ok": true,
                "message": result.message,
                "output_path": result.output_path
            })
            .to_string())
        }
        "decompress" => {
            let input = required_path(arguments, "input_path", state)?;
            let output = optional_path(arguments, "output_path", state)?
                .unwrap_or_else(|| PathBuf::from("."));
            let level = optional_i64(arguments, "level")?.unwrap_or(5);
            let level = i8::try_from(level)
                .map_err(|_| invalid_params("level must fit in an 8-bit signed integer"))?;
            ensure_allowed_path(&input, state)?;
            ensure_allowed_path(&output, state)?;

            let result = service::decompress(DecompressRequest {
                input,
                output,
                level,
            })
            .map_err(|err| ToolCallError::Tool(err.to_string()))?;

            Ok(json!({
                "ok": true,
                "message": result.message,
                "output_path": result.output_path
            })
            .to_string())
        }
        "detect_file_type" => {
            let input = required_path(arguments, "input_path", state)?;
            ensure_allowed_path(&input, state)?;
            let file_type = service::detect_file_type(&input)
                .map_err(|err| ToolCallError::Tool(err.to_string()))?;

            Ok(json!({
                "ok": true,
                "file_type": file_type_name(file_type)
            })
            .to_string())
        }
        "supported_formats" => Ok(json!({
            "ok": true,
            "formats": service::supported_formats()
        })
        .to_string()),
        _ => Err(ToolCallError::Protocol {
            code: -32602,
            message: format!("unknown tool: {}", name),
        }),
    }
}

fn required_path(
    arguments: &Map<String, Value>,
    key: &str,
    state: &ServerState,
) -> Result<PathBuf, ToolCallError> {
    let raw = arguments
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_params(format!("{} must be a string", key)))?;
    Ok(absolutize_path(&state.cwd, &PathBuf::from(raw)))
}

fn optional_path(
    arguments: &Map<String, Value>,
    key: &str,
    state: &ServerState,
) -> Result<Option<PathBuf>, ToolCallError> {
    match arguments.get(key) {
        Some(Value::String(raw)) => Ok(Some(absolutize_path(&state.cwd, &PathBuf::from(raw)))),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(invalid_params(format!("{} must be a string", key))),
    }
}

fn optional_i64(arguments: &Map<String, Value>, key: &str) -> Result<Option<i64>, ToolCallError> {
    match arguments.get(key) {
        Some(Value::Number(value)) => value
            .as_i64()
            .ok_or_else(|| invalid_params(format!("{} must be an integer", key)))
            .map(Some),
        Some(Value::Null) | None => Ok(None),
        Some(_) => Err(invalid_params(format!("{} must be an integer", key))),
    }
}

fn required_file_type(
    arguments: &Map<String, Value>,
    key: &str,
) -> Result<FileType, ToolCallError> {
    let raw = arguments
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| invalid_params(format!("{} must be a string", key)))?;
    match raw {
        "zip" => Ok(FileType::Zip),
        "tar" => Ok(FileType::Tar),
        "bz2" => Ok(FileType::Bz2),
        "gz" => Ok(FileType::Gz),
        "tarbz2" | "tar.bz2" => Ok(FileType::Tarbz2),
        "targz" | "tar.gz" => Ok(FileType::Targz),
        "7z" => Ok(FileType::SevenZ),
        "xz" => Ok(FileType::Xz),
        "tarxz" | "tar.xz" => Ok(FileType::Tarxz),
        _ => Err(invalid_params(
            "file_type must be one of zip, tar, bz2, gz, tarbz2, targz, tar.bz2, tar.gz, 7z, xz, tarxz, tar.xz",
        )),
    }
}

fn ensure_allowed_path(path: &Path, state: &ServerState) -> Result<(), ToolCallError> {
    if let Some(root) = &state.allowed_root {
        if !path.starts_with(root) {
            return Err(ToolCallError::Tool(format!(
                "path is outside MAGIC_PACK_MCP_ALLOWED_ROOT: {}",
                path.display()
            )));
        }
    }
    Ok(())
}

fn tool_definitions() -> Vec<Value> {
    vec![
        json!({
            "name": "compress",
            "description": "Compress a file or directory into a supported archive format.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "input_path": {
                        "type": "string",
                        "description": "File or directory to compress."
                    },
                    "output_path": {
                        "type": "string",
                        "description": "Destination archive path. Defaults to the current directory."
                    },
                    "file_type": {
                        "type": "string",
                        "description": "Archive format to create.",
                        "enum": ["zip", "tar", "bz2", "gz", "tarbz2", "targz", "tar.bz2", "tar.gz", "7z", "xz", "tarxz", "tar.xz"]
                    }
                },
                "required": ["input_path", "file_type"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "decompress",
            "description": "Decompress an archive, optionally unpacking nested layers.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "input_path": {
                        "type": "string",
                        "description": "Archive file to decompress."
                    },
                    "output_path": {
                        "type": "string",
                        "description": "Destination directory. Defaults to the current directory."
                    },
                    "level": {
                        "type": "integer",
                        "description": "Maximum nested archive layers to unpack.",
                        "default": 5,
                        "minimum": 1
                    }
                },
                "required": ["input_path"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "detect_file_type",
            "description": "Detect the archive type of a file using magic bytes.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "input_path": {
                        "type": "string",
                        "description": "Archive file to inspect."
                    }
                },
                "required": ["input_path"],
                "additionalProperties": false
            }
        }),
        json!({
            "name": "supported_formats",
            "description": "List the archive formats supported by magic-pack.",
            "inputSchema": {
                "type": "object",
                "properties": {},
                "additionalProperties": false
            }
        }),
    ]
}

fn file_type_name(file_type: FileType) -> &'static str {
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
    }
}

fn ensure_initialized(state: &ServerState, id: &Value) -> Result<(), Value> {
    if !state.initialize_seen {
        return Err(error_response(
            id.clone(),
            -32002,
            String::from("server not initialized"),
        ));
    }
    if !state.initialized {
        return Err(error_response(
            id.clone(),
            -32002,
            String::from("client must send notifications/initialized before using tools"),
        ));
    }
    Ok(())
}

fn success_response(id: Value, result: Value) -> Value {
    json!({
        "jsonrpc": JSONRPC_VERSION,
        "id": id,
        "result": result
    })
}

fn error_response(id: Value, code: i64, message: String) -> Value {
    json!({
        "jsonrpc": JSONRPC_VERSION,
        "id": id,
        "error": {
            "code": code,
            "message": message
        }
    })
}

fn invalid_params(message: impl Into<String>) -> ToolCallError {
    ToolCallError::Protocol {
        code: -32602,
        message: message.into(),
    }
}

fn absolutize_path(base: &Path, path: &Path) -> PathBuf {
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        base.join(path)
    };
    normalize_path(&joined)
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            Component::RootDir | Component::Prefix(_) | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }

    if normalized.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        normalized
    }
}

enum ToolCallError {
    Protocol { code: i64, message: String },
    Tool(String),
}
