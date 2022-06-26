# cargo-avrdude

[![crates.io](https://img.shields.io/crates/v/cargo-avrdude.svg)](https://crates.io/crates/cargo-avrdude)

Cargo extension for building your binary and seamlessly passing it to [AVRDUDE](https://github.com/avrdudes/avrdude) through arbitrary command-line arguments.

## Installation

Install via crates.io:

```console
cargo install cargo-avrdude
```

## Usage

Invoking `cargo avrdude` builds the crate, passing all arguments directly to `cargo build`. If there are multiple binaries in your workspace, please specify one with `--bin <binary_name>`.

The arguments passed to `avrdude` can be specified in the `Cargo.toml` of the binary crate like so:
```toml
[package.metadata.cargo_avrdude]
args = ["-p", "m328p", "-c", "usbasp", "-e", "-V", "-U", "flash:w:{}"]
```
where any occurence of the string `{}` will be replaced by the path to the compiled binary, resulting in, for example:
```console
avrdude -p m328p -c usbasp -e -V -U flash:w:/usr/binary_name/target/target/release/<binary_name>.elf
```

## License

This software is licensed under the MIT license.

See the [LICENSE](LICENSE) file for more details.