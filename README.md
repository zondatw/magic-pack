# Magic pack

Magic pack is a tool that makes it easy to compress and decompress files without remembering lots of commands.

## Quick install

```shell
cargo install magic-pack
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
  -f <FILE_TYPE>       [possible values: zip, tar, bz2, gz, tarbz2, targz]
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
