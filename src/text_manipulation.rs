use std::collections::HashMap;

use fontdue::{Font, FontSettings};

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

pub struct TextWrapper {
    rasterize_cache: HashMap<char, f32>,
    font: Font,
    font_size: f32,
}

impl TextWrapper {
    pub fn new(font_bytes: &[u8], font_size: f32) -> Self {
        Self {
            rasterize_cache: HashMap::new(),
            font: Font::from_bytes(font_bytes, FontSettings::default()).unwrap(),
            font_size,
        }
    }
    pub fn split_into_lines(&mut self, txt: &str) -> Vec<String> {
        split_into_lines_fontdue(
            txt,
            &self.font,
            self.font_size,
            printpdf::Mm(210.0 - (10.0 + 10.0)).into_pt().0,
            &mut self.rasterize_cache,
        )
    }
    pub fn font_size(&self) -> f32 {
        self.font_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let font_bytes = include_bytes!("../fonts/Helvetica.ttf") as &[u8];
				let text = "Hello World!! This is a vaguely long string to test string splitting!";
        let result = split_into_lines_fontdue(
            text,
            &Font::from_bytes(font_bytes, FontSettings::default()).unwrap(),
            20.0,
            100.0,
            &mut HashMap::new(),
        );
				assert_eq!(result.len(), 6);
				// Check that joining back together creates the original string (spaces are trimmed so doesn't matter if these aren't retained)
				assert_eq!(result.join("").replace(' ', ""), text.replace(' ', ""));
    }
}
