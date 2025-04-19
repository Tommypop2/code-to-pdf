cargo run --release --features "font-loading" --bin c2pdf -- ./ #--font "Departure Mono" # Needs to be in release for full PDF optimisation
cargo run --bin dc2pdf ./output.pdf