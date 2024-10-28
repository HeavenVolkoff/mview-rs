# mview-extract

A tool for unpacking Marmoset Viewer archive packages, enabling the extraction and decompression of 3D models and scenes.

## Installation

Via Cargo:

```sh
cargo install mview-extract
```

Or clone the repository and build it manually:

```sh
git clone https://github.com/HeavenVolkoff/mview-rs
cd mview-rs
cargo build --release -p mview-extract
```

## Usage

```
Usage: mview-extract [OPTIONS] <FILENAME>

Arguments:
  <FILENAME>  Path to the .mview file to be extracted

Options:
  -o, --output-dir <DIR>  Output directory for extracted files (defaults to current directory)
      --no-create-subdir  Do not create a subdirectory for extracted files
  -h, --help              Print help
  -V, --version           Print version
```

## License

This project is licensed under the MIT License. See the [LICENSE](../LICENSE) file for details.
