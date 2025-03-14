use std::{io::BufRead, path::PathBuf};

use ignore::Walk;
use printpdf::{color, FontId, Op, PdfPage, TextItem};
use syntect::{
    easy::HighlightFile,
    highlighting::{Color, Style, ThemeSet},
    parsing::SyntaxSet,
};

use crate::helpers::{init_page, split_into_chunks};
pub struct HighlighterConfig {
    syntax_set: SyntaxSet,
    theme_set: ThemeSet,
    max_line_len_to_highlight: usize,
}
impl HighlighterConfig {
    pub fn new(syntax_set: SyntaxSet, theme_set: ThemeSet) -> Self {
        Self {
            syntax_set,
            theme_set,
            max_line_len_to_highlight: 20_000,
        }
    }
}
pub struct CodeToPdf {
    current_page_contents: Vec<Op>,
    pages: Vec<PdfPage>,
    font_id: FontId,
    page_dimensions: (f32, f32),
    max_line_chars: usize,
}
impl CodeToPdf {
    /// Create new PdfPage with `current_page_contents` and reset `current_page_contents`
    fn new_page(&mut self) {
        let contents = std::mem::take(&mut self.current_page_contents);
        let page = PdfPage::new(
            printpdf::Mm(self.page_dimensions.0),
            printpdf::Mm(self.page_dimensions.1),
            contents,
        );
        self.pages.push(page);
    }

    /// Generates all the pages for a file
    fn generate_pages(
        &mut self,
        highlighter: &mut HighlightFile,
        path: PathBuf,
        highlighter_config: &HighlighterConfig,
    ) {
        let mut line = String::new();
        let mut line_count = 0;
        init_page(
            &mut self.current_page_contents,
            self.page_dimensions,
            self.font_id.clone(),
            path.clone(),
        );
        let mut has_added_text = false;
        while highlighter.reader.read_line(&mut line).unwrap_or(0) > 0 {
            has_added_text = true;
            // Store the char count for the current line
            let mut count_size_line_break = 0;
            let regions: Vec<(Style, &str)> =
                if line.len() < highlighter_config.max_line_len_to_highlight {
                    highlighter
                        .highlight_lines
                        .highlight_line(&line, &highlighter_config.syntax_set)
                        .unwrap()
                } else {
                    vec![(
                        Style {
                            foreground: Color::BLACK,
                            background: Color::WHITE,
                            font_style: syntect::highlighting::FontStyle::default(),
                        },
                        &line,
                    )]
                };
            for (style, text) in regions {
                count_size_line_break += text.len();
                // If current line is getting too long, add a line break
                if count_size_line_break > self.max_line_chars {
                    self.current_page_contents.push(Op::AddLineBreak);
                    count_size_line_break = 0;
                }
                let text_style = style.foreground;
                // Set PDF text colour
                self.current_page_contents.push(Op::SetFillColor {
                    col: color::Color::Rgb(color::Rgb {
                        r: (text_style.r as f32) / 255.0,
                        g: (text_style.g as f32) / 255.0,
                        b: (text_style.b as f32) / 255.0,
                        icc_profile: None,
                    }),
                });
                if text.len() < self.max_line_chars {
                    // Text fits within a line, so doesn't need any splitting
                    self.current_page_contents.push(Op::WriteText {
                        items: vec![TextItem::Text(text.to_owned())],
                        font: self.font_id.clone(),
                    });
                } else {
                    // Split text into chunks the maximum width of the view
                    let chunks = split_into_chunks(text, 100);
                    let mut first = true;
                    for c in chunks {
                        if !first {
                            self.current_page_contents.push(Op::AddLineBreak);
                        }
                        first = false;
                        self.current_page_contents.push(Op::WriteText {
                            items: vec![TextItem::Text(c.to_owned())],
                            font: self.font_id.clone(),
                        });
                        line_count += 1;
                        if line_count > 54 {
                            self.new_page();
                            init_page(
                                &mut self.current_page_contents,
                                self.page_dimensions,
                                self.font_id.clone(),
                                path.clone(),
                            );
                            line_count = 0;
                        }
                    }
                }
            }
            line_count += 1;
            // Move to new page if current page is full
            if line_count > 54 {
                self.new_page();
                init_page(
                    &mut self.current_page_contents,
                    self.page_dimensions,
                    self.font_id.clone(),
                    path.clone(),
                );
                line_count = 0;
            }
            self.current_page_contents.push(Op::AddLineBreak);
            line.clear();
        }
        // Clear page if no text has been added to it
        if has_added_text {
            self.new_page();
        } else {
            self.current_page_contents.clear()
        }
    }
    /// Generates pages for a file
    pub fn process_file(
        &mut self,
        file: PathBuf,
        highlighter_config: &HighlighterConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut highlighter = HighlightFile::new(
            file.clone(),
            &highlighter_config.syntax_set,
            &highlighter_config.theme_set.themes["InspiredGitHub"],
        )?;
        println!("Generating pages for {}", file.display());

        self.generate_pages(&mut highlighter, file.clone(), highlighter_config);

        Ok(())
    }
    /// Consumes entire walker
    pub fn process_files(&mut self, walker: Walk, highlighter_config: HighlighterConfig) {
        for result in walker {
            match result {
                Ok(entry) => {
                    if entry.file_type().is_some_and(|f| f.is_file()) {
                        if let Err(err) =
                            self.process_file(entry.path().to_path_buf(), &highlighter_config)
                        {
                            println!("ERROR: {}", err)
                        }
                    }
                }
                Err(err) => println!("ERROR: {}", err),
            }
        }
    }
    pub fn new(font_id: FontId, page_dimensions: (f32, f32)) -> Self {
        Self {
            current_page_contents: vec![],
            pages: vec![],
            font_id,
            page_dimensions,
            max_line_chars: 100,
        }
    }
    /// Consumes the instance and returns the pages Vec
    pub fn get_pages(self) -> Vec<PdfPage> {
        self.pages
    }
}
