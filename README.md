# Magic pack

## Prerequisites

Install following commands:
* file

## Command

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