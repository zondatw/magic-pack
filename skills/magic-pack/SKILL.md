---
name: magic-pack
description: |
  Compresses and decompresses files across 13 archive formats: zip, tar,
  bz2, gz, tar.bz2, tar.gz, 7z, xz, tar.xz, zst, tar.zst, lz4, tar.lz4.
  Auto-detects archive type via magic bytes on decompress. Activates
  when the user asks to compress, decompress, extract, unpack, unzip,
  archive, package, or bundle a directory; identify "what kind of
  archive is X"; or work with tar.gz, gzip, bzip2, 7-zip, xz, zstd,
  lz4, or nested archives. Provides an MCP server (`magic-pack-mcp`,
  preferred for agents — structured JSON output) and a CLI binary
  (`magic-pack`).
when_to_use: |
  Trigger on phrases like: zip / unzip, tar / untar, gzip / gunzip,
  compress, decompress, pack, unpack, extract, archive a folder,
  bundle this directory, "what is foo.bin", magic-byte detection,
  nested archive, "tar.gz inside a tar", multi-layer extract, list
  the supported archive formats.
allowed-tools: ["Bash", "Read"]
---

# magic-pack — multi-format archive compress / decompress

Source: <https://github.com/zondatw/magic-pack> · Crates: `magic-pack`
(CLI + library) and the optional `magic-pack-mcp` binary (behind the
`mcp` feature).

## TL;DR

Pick a format, hand it a path, get back a file. **Prefer the MCP
server** (returns JSON) when it's wired up; fall back to the CLI via
Bash when it isn't. Decompress auto-detects the format via magic
bytes — compress doesn't, so you must name the format up front.

```jsonc
// Minimal MCP call — every tool returns a one-line JSON document.
{ "name": "compress", "arguments": {
    "input_path": "/abs/path/to/dir",
    "output_path": "/abs/path/to/dir.tar.zst",
    "file_type": "tar.zst"
}}
// → { "ok": true, "message": "...", "output_path": "/abs/path/to/dir.tar.zst" }
```

## When to use

- **Pack a directory or file** for delivery / backup / upload.
- **Extract any archive**, including unknown extensions — call
  `detect_file_type` first if the extension is missing or wrong.
- **Unpack nested archives** in one shot (an `outer.tar.gz` that
  contains an `inner.zip`, etc.) via the `level` parameter.
- **Identify a mystery file** — magic-byte detection without unpacking.

## When NOT to use

- **Streaming / pipe-mode compression** — magic-pack works on file
  paths only. Use `gzip`, `bzip2`, `zstd`, etc., directly when piping.
- **Encryption at rest** — no password-protected zip, no age/gpg
  integration. Use `age` or `gpg` for that, then archive the
  ciphertext.
- **Incremental sync** — magic-pack creates / extracts archives
  whole. For "ship only what changed," use `rsync` or `restic`.
- **Listing or partial extract** — magic-pack unpacks the whole
  archive. Use `tar -t` / `unzip -l` to peek without extracting.
- **Splitting huge archives across volumes** — out of scope.

## Installing the binaries

The skill is just markdown — calling out to anything requires the
binaries on PATH. Detect first; install only if missing.

```bash
# 1. Probe what's installed.
command -v magic-pack-mcp >/dev/null 2>&1 && echo "MCP server present"
command -v magic-pack     >/dev/null 2>&1 && echo "CLI present"
```

If either is missing, surface this install command to the user (don't
just silently run it — installing into the user's cargo prefix is a
side effect):

```bash
# Both binaries via the latest crates.io release. The `mcp` feature
# is what gives you `magic-pack-mcp`; without it you get the CLI only.
cargo install magic-pack --features mcp
```

If `cargo` itself is missing, point the user at <https://rustup.rs>,
or at the `.deb` (Ubuntu / Debian, from
<https://github.com/zondatw/magic-pack/releases/latest>) or AUR
package (`yay -S magic-pack` / `magic-pack-bin`). Don't try to install
`cargo` automatically.

## MCP wiring

For Claude Desktop / Claude Code, add to the relevant config:

```jsonc
{
  "mcpServers": {
    "magic-pack": {
      "command": "magic-pack-mcp",
      "env": {
        "MAGIC_PACK_MCP_ALLOWED_ROOT": "/absolute/path/you/want/to/allow"
      }
    }
  }
}
```

For OpenAI Codex (`~/.codex/config.toml`):

```toml
[mcp_servers.magic-pack]
command = "magic-pack-mcp"

[mcp_servers.magic-pack.env]
MAGIC_PACK_MCP_ALLOWED_ROOT = "/absolute/path/you/want/to/allow"
```

`MAGIC_PACK_MCP_ALLOWED_ROOT` is **required in practice**: the server
refuses any input or output path that isn't a descendant of that
directory. Pick the narrowest root that covers the user's working
files. Restart the client after editing the config.

## Two delivery modes

| | When to pick | How to call |
|---|---|---|
| **MCP** (preferred) | `magic-pack-mcp` is wired into the running Claude / Codex session — i.e., MCP tools like `compress`, `decompress` appear in the tool list | Call the tool directly. Returns JSON. |
| **CLI fallback** | MCP not wired, but `magic-pack` is on PATH | `Bash`: `magic-pack -c -f <fmt> -o <out> <input>` for compress, `magic-pack -d -o <out> <input>` for decompress. Parse stdout / exit code. |

If neither is available, surface the install command above and stop.

## Tool / format cheat sheet

| MCP tool | CLI flag(s) | Required args | Optional args | Returns |
|---|---|---|---|---|
| `compress` | `-c -f <fmt> -o <out> <input>` | `input_path`, `file_type` | `output_path` (default `.`) | `{ ok, message, output_path }` |
| `decompress` | `-d -o <out> <input>` (`-l N` for nested) | `input_path` | `output_path` (default `.`), `level` (default 5) | `{ ok, message, output_path }` |
| `detect_file_type` | (no CLI equivalent — read magic bytes manually) | `input_path` | — | `{ ok, file_type }` |
| `supported_formats` | `magic-pack --help` | — | — | `{ ok, formats: [...] }` |

Format reference (canonical strings — both `tarbz2` and `tar.bz2`
spellings are accepted by the MCP `file_type` arg):

| Format | Single-file? | Container? | Use case |
|---|---|---|---|
| `zip` | — | yes | cross-platform delivery, Windows-friendly |
| `tar` | — | yes | uncompressed bundle (chain with another format) |
| `7z` | — | yes | high compression, multi-file native |
| `gz` | yes | — | classic single-file, ubiquitous |
| `bz2` | yes | — | better ratio than gz, slow |
| `xz` | yes | — | very high ratio, slow |
| `zst` | yes | — | modern default — fast + good ratio |
| `lz4` | yes | — | fastest, lowest ratio — pick for hot paths |
| `tar.gz` (`targz`) | — | yes | most portable Unix bundle |
| `tar.bz2` (`tarbz2`) | — | yes | legacy compatibility |
| `tar.xz` (`tarxz`) | — | yes | smallest archive of a directory |
| `tar.zst` (`tarzst`) | — | yes | recommended modern default for directories |
| `tar.lz4` (`tarlz4`) | — | yes | speed-first directory archive |

A "single-file" format wraps **exactly one file**. To compress a
directory, pick the `tar.*` variant — see "Common gotchas" below.

## Output interpretation

### MCP JSON

Success:
```jsonc
{ "ok": true,
  "message": "compressed src to /abs/dir.tar.zst",
  "output_path": "/abs/dir.tar.zst" }
```

Tool-level failure (`isError: true` on the wrapper, JSON-encoded text
inside `content[0].text`): the message is the underlying error
string. Common patterns to recognize:

- `"path is outside MAGIC_PACK_MCP_ALLOWED_ROOT: ..."` — input or
  output is outside the configured allow-list. Either widen the
  config (restart required) or move the file inside the root.
- `"Not a directory (os error 20)"` — happens when feeding a directory
  to a single-file format like `gz`. Use `tar.gz` / `tar.zst` instead.
- `"unsupported file type"` / `"unknown magic"` — `detect_file_type`
  on something it doesn't recognize, or `decompress` on a non-archive.
- `"file_type must be one of zip, tar, bz2, ..."` — typo on the
  format string; see cheat sheet for valid spellings.

Protocol-level failures use JSON-RPC error codes (e.g., `-32602`
"invalid params"); the agent rarely needs to parse those — re-read
the tool's input schema.

### CLI text

`magic-pack` is mostly silent on success and exits 0. Failures print
to stderr and exit non-zero. The `-V` / `--version` flag and
`--help` are the only stable text outputs to scrape.

## Diagnostic decision tree

1. **"Pack this directory."** Pick a `tar.*` format.
   - Default: `tar.zst` (fast + good ratio).
   - Need cross-platform / Windows users to open it: `zip`.
   - Need maximum compression and don't care about CPU: `tar.xz`.
   - Hot-path / huge data / speed-first: `tar.lz4`.
2. **"Pack this single file."** Single-file formats are fine
   (`gz`, `bz2`, `xz`, `zst`, `lz4`). `zst` is the modern default.
3. **"Extract `foo.zip` / `foo.tar.gz`."** Call `decompress` with
   the path — auto-detect handles it. No `file_type` needed.
4. **"Extract `foo.bin` / unknown extension."** Call
   `detect_file_type` first. If recognized, call `decompress`. If
   not, the file isn't a magic-pack-supported archive.
5. **"What kind of archive is this?"** `detect_file_type`.
6. **"Unpack the nested layers."** `decompress` with
   `level >= 2` (default is 5, so usually you can just call it
   without `level` and it does the right thing for up to 5 layers).
7. **"What formats are supported?"** `supported_formats`.

For deeper recipes covering specific scenarios, see `recipes.md` in
this skill directory.

## Common gotchas

- **Single-file formats reject directories.** `compress(input=./mydir,
  file_type=gz)` will fail with `"Not a directory (os error 20)"`.
  Use `tar.gz` / `tar.zst` / etc. for directories. The error message
  is correct but doesn't always make the cause obvious to the user
  — translate it ("you asked for `gz`, but `gz` only wraps a single
  file; use `tar.gz` for a directory").
- **`MAGIC_PACK_MCP_ALLOWED_ROOT` confines paths.** Both `input_path`
  and `output_path` must live under the configured root. Relative
  paths are resolved against the server's working directory before
  the check; pass absolute paths to avoid surprises.
- **Default `output_path` is `.`** (the server's cwd, not the user's
  shell cwd). For compress this means an archive of `dir` lands as
  `./` — almost always wrong. Spell out the destination explicitly.
- **`output_path` for compress is the *archive file*, not a
  directory.** `compress(input=./dir, output=./out/, file_type=zip)`
  may misbehave on some platforms; use
  `output_path=./out/dir.zip`.
- **`level` is unpack depth, not compression level.** Default 5,
  range 1–127. There is no per-format compression-level knob today.
- **No overwrite protection.** Compressing to an existing archive
  path overwrites it; decompressing into a populated directory may
  clobber files. Stage to a fresh path when in doubt.
- **Codex vs Claude config.** Codex reads
  `~/.codex/config.toml` (TOML). Claude Desktop / Claude Code read
  JSON. Same binary, same env var — only the file format differs.

## Limitations

- No encryption (no zip passwords, no age / gpg integration).
- No streaming / pipe mode — paths only.
- No archive listing or partial extract — extracts the whole thing.
- No update-in-place (creating an archive overwrites; no incremental
  add / replace).
- No per-format compression-level knob.

## Further reading

- `recipes.md` (this skill dir) — five worked walkthroughs.
- Repo README — install, build, full CLI reference, MCP / Codex
  config snippets: <https://github.com/zondatw/magic-pack>
- crates.io: <https://crates.io/crates/magic-pack>
