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
    let text_pos = Point {
        x: Mm(10.0).into(),
        y: Mm(page_dimensions.1 - 20.0).into(),
    }; // from bottom left
       // Highlighting stuff
    let ss = SyntaxSet::load_defaults_newlines();
    let ts = ThemeSet::load_defaults();

    let mut highlighter =
        HighlightFile::new("./hello.js", &ss, &ts.themes["InspiredGitHub"]).unwrap();
    let mut page_contents = vec![
        Op::SetLineHeight { lh: Pt(33.0) },
        // Op::SetWordSpacing { pt: Pt(1000.0) },
        // Op::SetCharacterSpacing { multiplier: 10.0 },
        Op::SetTextCursor { pos: text_pos },
        Op::SetTextRenderingMode {
            mode: TextRenderingMode::StrokeClip,
        },
        // Op::SetFillColor {
        //     col: color::Color::Rgb(Rgb {
        //         r: 255.0,
        //         g: 0.0,
        //         b: 0.0,
        //         icc_profile: None,
        //     }),
        // },
        // Op::SetOutlineColor {
        //     col: color::Color::Rgb(Rgb {
        //         r: 0.0,
        //         g: 0.0,
        //         b: 0.0,
        //         icc_profile: None,
        //     }),
        // },
    ];
    let mut lines = String::new();
    let mut line = String::new();
    while highlighter.reader.read_line(&mut line).unwrap() > 0 {
        {
            let regions: Vec<(Style, &str)> = highlighter
                .highlight_lines
                .highlight_line(&line, &ss)
                .unwrap();
            for r in regions {
                let (style, text) = r;
                let text_style = style.foreground;
                page_contents.push(Op::SetOutlineColor {
                    col: color::Color::Rgb(Rgb {
                        r: (text_style.r as f32) / 255.0,
                        g: (text_style.g as f32) / 255.0,
                        b: (text_style.b as f32) / 255.0,
                        icc_profile: None,
                    }),
                });
                page_contents.push(Op::WriteText {
                    items: vec![TextItem::Text(text.to_owned())],
                    size: Pt(12.0),
                    font: font_id.clone(),
                });
            }
            // lines += &format!("{}", as_24_bit_terminal_escaped(&regions[..], true));
        } // until NLL this scope is needed so we can clear the buffer after
        line.clear(); // read_line appends so we need to clear between lines
    }
    print!("{}", &lines);
    // Clear the formatting
    println!("\x1b[0m");

    let page1 = PdfPage::new(Mm(page_dimensions.0), Mm(page_dimensions.1), page_contents);
    let pdf_bytes: Vec<u8> = doc
        .with_pages(vec![page1])
        .save(&PdfSaveOptions::default(), &mut vec![]);
    fs::write("./hello.pdf", pdf_bytes);
}
