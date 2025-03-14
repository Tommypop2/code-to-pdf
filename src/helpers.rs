use core::str;
use std::path::PathBuf;

use printpdf::{FontId, Mm, Op, Point, Pt, TextItem, TextMatrix, TextRenderingMode};
fn index_close_to_chunk(slice: &str, i: usize, chunk_size: usize) -> (&str, usize) {
    let mut actual_chunk_size: usize = chunk_size;
    loop {
        match slice.get(i..(i + actual_chunk_size)) {
            Some(s) => return (s, actual_chunk_size),
            None => {}
        };
        actual_chunk_size -= 1;
    }
}
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
    ]
}
