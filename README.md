# Magic pack

Magic pack is a tool that makes it easy to compress and decompress files without remembering lots of commands.

## Quick install

```shell
cargo install magic-pack
```

### MCP server

#### Install the optional MCP server binary:

```shell
cargo install magic-pack --features mcp --bin magic-pack-mcp
```

Example MCP client config:

```json
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

Codex config example (`~/.codex/config.toml`):

```toml
[mcp_servers.magic-pack]
command = "magic-pack-mcp"

[mcp_servers.magic-pack.env]
MAGIC_PACK_MCP_ALLOWED_ROOT = "/absolute/path/you/want/to/allow"
```

#### Local test example:

```json
{
  "mcpServers": {
    "magic-pack": {
      "command": "magic-pack-mcp",
      "env": {
        "MAGIC_PACK_MCP_ALLOWED_ROOT": "/Users/zonda/Repos/magic-pack"
      }
    }
  }
}
```

Codex config example (`~/.codex/config.toml`):

```toml
[mcp_servers.magic-pack]
command = "cargo"
args = ["run", "--features", "mcp", "--bin", "magic-pack-mcp"]
cwd = "/Users/zonda/Repos/magic-pack"

[mcp_servers.magic-pack.env]
MAGIC_PACK_MCP_ALLOWED_ROOT = "/Users/zonda/Repos/magic-pack"
```

After saving the MCP config, restart your MCP client so it re-runs `magic-pack-mcp`.
Then call the tools with absolute paths inside `MAGIC_PACK_MCP_ALLOWED_ROOT`, for example:

```text
compress /Users/zonda/Repos/magic-pack/temp/test_dir to /Users/zonda/Repos/magic-pack/temp/test_dir.zip as zip
decompress /Users/zonda/Repos/magic-pack/temp/test_dir.zip to /Users/zonda/Repos/magic-pack/temp/test_dir_unpacked
detect file type of /Users/zonda/Repos/magic-pack/temp/test_dir.zip
list supported formats
```

### Commands

```shell
just all
just all-release
just quality
```

## Command

### Usage

```shell
Magic pack tool

Usage: magic-pack [OPTIONS] <--compress|--decompress> <INPUT>

Arguments:
  <INPUT>

Options:
  -c, --compress
  -f <FILE_TYPE>       [possible values: zip, tar, bz2, gz, tarbz2, targz, 7z]
  -d, --decompress
  -l, --level <LEVEL>  [default: 5]
  -o <OUTPUT>          [default: .]
  -h, --help           Print help information
  -V, --version        Print version information
```

### Example

```shell
// zip
./magic-pack -c -f zip -o temp/temp.zip src
./magic-pack -d -o temp/. temp/temp.zip

// gz (single file)
./magic-pack -c -f gz -o temp/file.txt.gz temp/file.txt
./magic-pack -d -o temp/. temp/file.txt.gz

// bz2 (single file)
./magic-pack -c -f bz2 -o temp/file.txt.bz2 temp/file.txt
./magic-pack -d -o temp/. temp/file.txt.bz2

// tar
./magic-pack -c -f tar -o temp/temp.tar src
./magic-pack -d -o temp/. temp/temp.tar

// tar.bz2
./magic-pack -c -f tarbz2 -o temp/temp.tar.bz2 src
./magic-pack -d -o temp/. temp/temp.tar.bz2

// tar.gz
./magic-pack -c -f targz -o temp/temp.tar.gz src
./magic-pack -d -o temp/. temp/temp.tar.gz

// 7z
./magic-pack -c -f 7z -o temp/temp.7z src
./magic-pack -d -o temp/. temp/temp.7z

// auto-detect format on decompress
./magic-pack -d -o temp/. temp/temp.tar.gz

// nested archives (decompress multiple layers)
./magic-pack -d -l 3 -o temp/. temp/archive.tar.gz

// output to current directory
./magic-pack -d temp/temp.zip
```

## Reference

[GNU / Linux 各種壓縮與解壓縮指令](http://note.drx.tw/2008/04/command.html)  
[File Magic Numbers](https://gist.github.com/leommoore/f9e57ba2aa4bf197ebc5)  
[File](https://github.com/file/file/blob/master/src/compress.c)  
