# chip8.rs

A [CHIP-8] interpreter implemented in Rust, based on the documentation the "[References](#references)", primarily Matt
Mikolay's "[CHIP‐8 Technical Reference]".

The Hardware specifications are:

* VM running at 48MHz.
* 64x32 pixel monochrome display, using text on STDIO.

VM starts from a ROM image stored in a file.

<!-- A mapping between keyboard typing and CHIP-8's hexadecimal keypad support is planned. -->

The plan is to support all of the opcodes on the [Wikipedia] page, with the exception of the FX18 instruction related to
sound (see "[Quirks](#quirks))".

## Quirks

Unlike the original CHIP-8, there is no interpreter in the first 512 bytes, but programs still start from 0x200.

There is no sound support at the moment.

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

## Coverage

Source:
[Visualizing Rust Code Coverage in VS Code](https://nattrio.medium.com/visualizing-rust-code-coverage-in-vs-code-781aaf334f11).

```shell
cargo install cargo-nextest --locked
cargo +stable install cargo-llvm-cov --locked
```

Install:
* [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
* [Coverage Gutters](https://marketplace.visualstudio.com/items?itemName=ryanluker.vscode-coverage-gutters)

Generate coverage:

```shell
cargo llvm-cov nextest --lcov --output-path ./target/lcov.info
```

_or_

```shell
cargo watch -x 'llvm-cov nextest --lcov --output-path ./target/lcov.info' -w src
```

Display the coverage from Coverage Gutters extension.

## References

* [CHIP-8]
* [CHIP-8 Classic Manual Rev 1.3.pdf](https://storage.googleapis.com/wzukusers/user-34724694/documents/5c83d6a5aec8eZ0cT194/CHIP-8%20Classic%20Manual%20Rev%201.3.pdf)
* [CHIP‐8 Technical Reference]
* [Mastering CHIP‐8](https://github.com/mattmikolay/chip-8/wiki/Mastering-CHIP%E2%80%908)
* [RCA COSMAC VIP CDP188711 Instruction Manual](http://www.bitsavers.org/components/rca/cosmac/COSMAC_VIP_Instruction_Manual_1978.pdf)
* [Wikipedia](https://en.wikipedia.org/wiki/CHIP-8#Opcode_table)

<!-- Links -->
[CHIP-8]: https://en.wikipedia.org/wiki/CHIP-8
[CHIP‐8 Technical Reference]: https://github.com/mattmikolay/chip-8/wiki/CHIP%E2%80%908-Technical-Reference