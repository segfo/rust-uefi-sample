What's this?
------------

This is sample program for UEFI apps written by Rust lang!

How to build
------------

- First, prepare GNU binutils, its target for x86_64-efi-pe
- Second, you have to use Rust nightly compiler.

```sh
$ rustup install nightly
$ rustup default nightly
$ cargo install xargo
$ export PATH="$HOME/.cargo/bin:$PATH"
```

and introduce x86\_64-efi-pe binutils

```sh
$ curl -O https://orum.in/distfiles/x86_64-efi-pe-binutils.tar.xz
$ mkdir $PWD/toolchain
$ tar xf x86_64-efi-pe-binutils.tar.xz -C $PWD/toolchain
$ export PATH=$PATH:$PWD/toolchain/usr/bin/
```

this repository clone, and run the build(make command).

```sh
$ git clone https://github.com/segfo/rust-uefi-sample
$ cd rust-uefi-sample
$ export RUST_TARGET_PATH=`pwd`
$ make
```

How to run
-------------

- then, kick `make run` command.
