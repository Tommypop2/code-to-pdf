//! Prints highlighted HTML for a file to stdout.
//! Basically just wraps a body around `highlighted_html_for_file`
use core::f32;
use printpdf::*;
use std::{cmp::Ordering, fs};
use syntect::highlighting::ThemeSet;
mod process_file;
use ignore::WalkBuilder;
mod code_to_pdf;
use code_to_pdf::CodeToPdf;
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
    let walker = WalkBuilder::new(path)
        .sort_by_file_path(|x, y| {
            {
                if x.is_dir() && !y.is_dir() {
                    Ordering::Less
                } else if y.is_dir() && !x.is_dir() {
                    Ordering::Greater
                } else {
                    Ordering::Equal
                }
            }
            .reverse()
        })
        .build();
    let mut c2pdf = CodeToPdf::new(ss, ts, font_id, page_dimensions);
    c2pdf.process_files(walker);
    let pdf_bytes: Vec<u8> = doc
        .with_pages(c2pdf.pages)
        .save(&PdfSaveOptions::default(), &mut vec![]);
    fs::write("./output.pdf", pdf_bytes).unwrap();
}
