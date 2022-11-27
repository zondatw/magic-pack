# Magic pack

## Prerequisites

Install following commands:
* file
* zip
* unzip
* tar

## Command

```shell
// zip
./magic-pack -c -f zip -i src/* -o temp/temp.zip
./magic-pack -d -i temp/temp.zip -o temp/.

// tar
./magic-pack -c -f tar -i src/* -o temp/temp.tar
./magic-pack -d -i temp/temp.tar -o temp/.

// tar.bz2
./magic-pack -c -f tarbz2 -i src/* -o temp/temp.tar.bz2
./magic-pack -d -i temp/temp.tar.bz2 -o temp/.
 
// tar.gz
./magic-pack -c -f targz -i src/* -o temp/temp.tar.gz
./magic-pack -d -i temp/temp.tar.gz -o temp/.
```

## Todo

Current is use existed command, maybe can coding compression algorithm by self in future.

## Reference

[GNU / Linux 各種壓縮與解壓縮指令](http://note.drx.tw/2008/04/command.html)  