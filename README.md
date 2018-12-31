
The following was taken from [here](https://raw.githubusercontent.com/ggez/ggez/master/docs/BuildingForEveryPlatform.md)

Run `cargo run` and it should build
and run!  ...maybe.  It depends on what platform you're on and what
libraries you have installed.  To make super-duper sure you have all
the bits and pieces in the right places to make this always work, read
on!

# Linux

## Debian

Very easy, just install the required dev packages:

```sh
apt install libasound2-dev libsdl2-dev pkg-config
```

Then you should be able to build with `cargo run`

## Redhat

Same libraries as Debian, slightly different names.  On CentOS 7 at least you can install them with:

```sh
yum install alsa-lib-devel SDL2-devel
```
# Mac

Install SDL2 with the [brew](https://brew.sh/) package manager like so:

```sh
brew install sdl2
```

which should build and install SDL2, header files and any dependencies.

# Windows

All you need to install is the SDL2 libraries but it's a pain in the butt.  The instructions here are from the [sdl2](https://github.com/AngryLawyer/rust-sdl2#user-content-windows-msvc) crate for building with MSVC, which is what I've found to be simplest:

1. Download MSVC development libraries from http://www.libsdl.org/ (SDL2-devel-2.0.x-VC.zip).
2. Unpack SDL2-devel-2.0.x-VC.zip to a folder of your choosing (You can delete it afterwards).
3. Copy all lib files from
    > SDL2-devel-2.0.x-VC\SDL2-2.0.x\lib\x64\
    
    to the Rust library folder.  For Rustup users (most common), this folder will be in
    > C:\\Users\\{Your Username}\\.rustup\\toolchains\\{current toolchain}\\lib\\rustlib\\{current toolchain}\\lib
    
    or, if not using Rustup, to (for Rust 1.6 and above)
    > C:\\Program Files\\Rust\\**lib**\\rustlib\\x86_64-pc-windows-msvc\\lib

    or to (for Rust versions 1.5 and below)
    > C:\\Program Files\\Rust\\**bin**\\rustlib\\x86_64-pc-windows-msvc\\lib

    or to your library folder of choice, and ensure you have a system environment variable of
    > LIB = C:\your\rust\library\folder

  Where current toolchain is likely `stable-x86_64-pc-windows-msvc`.
  
Note that SDL2.dll doesn't need to be copied into this project as it's already added.