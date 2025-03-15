# code-to-pdf

Converts a folder of source code to a fully syntax-highlighted PDF

## Installation

### From [crates.io](https://crates.io)

```bash
cargo install code-to-pdf
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
