use std::path::PathBuf;

use printpdf::{FontId, Mm, Op, Point, Pt, TextItem, TextMatrix, TextRenderingMode};

/// Generates a new page with basic contents
pub fn init_page(
    contents: &mut Vec<Op>,
    page_dimensions: (f32, f32),
    font_id: FontId,
    font_size: f32,
    path: PathBuf,
) {
    contents.extend_from_slice(&[
        Op::SetLineHeight { lh: Pt(14.0) },
        Op::SetFontSize {
            size: Pt(font_size),
            font: font_id.clone(),
        },
        // Write metadata
        Op::SetTextCursor {
            pos: Point {
                x: Mm(10.0).into(),
                y: Mm(page_dimensions.1 - 7.5).into(),
            },
        },
        Op::WriteText {
            items: vec![TextItem::Text(path.display().to_string())],
            font: font_id.clone(),
        },
        // This allows me to reset the text cursor for some reason
        Op::SetTextMatrix {
            matrix: TextMatrix::Translate(Pt(0.0), Pt(0.0)),
        },
        Op::SetTextCursor {
            pos: Point {
                x: Mm(10.0).into(),
                y: Mm(page_dimensions.1 - 20.0).into(),
            },
        },
        Op::SetTextRenderingMode {
            mode: TextRenderingMode::Fill,
        },
    ]);
}
