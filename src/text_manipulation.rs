use cosmic_text::{
    fontdb::Database, Attrs, BorrowedWithFontSystem, Buffer, FontSystem, Metrics, Shaping, Wrap,
};

/// Split txt into lines
pub fn split_into_lines_cosmic<'a>(
	txt: &str,
	buffer: &mut BorrowedWithFontSystem<'a, Buffer>,
) -> Vec<String> {
	let mut lines = vec![];
	let attrs = Attrs::new();

	buffer.set_text(txt, attrs, Shaping::Basic);
	let x = buffer.line_layout(0).unwrap();
	let mut chars_iterator = txt.chars();
	for w in x {
			let mut line = String::new();
			let chunk_size = w.glyphs.len();
			let mut i = 0;
			for ch in &mut chars_iterator {
					i += 1;
					line.push(ch);
					if i >= chunk_size {
							break;
					}
			}
			lines.push(line)
	}
	lines
}

pub struct TextWrapper {
    buffer: Buffer,
    font_system: FontSystem,
}

impl TextWrapper {
    pub fn font_bytes_to_font_system(bytes: &[u8]) -> FontSystem {
        let mut db = Database::new();
        db.load_font_data(bytes.to_vec());
        let font_system = cosmic_text::FontSystem::new_with_locale_and_db("asd".to_string(), db);
        font_system
    }
    pub fn new(mut buffer: Buffer, mut font_system: FontSystem, wrapping: Wrap) -> Self {
        buffer.set_wrap(&mut font_system, wrapping);
        Self {
            buffer,
            font_system,
        }
    }
    pub fn set_metrics(&mut self, metrics: Metrics) {
        self.buffer.set_metrics(&mut self.font_system, metrics);
    }
    pub fn split_into_lines(&mut self, txt: &str) -> Vec<String> {
        let borrowed = &mut self.buffer.borrow_with(&mut self.font_system);
        split_into_lines_cosmic(txt, borrowed)
    }
}
