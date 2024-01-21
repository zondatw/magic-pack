# Magic pack

This the pack tool, it's provides user can easy to compress and decompress without remember a lot of commands.

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

### Uages

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

// tar
./magic-pack -c -f tar -o temp/temp.tar src
./magic-pack -d -o temp/. temp/temp.tar

// tar.bz2
./magic-pack -c -f tarbz2 -o temp/temp.tar.bz2 src
./magic-pack -d -o temp/. temp/temp.tar.bz2

// tar.gz
./magic-pack -c -f targz -o temp/temp.tar.gz src
./magic-pack -d -o temp/. temp/temp.tar.gz
```

## Reference

[GNU / Linux 各種壓縮與解壓縮指令](http://note.drx.tw/2008/04/command.html)  
[File Magic Numbers](https://gist.github.com/leommoore/f9e57ba2aa4bf197ebc5)  
[File](https://github.com/file/file/blob/master/src/compress.c)  
