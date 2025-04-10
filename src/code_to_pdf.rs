//! Contains [`HighlighterConfig`] and [`CodeToPdf`] structs

use std::{
    cmp::Ordering,
    collections::BTreeMap,
    ffi::OsStr,
    fs,
    io::BufRead,
    mem,
    path::Path,
    sync::{Arc, Mutex},
};

use ignore::Walk;
use printpdf::{
    FontId, Op, PdfDocument, PdfPage, Pt, Px, RawImage, TextItem,
    XObject, XObjectId, XObjectRotation, XObjectTransform, color,
};
use syntect::{
    easy::HighlightFile,
    highlighting::{Color, Style, Theme},
    parsing::SyntaxSet,
};

use crate::{dimensions::Dimensions, helpers::init_page, text_manipulation::TextWrapper};
use rayon::prelude::*;
/// Configuration struct for the highlighter ([`syntect`])
///
/// Contains the desired theme, syntax set, and the maximum line length to highlight
pub struct HighlighterConfig {
    syntax_set: SyntaxSet,
    theme: Theme,
    max_line_len_to_highlight: usize,
}
impl HighlighterConfig {
    /// Initialises new [`HighlighterConfig`]
    pub fn new(syntax_set: SyntaxSet, theme: Theme) -> Self {
        Self {
            syntax_set,
            theme,
            max_line_len_to_highlight: 20_000,
        }
    }
}
/// Subset of `PdfDocument`. Created as some types within `PdfDocument` weren't sync so it couldn't be used with `rayon`
#[derive(Default)]
pub struct DocumentSubset {
    x_object_map: BTreeMap<XObjectId, XObject>,
    // font_map: Arc<Mutex<BTreeMap<FontId, ParsedFont>>>,
    pages: Vec<(PdfPage, usize)>,
}
impl DocumentSubset {
    pub fn add_image(&mut self, image: &RawImage) -> XObjectId {
        let id = XObjectId::new();
        self.x_object_map
            .insert(id.clone(), XObject::Image(image.clone()));
        id
    }
    pub fn to_document(&mut self, doc: &mut PdfDocument) {
        let x_obj_map = mem::take(&mut self.x_object_map);
        doc.resources.xobjects.map = x_obj_map;
        let mut pages = mem::take(&mut self.pages);
        pages.sort_by(|a, b| {
            let ia = a.1;
            let ib = b.1;
            if ia > ib {
                Ordering::Greater
            } else if ia < ib {
                Ordering::Less
            } else {
                Ordering::Equal
            }
        });
        doc.pages = pages.into_iter().map(|f| f.0).collect();
    }
    // pub fn add_font(&mut self, font: &ParsedFont) -> FontId {
    //     let id = FontId::new();
    //     self.font_map.lock().unwrap().insert(id.clone(), font.clone());
    //     id
    // }
}
/// Main struct for generating PDFs.
/// It handles almost the entire process of reading and highlighting code,
/// as well as actually writing it to the PDF
pub struct CodeToPdf {
    current_page_contents: Vec<Op>,
    doc: Arc<Mutex<DocumentSubset>>,
    font_id: FontId,
    page_dimensions: Dimensions,
    text_wrapper: TextWrapper,
    processed_file_count: usize,
    // Text to put at the top of every page
    page_text: Option<String>,
}
impl CodeToPdf {
    /// Initialises a new [`CodeToPdf`]
    pub fn new(
        doc: Arc<Mutex<DocumentSubset>>,
        font_id: FontId,
        page_dimensions: Dimensions,
        text_wrapper: TextWrapper,
        page_text: Option<String>,
    ) -> Self {
        Self {
            current_page_contents: vec![],
            doc,
            font_id,
            page_dimensions,
            text_wrapper,
            processed_file_count: 0,
            page_text,
        }
    }
    /// Saves the current page contents to the document, and clears [`CodeToPdf::current_page_contents`]
    fn save_page(&mut self, index: usize) {
        let contents = std::mem::take(&mut self.current_page_contents);
        let page = PdfPage::new(
            self.page_dimensions.width,
            self.page_dimensions.height,
            contents,
        );
        _ = self.doc.lock().map(|mut doc| {
            doc.pages.push((page, index));
            
        });
        // self.doc.pages.push(page);
    }

    /// Initialises [`CodeToPdf::current_page_contents`] with basic contents
    fn init_page(&mut self, path: &Path) {
        // Should never be called on a non-empty current_pages_contents, so check it in debug mode
        debug_assert_eq!(self.current_page_contents.len(), 0);

        init_page(
            &mut self.current_page_contents,
            &self.page_dimensions,
            self.font_id.clone(),
            self.text_wrapper.font_size(),
            path,
            self.page_text.as_deref(),
            &mut self.text_wrapper,
        );
    }
    /// Computes maximum number of lines that can be displayed on a page
    fn max_line_count(&self) -> u32 {
        let max_height = self.page_dimensions.max_text_height();
        ((max_height).into_pt().0 / (self.text_wrapper.font_size() * 1.2)).floor() as u32
    }
    /// Increment given line_count. Begin a new page if it's too high
    /// Returns `true` if a new page is created
    fn increment_line_count(&mut self, line_count: &mut u32, path: &Path, index: usize) -> bool {
        *line_count += 1;
        if *line_count > self.max_line_count() {
            self.save_page(index);
            self.init_page(path);
            *line_count = 0;
            true
        } else {
            false
        }
    }
    /// Generates all the pages for a file
    fn generate_highlighted_pages(
        &mut self,
        highlighter: &mut HighlightFile,
        path: &Path,
        highlighter_config: &HighlighterConfig,
        index: usize,
    ) {
        let mut line = String::new();
        let mut line_count = 0;
        self.init_page(path);
        let mut has_added_text = false;
        while highlighter.reader.read_line(&mut line).unwrap_or(0) > 0 {
            has_added_text = true;
            // Store the char count for the current line
            let mut line_width = 0.0;
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
                let text_width = self.text_wrapper.get_width(text).0;

                let line_width_remaining =
                    self.page_dimensions.max_text_width().into_pt().0 - line_width;

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
                // Split into lines, with the length of the first line being the length remaining on the current line
                let lines = self.text_wrapper.split_into_lines(text, |i| match i {
                    0 => Pt(line_width_remaining),
                    _ => self.page_dimensions.max_text_width().into_pt(),
                });
                match lines.len() {
                    // If only a single line, then no new lines are going to be made (as we're processing a region here)
                    1 => {
                        self.current_page_contents.push(Op::WriteText {
                            items: vec![TextItem::Text(text.to_owned())],
                            font: self.font_id.clone(),
                        });
                        line_width += text_width;
                    }
                    // If the region is too long to fit onto the current line, write to multiple different lines
                    _ => {
                        let mut first = true;
                        for (l, width) in lines {
                            if !first {
                                self.current_page_contents.push(Op::AddLineBreak);
                                line_width = 0.0;
                            }
                            first = false;
                            line_width += width;
                            self.current_page_contents.push(Op::WriteText {
                                items: vec![TextItem::Text(l)],
                                font: self.font_id.clone(),
                            });
                            self.increment_line_count(&mut line_count, path, index);
                        }
                    }
                }
            }

            if !self.increment_line_count(&mut line_count, path, index) {
                self.current_page_contents.push(Op::AddLineBreak);
            }
            line.clear();
        }
        // Clear page if no text has been added to it
        if has_added_text {
            self.save_page(index);
        } else {
            self.current_page_contents.clear()
        }
    }

    /// Generates a page containing the image at the path given
    fn generate_image_page(&mut self, path: &Path, index: usize) {
        let bytes = if let Ok(b) = fs::read(path) {
            b
        } else {
            return;
        };
        let image = if let Ok(img) = RawImage::decode_from_bytes(&bytes, &mut vec![]) {
            img
        } else {
            return;
        };
        self.init_page(path);
        // let image_id = self.doc.add_image(&image);
        let image_id = self
            .doc
            .lock()
            .map(|mut doc| {
                
                doc.add_image(&image)
            })
            .unwrap();
        let pg_x_dpi = self.page_dimensions.width.into_pt().into_px(300.0).0;
        let pg_y_dpi = self.page_dimensions.height.into_pt().into_px(300.0).0;

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
        self.save_page(index);
    }
    /// Generates pages for a file
    pub fn process_file(
        &mut self,
        file: &Path,
        highlighter_config: &HighlighterConfig,
        index: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        println!("Generating pages for {}, index {index}", file.display());
        self.processed_file_count += 1;
        match file.extension().and_then(OsStr::to_str) {
            Some("jpg" | "jpeg" | "png" | "ico" | "bmp" | "webp") => {
                self.generate_image_page(file, index);
                Ok(())
            }
            _ => {
                let mut highlighter = HighlightFile::new(
                    file,
                    &highlighter_config.syntax_set,
                    &highlighter_config.theme,
                )?;

                self.generate_highlighted_pages(&mut highlighter, file, highlighter_config, index);

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
                        if let Err(err) = self.process_file(entry.path(), &highlighter_config, 0) {
                            println!("ERROR: {}", err)
                        }
                    }
                }
                Err(err) => println!("ERROR: {}", err),
            }
        }
    }

    /// Consumes the instance and returns the underlying document
    // pub fn document(self) -> PdfDocument {
    //     self.doc
    // }

    /// Returns number of files processed by [`CodeToPdf::process_files`]
    pub fn processed_file_count(&self) -> usize {
        self.processed_file_count
    }
}
