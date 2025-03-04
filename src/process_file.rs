use core::str;
use std::{io::BufRead, path::PathBuf};

use printpdf::{
    color, FontId, Mm, Op, PdfPage, Point, Pt, Rgb, TextItem, TextMatrix, TextRenderingMode,
};
use syntect::{
    easy::HighlightFile,
    highlighting::{Style, ThemeSet},
    parsing::SyntaxSet,
};
static MAX_LINE_LENGTH: usize = 100;
fn split_into_chunks(slice: &str, chunk_size: usize) -> Vec<&str> {
    let mut v = vec![];
    let mut i = 0;
    while (i + chunk_size) <= slice.len() {
        v.push(&slice[i..(i + chunk_size)]);
        i += chunk_size;
    }
    v.push(&slice[i..slice.len()]);
    v
}
pub fn new_page_contents(page_dimensions: (f32, f32), font_id: FontId, path: PathBuf) -> Vec<Op> {
    vec![
        Op::SetLineHeight { lh: Pt(14.0) },
        Op::SetFontSize {
            size: Pt(12.0),
            font: font_id.clone(),
        },
        // Write metadata
        Op::SetTextCursor {
            pos: Point {
                x: Mm(10.0).into(),
                y: Mm(page_dimensions.1 - 7.5).into(),
            },
        },
        Op::WriteText {
            items: vec![TextItem::Text(path.to_str().unwrap().to_owned())],
            font: font_id.clone(),
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
            mode: TextRenderingMode::Fill,
        },
    ]
}
pub fn process_file(
    syntax_set: &SyntaxSet,
    theme_set: &ThemeSet,
    font_id: FontId,
    file_path: PathBuf,
    page_dimensions: (f32, f32),
) -> std::io::Result<Vec<PdfPage>> {
    let mut highlighter = HighlightFile::new(
        file_path.clone(),
        &syntax_set,
        &theme_set.themes["InspiredGitHub"],
    )?;

    let mut line = String::new();
    let mut pages: Vec<PdfPage> = vec![];
    let mut page_contents = new_page_contents(page_dimensions, font_id.clone(), file_path.clone());
    let mut line_count = 0;
    while highlighter.reader.read_line(&mut line).unwrap() > 0 {
        {
            line_count += 1;
            if line_count > 54 {
                // Move onto a new page
                pages.push(PdfPage::new(
                    Mm(page_dimensions.0),
                    Mm(page_dimensions.1),
                    page_contents,
                ));
                page_contents =
                    new_page_contents(page_dimensions, font_id.clone(), file_path.clone());
                line_count = 0;
            }
            // Store the char count for the current line
            let mut count_size_line_break = 0;
            let regions: Vec<(Style, &str)> = highlighter
                .highlight_lines
                .highlight_line(&line, &syntax_set)
                .unwrap();
            for r in regions {
                let (style, text) = r;
                let text_style = style.foreground;
                count_size_line_break += text.len();
                // If current line is getting too long, add a line break
                if count_size_line_break > MAX_LINE_LENGTH {
                    page_contents.push(Op::AddLineBreak);
                    count_size_line_break = 0;
                }
                page_contents.push(Op::SetOutlineColor {
                    col: color::Color::Rgb(Rgb {
                        r: (text_style.r as f32) / 255.0,
                        g: (text_style.g as f32) / 255.0,
                        b: (text_style.b as f32) / 255.0,
                        icc_profile: None,
                    }),
                });
                page_contents.push(Op::SetFillColor {
                    col: color::Color::Rgb(Rgb {
                        r: (text_style.r as f32) / 255.0,
                        g: (text_style.g as f32) / 255.0,
                        b: (text_style.b as f32) / 255.0,
                        icc_profile: None,
                    }),
                });
                if text.len() < MAX_LINE_LENGTH {
                    page_contents.push(Op::WriteText {
                        items: vec![TextItem::Text(text.to_owned())],
                        font: font_id.clone(),
                    });
                } else {
                    // Split text into chunks the maximum width of the view
                    let chunks = split_into_chunks(text, 100);
                    let mut first = true;
                    for c in chunks {
                        if !first {
                            page_contents.push(Op::AddLineBreak);
                        }
                        first = false;
                        page_contents.push(Op::WriteText {
                            items: vec![TextItem::Text(c.to_owned())],
                            font: font_id.clone(),
                        });
                    }
                }
            }
            page_contents.push(Op::AddLineBreak);
            // lines += &format!("{}", as_24_bit_terminal_escaped(&regions[..], true));
        } // until NLL this scope is needed so we can clear the buffer after
        line.clear(); // read_line appends so we need to clear between lines
    }
    pages.push(PdfPage::new(
        Mm(page_dimensions.0),
        Mm(page_dimensions.1),
        page_contents,
    ));
    Ok(pages)
}
