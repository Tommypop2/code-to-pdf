//! Primitives for wrapping text

use std::collections::HashMap;

use fontdue::{Font, FontSettings};
use printpdf::Mm;

/// Uses the [`fontdue`] text rasterizer to split text into lines less than the `max_width`
pub fn split_into_lines_fontdue(
    txt: &str,
    font: &Font,
    font_size: f32,
    max_width: f32,
    cache: &mut std::collections::HashMap<char, f32>,
) -> Vec<String> {
    let mut lines: Vec<String> = vec![];
    let mut line_buf = String::new();
    let mut current_line_width = 0.0;
    for ch in txt.chars() {
        let width = match cache.get(&ch) {
            Some(w) => *w,
            None => {
                let width = font.rasterize(ch, font_size).0.advance_width;
                cache.insert(ch, width);
                width
            }
        };
        // Move onto new line if width exceeds maximum, or if we're close to the maximum and find a space
        if current_line_width + width >= max_width
            || (max_width - (current_line_width + width) < 5.0) && ch.is_whitespace()
        {
            lines.push(line_buf.trim().to_string());
            line_buf.clear();
            current_line_width = 0.0;
        }
        line_buf.push(ch);
        current_line_width += width
    }
    lines.push(line_buf);
    lines
}

/// Handles wrapping text into multiple lines
pub struct TextWrapper {
    rasterize_cache: HashMap<char, f32>,
    font: Font,
    font_size: f32,
}

impl TextWrapper {
    /// Initialises new [`TextWrapper`] from `font_bytes`, and `font_size`
    pub fn new(font_bytes: &[u8], font_size: f32) -> Self {
        Self {
            rasterize_cache: HashMap::new(),
            font: Font::from_bytes(font_bytes, FontSettings::default()).unwrap(),
            font_size,
        }
    }
    /// Splits a given &[`str`] into a [`Vec<String>`] of lines not exceeding the `max_width` set
    pub fn split_into_lines(&mut self, txt: &str, max_width: Mm) -> Vec<String> {
        split_into_lines_fontdue(
            txt,
            &self.font,
            self.font_size,
            max_width.into_pt().0,
            &mut self.rasterize_cache,
        )
    }

    /// Returns the width of a given string
    pub fn get_width(&mut self, txt: &str) -> f32 {
        let mut total_width = 0.0;
        for ch in txt.chars() {
            let char_width = match self.rasterize_cache.get(&ch) {
                Some(w) => *w,
                None => {
                    let width = self.font.rasterize(ch, self.font_size).0.advance_width;
                    self.rasterize_cache.insert(ch, width);
                    width
                }
            };
            total_width += char_width;
        }
        total_width
    }
    /// Returns the set `font_size`
    pub fn font_size(&self) -> f32 {
        self.font_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FONT_BYTES: &[u8] = include_bytes!("../fonts/Helvetica.ttf") as &[u8];
    const TEXT: &str = "Hello World!! This is a vaguely long string to test string splitting!";
    #[test]
    fn splitting_lines() {
        let result = split_into_lines_fontdue(
            TEXT,
            &Font::from_bytes(FONT_BYTES, FontSettings::default()).unwrap(),
            20.0,
            100.0,
            &mut HashMap::new(),
        );
        assert_eq!(result.len(), 6);
        // Check that joining back together creates the original string (spaces are trimmed so doesn't matter if these aren't retained)
        assert_eq!(result.join("").replace(' ', ""), TEXT.replace(' ', ""));
    }
}
