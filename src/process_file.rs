use core::str;
use std::{io::BufRead, path::PathBuf};

use printpdf::{color, FontId, Op, Pt, Rgb, TextItem};
use syntect::{
    easy::HighlightFile,
    highlighting::{Style, ThemeSet},
    parsing::SyntaxSet,
};
static MAX_LINE_LENGTH: usize = 100;
pub fn process_file(
    syntax_set: &SyntaxSet,
    theme_set: &ThemeSet,
    font_id: FontId,
    page_contents: &mut Vec<Op>,
    file_path: PathBuf,
) {
    let mut highlighter =
        HighlightFile::new(file_path, &syntax_set, &theme_set.themes["InspiredGitHub"]).unwrap();
    let mut line = String::new();
    while highlighter.reader.read_line(&mut line).unwrap() > 0 {
        {
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
                        size: Pt(12.0),
                        font: font_id.clone(),
                    });
                } else {
                    // Split text into chunks the maximum size of the view
                    let chunks = text
                        .as_bytes()
                        .chunks(MAX_LINE_LENGTH)
                        .map(str::from_utf8)
                        .collect::<Result<Vec<&str>, _>>()
                        .unwrap();
                    let mut first = true;
                    for c in chunks {
                        if !first {
                            page_contents.push(Op::AddLineBreak);
                        }
                        first = false;
                        page_contents.push(Op::WriteText {
                            items: vec![TextItem::Text(c.to_owned())],
                            size: Pt(12.0),
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
}
