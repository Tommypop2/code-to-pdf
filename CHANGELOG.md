# Changelog

All notable changes to this project will be documented in this file.

## [0.1.5] - 2025-03-29

### 🚜 Refactor

- Move `Dimensions` into a separate file
- Make `processed_file_count` private

### 📚 Documentation

- Add `syntect` mention to syntax highlighting section of `Features`
- Add comment to `Dimensions` struct
- Add more doc comments

### ⚙️ Miscellaneous Tasks

- Add warn on missing docs

<!-- generated by git-cliff -->
## [0.1.4] - 2025-03-27

### 🚀 Features

- Include more supported image formats
- Calculate the maximum number of lines that can be displayed
- Allow setting margin sizes as command-line arguments

### 🐛 Bug Fixes

- Only push a line break if a new page isn't created

### 🚜 Refactor

- Add `increment_line_function` to simplify making a new page if necessary
- Pass references to `PathBuf` where possible
- Remove (now) unused `max_line_chars`
- Replace `PathBuf` with `Path`
- Move page dimension info into `Dimensions` struct
- Remove `max_width` from `TextWrapper` initialiser
- Implement `Default` for `Dimensions`
- Only store `theme` in `HighlighterConfig`
- Switch to theme set from `two_face`

### 📚 Documentation

- Add docs for installing via `cargo binstall`
- Show usage for more command line options
- Add mention of supporting ignore files
- Add example for setting margins

### ⚙️ Miscellaneous Tasks

- Fmt + clippy
- Fmt + clippy
- Don't create updater programs

<!-- generated by git-cliff -->
## [0.1.3] - 2025-03-24

### ⚙️ Miscellaneous Tasks

- Allow for usage as a library

<!-- generated by git-cliff -->
## [0.1.2] - 2025-03-24

### ⚙️ Miscellaneous Tasks

- Add license and description fields to `Cargo.toml`

<!-- generated by git-cliff -->
