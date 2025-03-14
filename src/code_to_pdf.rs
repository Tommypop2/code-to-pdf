use std::{io::BufRead, path::PathBuf};

use ignore::Walk;
use printpdf::{color, FontId, Op, PdfPage, TextItem};
use syntect::{
    easy::HighlightFile,
    highlighting::{Style, ThemeSet},
    parsing::SyntaxSet,
};

use crate::helpers::{new_page_contents, split_into_chunks};
static MAX_LINE_LENGTH: usize = 100;

pub struct CodeToPdf {
    pages: Vec<PdfPage>,
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    font_id: FontId,
    page_dimensions: (f32, f32),
}
impl CodeToPdf {
    /// Generates a single PdfPage
    fn generate_page(&self, highlighter: &mut HighlightFile, path: PathBuf) -> Option<PdfPage> {
        let mut line = String::new();
        let mut line_count = 0;
        let mut page_contents = new_page_contents(self.page_dimensions, self.font_id.clone(), path);
        let mut has_added_text = false;
        while highlighter.reader.read_line(&mut line).unwrap_or(0) > 0 {
            has_added_text = true;
            // Store the char count for the current line
            let mut count_size_line_break = 0;
            let regions: Vec<(Style, &str)> = highlighter
                .highlight_lines
                .highlight_line(&line, &self.syntax_set)
                .unwrap();
            for (style, text) in regions {
                count_size_line_break += text.len();
                // If current line is getting too long, add a line break
                if count_size_line_break > MAX_LINE_LENGTH {
                    page_contents.push(Op::AddLineBreak);
                    count_size_line_break = 0;
                }
                let text_style = style.foreground;
                // Set PDF text colour
                page_contents.push(Op::SetFillColor {
                    col: color::Color::Rgb(color::Rgb {
                        r: (text_style.r as f32) / 255.0,
                        g: (text_style.g as f32) / 255.0,
                        b: (text_style.b as f32) / 255.0,
                        icc_profile: None,
                    }),
                });
                if text.len() < MAX_LINE_LENGTH {
                    // Text fits within a line, so doesn't need any splitting
                    page_contents.push(Op::WriteText {
                        items: vec![TextItem::Text(text.to_owned())],
                        font: self.font_id.clone(),
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
                            font: self.font_id.clone(),
                        });
                    }
                }
            }
            line_count += 1;
            // Stop if this page is full
            if line_count > 54 {
                break;
            }
            page_contents.push(Op::AddLineBreak);
            line.clear();
        }
				// Only push new page if text has been added to it
        if has_added_text {
            Some(PdfPage::new(
                printpdf::Mm(self.page_dimensions.0),
                printpdf::Mm(self.page_dimensions.1),
                page_contents,
            ))
        } else {
            None
        }
    }
    /// Generates pages for a file
    pub fn process_file(&mut self, file: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
        let mut highlighter = HighlightFile::new(
            file.clone(),
            &self.syntax_set,
            &self.theme_set.themes["InspiredGitHub"],
        )?;
        println!("Generating pages for {}", file.display());
        while let Some(page) = self.generate_page(&mut highlighter, file.clone()) {
            self.pages.push(page)
        }
        Ok(())
    }
		/// Consumes entire walker
    pub fn process_files(&mut self, walker: Walk) {
        for result in walker {
            match result {
                Ok(entry) => {
                    if entry.file_type().is_some_and(|f| f.is_file()) {
                        if let Err(err) = self.process_file(entry.path().to_path_buf()) {
                            println!("ERROR: {}", err)
                        }
                    }
                }
                Err(err) => println!("ERROR: {}", err),
            }
        }
    }
    pub fn new(
        syntax_set: SyntaxSet,
        theme_set: ThemeSet,
        font_id: FontId,
        page_dimensions: (f32, f32),
    ) -> Self {
        Self {
            pages: vec![],
            syntax_set,
            theme_set,
            font_id,
            page_dimensions,
        }
    }
    /// Consumes the instance and returns the pages Vec
    pub fn get_pages(self) -> Vec<PdfPage> {
        self.pages
    }
}
