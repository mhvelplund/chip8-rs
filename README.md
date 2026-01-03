# chip8.rs

A [CHIP-8] emulator implemented in Rust.

Based on the [Wikipedia][CHIP-8] documentation and [CHIP-8 Classic Manual Rev 1.3.pdf].

The Hardware specifications are:

* VM running at 48MHz.
* 64x32 pixel monochrome display, using text on STDIO.

There is no sound at the moment, and the VM starts from a cartridge memory image stored in a file.

A mapping between keyboard typing and CHIP-8's hexadecimal keypad support is planned.

The plan is to support all of the opcodes on the [Wikipedia] page, with the exception of the FX18 instruction related to
sound.

## Build and run

Build for Linux:

```shell
cargo build --release
```

Cross build from Linux to Windows:

```shell
cargo install cross
cross build -r --target i686-pc-windows-gnu
```

<!-- Links -->
[CHIP-8]: https://en.wikipedia.org/wiki/CHIP-8
[CHIP-8 Classic Manual Rev 1.3.pdf]: https://storage.googleapis.com/wzukusers/user-34724694/documents/5c83d6a5aec8eZ0cT194/CHIP-8%20Classic%20Manual%20Rev%201.3.pdf
[Wikipedia]: https://en.wikipedia.org/wiki/CHIP-8#Opcode_table