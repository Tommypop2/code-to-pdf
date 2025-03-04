//! Prints highlighted HTML for a file to stdout.
//! Basically just wraps a body around `highlighted_html_for_file`
use core::f32;
use printpdf::*;
use std::fs;
use std::path::PathBuf;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
mod process_file;
use ignore::{Walk, WalkBuilder};
use process_file::{new_page_contents, process_file};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        println!("Please pass in a path!");
        return;
    }
    let path = args[1].clone();
    let page_dimensions: (f32, f32) = (210.0, 297.0);
    let mut doc = PdfDocument::new("Project Code");
    let helvetica_bytes = include_bytes!("../fonts/Helvetica.ttf");
    let font = ParsedFont::from_bytes(helvetica_bytes, 33, &mut vec![]).unwrap();
    let font_id = doc.add_font(&font);
    // Highlighting stuff
    // let ss = SyntaxSet::load_defaults_newlines();
		let ss = two_face::syntax::extra_newlines();
    let ts = ThemeSet::load_defaults();
    let mut pages: Vec<PdfPage> = vec![];
    for result in WalkBuilder::new(path).add_custom_ignore_filename("pnpm-lock.yaml").build() {
        match result {
            Ok(entry) => {
                if entry.file_type().is_some_and(|f| f.is_file()) {
                    // dbg!(entry.path());

                    let res = process_file(
                        &ss,
                        &ts,
                        font_id.clone(),
                        entry.path().to_path_buf(),
                        page_dimensions,
                    );
                    match res {
                        Ok(ps) => {
                            for p in ps {
                                pages.push(p);
                            }
                        }
                        Err(err) => {
                            println!(
                                "Processing {} failed",
                                entry.path().to_str().unwrap_or("unknown")
                            );
                            println!("ERROR: {}", err);
                        }
                    }
                    // pages.push(PdfPage::new(
                    //     Mm(page_dimensions.0),
                    //     Mm(page_dimensions.1),
                    //     page_contents,
                    // ));
                }
            }
            Err(err) => println!("ERROR: {}", err),
        }
    }
    let pdf_bytes: Vec<u8> = doc
        .with_pages(pages)
        .save(&PdfSaveOptions::default(), &mut vec![]);
    fs::write("./hello.pdf", pdf_bytes).unwrap();
}
