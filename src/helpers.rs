use core::str;
use std::path::PathBuf;

use printpdf::{FontId, Mm, Op, Point, Pt, TextItem, TextMatrix, TextRenderingMode};

/// Slicing into a &str can slice part-way through a character, which would panic.
/// This slices into the nearest full character chunk_size given
pub fn index_close_to_chunk(slice: &str, i: usize, chunk_size: usize) -> (&str, usize) {
    let mut actual_chunk_size: usize = chunk_size;
    loop {
        if let Some(s) = slice.get(i..(i + actual_chunk_size)) {
            return (s, actual_chunk_size);
        };
        actual_chunk_size -= 1;
    }
}
/// Splits a slice into chunks
pub fn split_into_chunks(slice: &str, chunk_size: usize) -> Vec<&str> {
    let mut v = vec![];
    let mut i = 0;
    while (i + chunk_size) <= slice.len() {
        let (sub, actual_chunk_size) = index_close_to_chunk(slice, i, chunk_size);
        v.push(sub);
        i += actual_chunk_size;
    }
    v.push(&slice[i..slice.len()]);
    v
}
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
