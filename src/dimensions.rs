use printpdf::Mm;

pub struct Dimensions {
    pub width: Mm,
    pub height: Mm,
    pub margin_left: Mm,
    pub margin_right: Mm,
    pub margin_top: Mm,
    pub margin_bottom: Mm,
}
impl Default for Dimensions {
    /// Initialises a default `Dimensions`.
    /// Default document size is A4
    fn default() -> Self {
        Self {
            width: Mm(210.0),
            height: Mm(297.0),
            margin_top: Mm(20.0),
            margin_bottom: Mm(5.0),
            margin_left: Mm(10.0),
            margin_right: Mm(10.0),
        }
    }
}
impl Dimensions {
    pub fn new_default_margins(width: Mm, height: Mm) -> Self {
        Self {
            width,
            height,
            ..Default::default()
        }
    }
    pub fn new(
        width: Mm,
        height: Mm,
        margin_top: Mm,
        margin_bottom: Mm,
        margin_left: Mm,
        margin_right: Mm,
    ) -> Self {
        Self {
            width,
            height,
            margin_left,
            margin_right,
            margin_top,
            margin_bottom,
        }
    }
    pub fn max_text_width(&self) -> Mm {
        self.width - self.margin_left - self.margin_right
    }
    pub fn max_text_height(&self) -> Mm {
        self.height - self.margin_top - self.margin_bottom
    }
}
