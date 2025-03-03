//! Prints highlighted HTML for a file to stdout.
//! Basically just wraps a body around `highlighted_html_for_file`
use core::f32;
use printpdf::*;
use std::fs;
use std::path::PathBuf;
use syntect::highlighting::ThemeSet;
use syntect::parsing::SyntaxSet;
mod process_file;
use ignore::Walk;
use process_file::process_file;
fn new_page_contents(page_dimensions: (f32, f32), font_id: FontId, path: PathBuf) -> Vec<Op> {
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
            items: vec![TextItem::Text(path.to_str().unwrap().to_owned())],
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
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();
    let mut pages: Vec<PdfPage> = vec![];
    for result in Walk::new(path) {
        match result {
            Ok(entry) => {
                if entry.file_type().is_some_and(|f| f.is_file()) {
                    // dbg!(entry.path());
                    let mut page_contents = new_page_contents(
                        page_dimensions,
                        font_id.clone(),
                        entry.path().to_path_buf(),
                    );
                    process_file(
                        &ss,
                        &ts,
                        font_id.clone(),
                        &mut page_contents,
                        entry.path().to_path_buf(),
                    );
                    pages.push(PdfPage::new(
                        Mm(page_dimensions.0),
                        Mm(page_dimensions.1),
                        page_contents,
                    ));
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
