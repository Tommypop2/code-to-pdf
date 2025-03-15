use std::collections::HashMap;

use fontdue::{Font, FontSettings};

pub fn split_into_lines_fontdue(
    txt: &str,
    font: &Font,
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
                let width = font.rasterize(ch, 12.0).0.advance_width;
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

pub struct TextWrapper {
    rasterize_cache: HashMap<char, f32>,
    font: Font,
}

impl TextWrapper {
    pub fn new(font_bytes: &[u8]) -> Self {
        Self {
            rasterize_cache: HashMap::new(),
            font: Font::from_bytes(font_bytes, FontSettings::default()).unwrap(),
        }
    }
    pub fn split_into_lines(&mut self, txt: &str) -> Vec<String> {
        split_into_lines_fontdue(
            txt,
            &self.font,
            printpdf::Mm(210.0 - (10.0 + 10.0)).into_pt().0,
            &mut self.rasterize_cache,
        )
    }
}
