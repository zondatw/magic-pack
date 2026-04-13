# Dev

## Arch Linux

```shell
docker run -it --rm --platform linux/amd64 --security-opt seccomp=unconfined greyltc/archlinux-aur:yay bash
```

In docker:
```shell
sudo pacman -S --needed base-devel git
sudo sed -i '/\[options\]/a DisableSandbox' /etc/pacman.conf
su -s /bin/bash ab
yay -S magic-pack
```

## ubuntu

```shell
docker run --rm -it ubuntu:22.04 bash
```


In docker:
```shell
apt update && apt install -y wget sudo

wget https://github.com/zondatw/magic-pack/releases/download/v0.11.1/magic-pack_0.11.1-1_arm64.deb
sudo apt install ./magic-pack_0.11.1_arm64.deb -y

magic-pack --version
```
