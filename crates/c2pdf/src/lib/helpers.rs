//! Miscellaneous helper functions

use std::{path::Path, str::Lines};

use printpdf::{FontId, Mm, Op, Point, Pt, TextItem, TextMatrix, TextRenderingMode};

use crate::{dimensions::Dimensions, text_manipulation::TextWrapper};
/// Processed additional text.
///
/// Pretty much just serves to cache the maximum line width within the text so it doesn't have to be recalculated
#[derive(Clone)]
pub struct ProcessedText {
  text: String,
  width: f32,
}
impl ProcessedText {
  /// Creates a new instance of [`ProcessedText`]
  pub fn new(text: String, wrapper: &mut TextWrapper) -> Option<Self> {
    let width = text
      .lines()
      .map(|line| wrapper.get_width(line).0)
      .reduce(f32::max)?;
    Some(Self { text, width })
  }
  fn lines(&self) -> Lines {
    self.text.lines()
  }
}
/// Generates a new page with basic contents
pub fn init_page(
  contents: &mut Vec<Op>,
  page_dimensions: &Dimensions,
  font_id: FontId,
  font_size: f32,
  path: &Path,
  additional_text: Option<&ProcessedText>,
  include_path: bool,
  wrapper: &mut TextWrapper,
) {
  contents.extend_from_slice(&[
    Op::StartTextSection,
    Op::SetLineHeight {
      lh: Pt(font_size * 1.2),
    },
    Op::SetFontSize {
      size: Pt(font_size),
      font: font_id.clone(),
    },
  ]);
  // Write additional text
  if let Some(text) = additional_text {
    contents.extend_from_slice(&[
      Op::SetTextMatrix {
        matrix: TextMatrix::Translate(Pt(0.0), Pt(0.0)),
      },
      Op::SetTextCursor {
        pos: Point {
          x: (page_dimensions.width - page_dimensions.margin_right).into_pt() - Pt(text.width),
          y: (page_dimensions.height - Mm(7.5)).into(),
        },
      },
    ]);
    for line in text.lines() {
      contents.push(Op::WriteText {
        items: vec![TextItem::Text(line.to_string())],
        font: font_id.clone(),
      });
      contents.push(Op::AddLineBreak);
    }
  }
  if include_path {
    contents.extend_from_slice(&[
      Op::SetTextMatrix {
        matrix: TextMatrix::Translate(Pt(0.0), Pt(0.0)),
      },
      // Write metadata
      Op::SetTextCursor {
        pos: Point {
          x: page_dimensions.margin_left.into(),
          y: (page_dimensions.height - Mm(7.5)).into(),
        },
      },
    ]);
    let max_path_width = (page_dimensions.max_text_width()
      - if let Some(text) = &additional_text {
        Mm::from(Pt(text.width)) + Mm(5.0)
      } else {
        Mm(0.0)
      })
    .into_pt();
    for (line, _) in wrapper.split_into_lines(&path.display().to_string(), |_| max_path_width) {
      contents.push(Op::WriteText {
        items: vec![TextItem::Text(line)],
        font: font_id.clone(),
      });
      contents.push(Op::AddLineBreak);
    }
  }

  // Set cursor to main body
  contents.extend_from_slice(&[
    // This allows me to reset the text cursor for some reason
    Op::SetTextMatrix {
      matrix: TextMatrix::Translate(Pt(0.0), Pt(0.0)),
    },
    Op::SetTextCursor {
      pos: Point {
        x: page_dimensions.margin_left.into(),
        y: (page_dimensions.height - page_dimensions.margin_top).into(),
      },
    },
    Op::SetTextRenderingMode {
      mode: TextRenderingMode::Fill,
    },
  ]);
}
