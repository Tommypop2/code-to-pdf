use std::path::PathBuf;

use printpdf::{FontId, Mm, Op, Point, Pt, TextItem, TextMatrix, TextRenderingMode};

use crate::text_manipulation::TextWrapper;

/// Generates a new page with basic contents
pub fn init_page(
    contents: &mut Vec<Op>,
    page_dimensions: (Mm, Mm),
    font_id: FontId,
    font_size: f32,
    path: &PathBuf,
    wrapper: &mut TextWrapper,
) {
    let mut new_contents = vec![
        Op::SetLineHeight {
            lh: Pt(font_size * 1.2),
        },
        Op::SetFontSize {
            size: Pt(font_size),
            font: font_id.clone(),
        },
        // Write metadata
        Op::SetTextCursor {
            pos: Point {
                x: Mm(10.0).into(),
                y: (page_dimensions.1 - Mm(7.5)).into(),
            },
        },
    ];
    for line in wrapper.split_into_lines(&path.display().to_string()) {
        new_contents.push(Op::WriteText {
            items: vec![TextItem::Text(line)],
            font: font_id.clone(),
        });
        new_contents.push(Op::AddLineBreak);
    }
    new_contents.extend_from_slice(&[
        // This allows me to reset the text cursor for some reason
        Op::SetTextMatrix {
            matrix: TextMatrix::Translate(Pt(0.0), Pt(0.0)),
        },
        Op::SetTextCursor {
            pos: Point {
                x: Mm(10.0).into(),
                y: (page_dimensions.1 - Mm(20.0)).into(),
            },
        },
        Op::SetTextRenderingMode {
            mode: TextRenderingMode::Fill,
        },
    ]);
    contents.extend(new_contents);
}
