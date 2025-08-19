<img src="./assets/logo.png" width="320" />

# code-to-pdf

Converts a folder of source code to a fully syntax-highlighted PDF

## Features

- Syntax highlights code (uses [syntect](https://github.com/trishume/syntect) for highlighting and [two-face](https://crates.io/crates/two-face) for syntax definitions)
- Automatically handles line wrapping and page overflowing
- Fast. Processing runs in parallel on multiple cores
- Error-tolerant
- Configurable (custom file exclusions, output filename, fonts)
- Displays images
- Respects ignore globs in `.ignore` and `.gitignore` (uses [ignore](https://crates.io/crates/ignore))

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

### Using artefacts on Github Releases

Navigate to the latest [release](https://github.com/Tommypop2/code-to-pdf/releases)

Run the correct installer script for your system, or just download the correct binary directly

### Build from repository source

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

### Excluding paths

```bash
c2pdf . --exclude pnpm-lock.yaml,*.lock
```

### Custom output path

```bash
c2pdf . --out custom_name.pdf
```

### Setting the font

Supports loading fonts from a path or the system fonts

```bash
# From a path
c2pdf . --font fonts/Helvetica.ttf

# From system fonts
c2pdf . --font "Arial"
```

### Setting font size

```bash
c2pdf . --font-size 12.5 # 12.0 by default
```

### Setting margins

```bash
c2pdf . --margin-top 20 --margin-bottom 5 --margin-left 10 --margin-right 10 # (these are the defaults)
```

### Setting custom page text

This is text that is added to every page

```bash
c2pdf . --page-text "Hello\nWorld" # (use `\n` to indicate a newline)
```

## Decoding

Generated PDFs can be decoded - recreating the original source tree which they were generated from

```bash
dc2pdf ./generated_pdf.pdf --out-dir ./generated # Default out-dir is `./generated`
```
