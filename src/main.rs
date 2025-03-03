//! Prints highlighted HTML for a file to stdout.
//! Basically just wraps a body around `highlighted_html_for_file`
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
fn main() {
    // let args: Vec<String> = std::env::args().collect();
    // if args.len() < 2 {
    //     println!("Please pass in a file to highlight");
    //     return;
    // }

    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let mut highlighter =
        HighlightFile::new("./hello.js", &ss, &ts.themes["InspiredGitHub"]).unwrap();
    let mut lines= String::new();
    let mut line = String::new();
    while highlighter.reader.read_line(&mut line).unwrap() > 0 {
        {
            let regions: Vec<(Style, &str)> = highlighter
                .highlight_lines
                .highlight_line(&line, &ss)
                .unwrap();
						dbg!(&regions);
            lines += &format!("{}", as_24_bit_terminal_escaped(&regions[..], true));

        } // until NLL this scope is needed so we can clear the buffer after
        line.clear(); // read_line appends so we need to clear between lines
    }
		print!("{}", &lines);
    // Clear the formatting
    println!("\x1b[0m");
    let page_dimensions: (f32, f32) = (210.0, 297.0);
    let mut doc = PdfDocument::new("My first PDF");
    let roboto_bytes = include_bytes!("../fonts/Helvetica.ttf");
    let font = ParsedFont::from_bytes(roboto_bytes, 33, &mut vec![]).unwrap();
    let font_id = doc.add_font(&font);
    let text_pos = Point {
        x: Mm(10.0).into(),
        y: Mm(page_dimensions.1 - 20.0).into(),
    }; // from bottom left
    let page1_contents = vec![
        Op::SetLineHeight { lh: Pt(33.0) },
        // Op::SetWordSpacing { pt: Pt(1000.0) },
        // Op::SetCharacterSpacing { multiplier: 10.0 },
        Op::SetTextCursor { pos: text_pos },
        Op::WriteText {
            items: vec![TextItem::Text(lines)],
            size: Pt(33.0),
            font: font_id,
        },
    ];
    let page1 = PdfPage::new(Mm(page_dimensions.0), Mm(page_dimensions.1), page1_contents);
    let pdf_bytes: Vec<u8> = doc
        .with_pages(vec![page1])
        .save(&PdfSaveOptions::default(), &mut vec![]);
    fs::write("./hello.pdf", pdf_bytes);
}
