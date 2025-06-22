//! Primitives for wrapping text

use std::collections::HashMap;

use fontdue::{Font, FontSettings};
use printpdf::Pt;

/// Uses the [`fontdue`] text rasterizer to split text into lines less than the `max_width`
pub fn split_into_lines_fontdue<F: Fn(usize) -> Pt>(
  txt: &str,
  font: &Font,
  font_size: f32,
  max_width: F,
  cache: &mut std::collections::HashMap<char, f32>,
) -> Vec<(String, f32)> {
  let mut lines: Vec<(String, f32)> = vec![];
  let mut line_buf = String::new();
  let mut current_line_width = 0.0;
  // Stores the max line width for the current line (may be different depending on what line we're on)
  let mut max_line_width = max_width(0).0;
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
    if (current_line_width + width >= max_line_width)
      || ((max_line_width - (current_line_width + width) < 30.0) && ch.is_whitespace())
    {
      // Push this character so we know that the new line was due to line splitting
      line_buf.push('\n');
      lines.push((line_buf.trim_start().to_string(), current_line_width));
      // Retrieve new line width for the next line
      max_line_width = max_width(lines.len()).0;
      line_buf.clear();
      current_line_width = 0.0;
    }
    line_buf.push(ch);
    current_line_width += width;
  }
  lines.push((line_buf.trim().to_string(), current_line_width));
  lines
}

/// Handles wrapping text into multiple lines
#[derive(Clone)]
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
  pub fn split_into_lines<T: Fn(usize) -> Pt>(
    &mut self,
    txt: &str,
    max_width: T,
  ) -> Vec<(String, f32)> {
    split_into_lines_fontdue(
      txt,
      &self.font,
      self.font_size,
      max_width,
      &mut self.rasterize_cache,
    )
  }

  /// Returns the width of a given string in Point
  pub fn get_width(&mut self, txt: &str) -> Pt {
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
    Pt(total_width)
  }
  /// Returns the set `font_size`
  pub fn font_size(&self) -> f32 {
    self.font_size
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  const FONT_BYTES: &[u8] = include_bytes!("./fonts/Helvetica.ttf") as &[u8];
  const TEXT: &str = "Hello World!! This is a vaguely long string to test string splitting!";
  #[test]
  fn splitting_lines() {
    let result = split_into_lines_fontdue(
      TEXT,
      &Font::from_bytes(FONT_BYTES, FontSettings::default()).unwrap(),
      20.0,
      |_| Pt(100.0),
      &mut HashMap::new(),
    );
    assert_eq!(result.len(), 7);
    // Check that joining back together creates the original string (spaces are trimmed so doesn't matter if these aren't retained)
    assert_eq!(
      result
        .iter()
        .map(|x| x.0.clone())
        .collect::<Vec<String>>()
        .join("")
        .replace([' ', '\n'], ""),
      TEXT.replace(' ', "")
    );
  }
}
