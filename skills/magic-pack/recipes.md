# magic-pack — diagnostic recipes

Companion to `SKILL.md`. Loaded only when the agent decides a recipe
walkthrough is helpful for the current user query. Each recipe is
self-contained: trigger → strategy → MCP example → read-out.

All examples assume the MCP server is wired up and
`MAGIC_PACK_MCP_ALLOWED_ROOT` covers every path used. For CLI
fallback the mapping is:

- `compress`        →  `magic-pack -c -f <fmt> -o <out> <input>`
- `decompress`      →  `magic-pack -d [-l N] -o <out> <input>`
- `detect_file_type` → no direct CLI; use `file <path>` or open the
  archive with the matching tool.
- `supported_formats` → `magic-pack --help` (the `-f` enum lists them).

## 1. Pack a directory — picking a format

**Trigger**: "Zip up this folder", "Bundle `./dist` for upload", "Make
me an archive of `~/data`".

**Strategy**: directories require a *container* format. The right
pick depends on who's opening the archive and what they care about.

| Audience / constraint | Pick |
|---|---|
| Cross-platform, Windows users will open it | `zip` |
| Modern Unix audience, want fast + small | `tar.zst` (recommended default) |
| Legacy Unix tooling, broadest compatibility | `tar.gz` |
| Smallest possible archive, CPU is free | `tar.xz` |
| Speed-first (CI, hot path, large logs) | `tar.lz4` |

```jsonc
// Modern default — fast + good ratio.
{ "name": "compress", "arguments": {
    "input_path":  "/abs/path/to/dist",
    "output_path": "/abs/path/to/dist.tar.zst",
    "file_type":   "tar.zst"
}}

// Cross-platform delivery.
{ "name": "compress", "arguments": {
    "input_path":  "/abs/path/to/dist",
    "output_path": "/abs/path/to/dist.zip",
    "file_type":   "zip"
}}
```

**Read-out**: success returns `output_path` — confirm the file
exists and is non-empty. If `ok: false` with `"Not a directory"`,
the user picked a single-file format by accident; switch to the
matching `tar.*` variant.

## 2. Extract an unknown archive

**Trigger**: "Unpack `./payload.bin`", "Someone sent me `data.dat`,
what's in it?"

**Strategy**: extension-free or wrong-extension files won't reveal
their format. Detect first, then decompress if it's something
magic-pack supports.

```jsonc
{ "name": "detect_file_type", "arguments": {
    "input_path": "/abs/path/to/payload.bin"
}}
// → { "ok": true, "file_type": "tar.gz" }

// Got a result — now extract. No `file_type` needed: decompress
// auto-detects via magic bytes.
{ "name": "decompress", "arguments": {
    "input_path":  "/abs/path/to/payload.bin",
    "output_path": "/abs/path/to/extracted/"
}}
```

**Read-out**: if `detect_file_type` returns `"unsupported file
type"` or `"unknown magic"`, the file isn't a magic-pack-supported
archive. Tell the user, suggest `file <path>` to identify it, and
stop — don't guess a format and call `decompress` blindly.

## 3. Unpack nested archives in one shot

**Trigger**: "There's a tar.gz inside this tar.gz", "Unpack
everything", "Extract recursively".

**Strategy**: `decompress` accepts a `level` arg — the maximum
nested-archive unpack depth. Default is 5, so simple two-layer
cases usually work without specifying it. Pass an explicit `level`
when the user names a depth or when you want to bound how far it
goes.

```jsonc
// outer.tar.gz contains inner.zip; one call, both layers.
{ "name": "decompress", "arguments": {
    "input_path":  "/abs/path/to/outer.tar.gz",
    "output_path": "/abs/path/to/extracted/",
    "level":       2
}}
```

**Read-out**: the final `output_path` contains the *deepest* layer's
contents. Intermediate archives are unpacked and the originals are
removed as part of the unwrap. If the user wanted to keep the
intermediates, decompress with `level: 1` and re-run on each layer.

## 4. Single file vs directory — wrap in tar first?

**Trigger**: agent or user says `compress this folder as gz` and
gets back `"Not a directory (os error 20)"`.

**Strategy**: `gz`, `bz2`, `xz`, `zst`, `lz4` wrap exactly **one**
file. Directories must go through `tar` first. The `tar.*` variants
combine both steps in one call.

```jsonc
// WRONG — single-file format on a directory.
{ "name": "compress", "arguments": {
    "input_path": "/abs/path/to/mydir",
    "output_path": "/abs/path/to/mydir.gz",
    "file_type":   "gz"
}}
// → { "ok": false, ...isError: true, "Not a directory (os error 20)" }

// RIGHT — tar.gz packs the directory in one step.
{ "name": "compress", "arguments": {
    "input_path":  "/abs/path/to/mydir",
    "output_path": "/abs/path/to/mydir.tar.gz",
    "file_type":   "tar.gz"
}}
```

**Inverse case**: a single file (`./report.csv`) works with either
`gz` (no tar wrapping — slightly smaller, no metadata padding) or
`tar.gz` (consistent with the directory case, but the archive
includes the tar header). Pick `gz` unless you want a uniform
"always tar" pipeline.

**Read-out**: when in doubt, ask the user *what* they're packing
(file vs directory) before picking the format string.

## 5. Working with `MAGIC_PACK_MCP_ALLOWED_ROOT`

**Trigger**: any tool call returns `"path is outside
MAGIC_PACK_MCP_ALLOWED_ROOT: ..."`.

**Strategy**: the MCP server enforces a single allow-list directory
on every input and output path. Resolve relative paths to absolute
first — they're absolutized against the *server's* cwd, which may
not be where the user expects. Inspect the server config; either
move the file inside the allow-list, or update
`MAGIC_PACK_MCP_ALLOWED_ROOT` and restart the client.

```jsonc
// Confined to /Users/me/Repos/work — anything else is rejected.
{ "name": "compress", "arguments": {
    "input_path":  "/Users/me/Repos/work/project",
    "output_path": "/Users/me/Repos/work/project.tar.zst",
    "file_type":   "tar.zst"
}}
// OK.

// This will fail:
{ "name": "compress", "arguments": {
    "input_path":  "/Users/me/Downloads/foo",
    "output_path": "/Users/me/Repos/work/foo.zip",
    "file_type":   "zip"
}}
// → "path is outside MAGIC_PACK_MCP_ALLOWED_ROOT: /Users/me/Downloads/foo"
```

**Read-out**: the error message names the offending path verbatim
— surface it to the user so they know which file to move (or which
config entry to widen). Don't silently retry with a different path.

## Recipe selection

When unsure which recipe a user query maps to:

| User phrase | Recipe |
|---|---|
| zip / tar / pack / archive a folder | #1 |
| extract / unpack `*.bin` / unknown ext | #2 |
| nested / recursive / multi-layer extract | #3 |
| "Not a directory" error / single-file vs dir | #4 |
| "outside ALLOWED_ROOT" error / config issue | #5 |
| "what kind of file is this" | start with `detect_file_type`, no recipe needed |
| "what formats do you support" | `supported_formats`, no recipe needed |
