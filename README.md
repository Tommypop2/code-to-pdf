# code-to-pdf

Converts a folder of source code to a fully syntax-highlighted PDF

## Features

- Syntax highlights code (uses [two-face](https://crates.io/crates/two-face) for syntax definitions)
- Automatically handles line wrapping and page overflowing
- Fast
- Error-tolerant
- Configurable (custom file exclusions, output filename, fonts)
- Displays images

## Installation

### From [crates.io](https://crates.io/crates/code-to-pdf)

```bash
cargo install code-to-pdf
```

### Using `cargo binstall`

Installs a pre-built binary if it is available for your system

```bash
cargo binstall code-to-pdf
```

### From this repository

```bash
git clone https://github.com/Tommypop2/code-to-pdf
cargo install --path ./code-to-pdf
```

## Usage

### Generating a PDF from a folder

```bash
c2pdf .
```

This walks the current folder and generates a syntax-highlighted PDF of all files in that folder
