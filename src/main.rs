//! Prints highlighted HTML for a file to stdout.
//! Basically just wraps a body around `highlighted_html_for_file`
use core::f32;
use printpdf::html;
use printpdf::*;
use std::collections::BTreeMap;
use std::fs;
use std::io::BufRead;
use syntect::easy::HighlightFile;
use syntect::highlighting::{Color, Highlighter, Style, ThemeSet};
use syntect::html::highlighted_html_for_file;
use syntect::parsing::SyntaxSet;
use syntect::util::as_24_bit_terminal_escaped;
mod process_file;
use process_file::process_file;
fn new_page_contents(page_dimensions: (f32, f32), font_id: FontId) -> Vec<Op> {
    vec![
        Op::SetLineHeight { lh: Pt(14.0) },
        // Write metadata
        Op::SetTextCursor {
            pos: Point {
                x: Mm(10.0).into(),
                y: Mm(page_dimensions.1 - 5.0).into(),
            },
        },
        Op::WriteText {
            items: vec![TextItem::Text("src/hello.js".to_owned())],
            size: Pt(12.0),
            font: font_id,
        },
        // This allows me to reset the text cursor for some reason
        Op::SetTextMatrix {
            matrix: TextMatrix::Translate(Pt(0.0), Pt(0.0)),
        },
        Op::SetTextCursor {
            pos: Point {
                x: Mm(10.0).into(),
                y: Mm(page_dimensions.1 - 20.0).into(),
            },
        },
        Op::SetTextRenderingMode {
            mode: TextRenderingMode::Stroke,
        },
    ]
}
fn main() {
    // let args: Vec<String> = std::env::args().collect();
    // if args.len() < 2 {
    //     println!("Please pass in a file to highlight");
    //     return;
    // }
    let page_dimensions: (f32, f32) = (210.0, 297.0);
    let mut doc = PdfDocument::new("My first PDF");
    let helvetica_bytes = include_bytes!("../fonts/Helvetica.ttf");
    let font = ParsedFont::from_bytes(helvetica_bytes, 33, &mut vec![]).unwrap();
    let font_id = doc.add_font(&font);
    // Highlighting stuff
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let mut page_contents = new_page_contents(page_dimensions, font_id.clone());
    process_file(&ss, &ts, font_id, &mut page_contents, "./test_folder/hello.js".into());
		
    let page1 = PdfPage::new(Mm(page_dimensions.0), Mm(page_dimensions.1), page_contents);
    let pdf_bytes: Vec<u8> = doc
        .with_pages(vec![page1])
        .save(&PdfSaveOptions::default(), &mut vec![]);
    fs::write("./hello.pdf", pdf_bytes).unwrap();
}
