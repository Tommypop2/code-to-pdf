//! Miscellaneous helper functions

use std::path::Path;

use printpdf::{FontId, Mm, Op, Point, Pt, TextItem, TextMatrix, TextRenderingMode};

use crate::{dimensions::Dimensions, text_manipulation::TextWrapper};

/// Generates a new page with basic contents
pub fn init_page(
    contents: &mut Vec<Op>,
    page_dimensions: &Dimensions,
    font_id: FontId,
    font_size: f32,
    path: &Path,
    additional_text: Option<&str>,
    wrapper: &mut TextWrapper,
) {
    contents.extend_from_slice(&[
        Op::SetLineHeight {
            lh: Pt(font_size * 1.2),
        },
        Op::SetFontSize {
            size: Pt(font_size),
            font: font_id.clone(),
        },
    ]);
    let mut additional_text_width = 0.0;
    // Write additional text
    if let Some(text) = additional_text {
        for line in text.lines() {
            let line_width = wrapper.get_width(line).0;
            if line_width > additional_text_width {
                additional_text_width = line_width
            }
        }
        contents.extend_from_slice(&[
            Op::SetTextMatrix {
                matrix: TextMatrix::Translate(Pt(0.0), Pt(0.0)),
            },
            Op::SetTextCursor {
                pos: Point {
                    x: (page_dimensions.width - page_dimensions.margin_right).into_pt()
                        - Pt(additional_text_width),
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
    for (line, _) in wrapper.split_into_lines(&path.display().to_string(), |_| {
        (page_dimensions.max_text_width()
            - if additional_text.is_some() {
                Mm::from(Pt(additional_text_width)) + Mm(5.0)
            } else {
                Mm(0.0)
            })
        .into_pt()
    }) {
        contents.push(Op::WriteText {
            items: vec![TextItem::Text(line)],
            font: font_id.clone(),
        });
        contents.push(Op::AddLineBreak);
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
