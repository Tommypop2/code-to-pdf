cargo build --bin c2pdf --release
hyperfine "./target/release/c2pdf.exe ./" --warmup 3 --shell=none --export-json ./hyperfine-output.json