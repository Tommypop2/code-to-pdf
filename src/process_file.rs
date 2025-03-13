use core::str;
use std::path::PathBuf;

use printpdf::{
    FontId, Mm, Op, Point, Pt, TextItem, TextMatrix, TextRenderingMode,
};
pub fn split_into_chunks(slice: &str, chunk_size: usize) -> Vec<&str> {
    let mut v = vec![];
    let mut i = 0;
    while (i + chunk_size) <= slice.len() {
        v.push(&slice[i..(i + chunk_size)]);
        i += chunk_size;
    }
    v.push(&slice[i..slice.len()]);
    v
}
pub fn new_page_contents(page_dimensions: (f32, f32), font_id: FontId, path: PathBuf) -> Vec<Op> {
    vec![
        Op::SetLineHeight { lh: Pt(14.0) },
        Op::SetFontSize {
            size: Pt(12.0),
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
            items: vec![TextItem::Text(path.to_str().unwrap().to_owned())],
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
    ]
}