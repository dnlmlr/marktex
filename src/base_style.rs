use genpdf::{Document, Margins, PaperSize};
use hyphenation::Load;

#[derive(Debug, Clone)]
pub struct DocumentStyle {
    pub text_size: u8,
    pub h1_size: u8,
    pub h2_size: u8,
    pub h3_size: u8,
    pub h4_size: u8,
    pub h5_size: u8,
    pub h6_size: u8,

    pub line_spacing: f64,
    pub paragraph_spacing: f64,
    pub header_spacing: f64,

    pub paper_size: PaperSize,
    pub page_margins: Margins,

    pub align_justify: bool,
    pub hyphenation: bool,

    pub title: String,
}

impl Default for DocumentStyle {
    fn default() -> Self {
        let text_size = 10;
        Self {
            text_size,

            h1_size: (text_size as f32 * 2.5).round() as u8,
            h2_size: (text_size as f32 * 2.0).round() as u8,
            h3_size: (text_size as f32 * 1.5).round() as u8,
            h4_size: (text_size as f32 * 1.2).round() as u8,
            h5_size: (text_size as f32 * 1.0).round() as u8,
            h6_size: (text_size as f32 * 0.8).round() as u8,

            line_spacing: 1.25,
            paragraph_spacing: 1.5,
            header_spacing: 2.0,

            paper_size: PaperSize::A4,
            page_margins: Margins::trbl(30.0, 35.0, 30.0, 40.0),

            align_justify: true,
            hyphenation: true,

            title: String::new(),
        }
    }
}

impl DocumentStyle {
    pub fn get_header_size(&self, h: u8) -> u8 {
        match h {
            1 => self.h1_size,
            2 => self.h2_size,
            3 => self.h3_size,
            4 => self.h4_size,
            5 => self.h5_size,
            6 => self.h6_size,
            _ => self.text_size,
        }
    }

    pub fn apply_base_style(&self, doc: &mut Document) {
        if self.hyphenation {
            doc.set_hyphenator(
                hyphenation::Standard::from_embedded(hyphenation::Language::German1996).unwrap(),
            );
        }
        doc.set_font_size(self.text_size);
        doc.set_line_spacing(self.line_spacing);
        doc.set_paper_size(self.paper_size);
        doc.set_title(&self.title);

        let mut deco = genpdf::SimplePageDecorator::new();
        deco.set_margins(self.page_margins);
        doc.set_page_decorator(deco);
    }
}