use std::{ffi::OsStr, fs, io::BufRead, path::PathBuf};

use ignore::Walk;
use printpdf::{
    FontId, Mm, Op, PdfDocument, PdfPage, Px, RawImage, TextItem, XObjectRotation,
    XObjectTransform, color,
};
use syntect::{
    easy::HighlightFile,
    highlighting::{Color, Style, ThemeSet},
    parsing::SyntaxSet,
};

use crate::{helpers::init_page, text_manipulation::TextWrapper};

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
/// Main struct for generating PDFs.
/// It handles almost the entire process of reading and highlighting code,
/// as well as actually writing it to the PDF
pub struct CodeToPdf {
    current_page_contents: Vec<Op>,
    doc: PdfDocument,
    font_id: FontId,
    page_dimensions: (Mm, Mm),
    max_line_chars: usize,
    text_wrapper: TextWrapper,
    pub processed_file_count: usize,
}
impl CodeToPdf {
    /// Saves the current page contents to the document, and clears `current_page_contents`
    fn save_page(&mut self) {
        let contents = std::mem::take(&mut self.current_page_contents);
        let page = PdfPage::new(self.page_dimensions.0, self.page_dimensions.1, contents);
        self.doc.pages.push(page);
    }

    /// Initialises `current_page_contents` with basic contents
    fn init_page(&mut self, path: PathBuf) {
        // Should never be called on a non-empty `current_page_contents`, so check it in debug mode
        debug_assert_eq!(self.current_page_contents.len(), 0);

        init_page(
            &mut self.current_page_contents,
            self.page_dimensions,
            self.font_id.clone(),
            self.text_wrapper.font_size(),
            path,
            &mut self.text_wrapper,
        );
    }
    /// Generates all the pages for a file
    fn generate_highlighted_pages(
        &mut self,
        highlighter: &mut HighlightFile,
        path: PathBuf,
        highlighter_config: &HighlighterConfig,
    ) {
        let mut line = String::new();
        let mut line_count = 0;
        self.init_page(path.clone());
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
                let lines = self.text_wrapper.split_into_lines(text);
                // If only a single line, then no new lines are going to be made (as we're processing a region here)
                match lines.len() {
                    1 => {
                        self.current_page_contents.push(Op::WriteText {
                            items: vec![TextItem::Text(text.to_owned())],
                            font: self.font_id.clone(),
                        });
                    }
                    // If the region is too long to fit onto a new line, split and write to multiple different lines
                    _ => {
                        let mut first = true;
                        for l in lines {
                            if !first {
                                self.current_page_contents.push(Op::AddLineBreak);
                            }
                            first = false;
                            self.current_page_contents.push(Op::WriteText {
                                items: vec![TextItem::Text(l)],
                                font: self.font_id.clone(),
                            });
                            line_count += 1;
                            if line_count > 54 {
                                self.save_page();
                                self.init_page(path.clone());
                                line_count = 0;
                            }
                        }
                    }
                }
            }

            // Split text into chunks the maximum width of the view

            line_count += 1;
            // Move to new page if current page is full
            if line_count > 54 {
                self.save_page();
                self.init_page(path.clone());
                line_count = 0;
            }
            self.current_page_contents.push(Op::AddLineBreak);
            line.clear();
        }
        // Clear page if no text has been added to it
        if has_added_text {
            self.save_page();
        } else {
            self.current_page_contents.clear()
        }
    }

    /// Generates a page containing the image at the path given
    fn generate_image_page(&mut self, path: PathBuf) {
        let bytes = if let Ok(b) = fs::read(path.clone()) {
            b
        } else {
            return;
        };
        let image = if let Ok(img) = RawImage::decode_from_bytes(&bytes, &mut vec![]) {
            img
        } else {
            return;
        };
        self.init_page(path.clone());
        let image_id = self.doc.add_image(&image);
        let pg_x_dpi = self.page_dimensions.0.into_pt().into_px(300.0).0;
        let pg_y_dpi = self.page_dimensions.1.into_pt().into_px(300.0).0;

        let x_scaling = pg_x_dpi as f32 / image.width as f32;
        let y_scaling = pg_y_dpi as f32 / image.height as f32;

        let scale = f32::min(x_scaling, y_scaling);
        // If width is significantly bigger than the height, rotate so it's oriented to fill more of the page
        let rotation = if image.width > (image.height as f32 * 1.25) as usize {
            Some(XObjectRotation {
                angle_ccw_degrees: -90.0,
                rotation_center_x: Px(((image.width as f32 * scale) / 2.0) as usize),
                rotation_center_y: Px(((image.height as f32 * scale) / 2.0) as usize),
            })
        } else {
            None
        };
        self.current_page_contents.push(Op::UseXobject {
            id: image_id.clone(),
            transform: XObjectTransform {
                scale_x: Some(scale),
                scale_y: Some(scale),
                rotate: rotation,
                ..Default::default()
            },
        });
        self.save_page();
    }
    /// Generates pages for a file
    pub fn process_file(
        &mut self,
        file: PathBuf,
        highlighter_config: &HighlighterConfig,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Generating pages for {}", file.display());
        self.processed_file_count += 1;
        match file.extension().and_then(OsStr::to_str) {
            Some("jpg" | "png") => {
                self.generate_image_page(file);
                Ok(())
            }
            _ => {
                let mut highlighter = HighlightFile::new(
                    file.clone(),
                    &highlighter_config.syntax_set,
                    &highlighter_config.theme_set.themes["InspiredGitHub"],
                )?;

                self.generate_highlighted_pages(&mut highlighter, file.clone(), highlighter_config);

                Ok(())
            }
        }
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
    pub fn new(
        doc: PdfDocument,
        font_id: FontId,
        page_dimensions: (Mm, Mm),
        text_wrapper: TextWrapper,
    ) -> Self {
        Self {
            current_page_contents: vec![],
            doc,
            font_id,
            page_dimensions,
            max_line_chars: 100,
            text_wrapper,
            processed_file_count: 0,
        }
    }
    /// Consumes the instance and returns the underlying document
    pub fn document(self) -> PdfDocument {
        self.doc
    }
}
