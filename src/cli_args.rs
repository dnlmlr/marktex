use clap::{Parser, ValueEnum};
use genpdf::Mm;
use hyphenation::{Load, Standard};

use crate::{base_style::DocumentStyle, resources};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ArgHyphenationLang {
    /// German language hyphenation rules
    De,
    /// English (US) language hyphenation rules
    En,
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct CliArgs {
    /// Path to the input markdown file
    #[arg()]
    pub input: String,

    /// Path to the output PDF file
    #[arg()]
    pub output: String,

    /// PDF file title
    #[arg(long)]
    pub title: Option<String>,

    /// Page margin left in mm
    #[arg(long)]
    pub margin_left: Option<f64>,

    /// Page margin right in mm
    #[arg(long)]
    pub margin_right: Option<f64>,

    /// Page margin top in mm
    #[arg(long)]
    pub margin_top: Option<f64>,

    /// Page margin bottom in mm
    #[arg(long)]
    pub margin_bottom: Option<f64>,

    /// Base fontsize for the text
    #[arg(long)]
    pub font_size: Option<u8>,

    /// What language to use for hyphenation. Default is no hyphenation
    #[arg(long, value_enum)]
    pub hyphenation: Option<ArgHyphenationLang>,

    /// Print the parsed markdown nodes during mapping
    #[arg(long)]
    pub print_ast: bool,

    /// By default font-subsetting is used to remove unused glyphs from the embedded fonts in order
    /// to reduce the PDF file size. Setting this flag disables the subsetting, increasing the PDF
    /// file size drastically. Currently this doesn't actually chatch all unused glyphs, so there is
    /// room to improve.
    #[arg(long)]
    pub disable_font_subsetting: bool,
}

impl From<&CliArgs> for DocumentStyle {
    fn from(value: &CliArgs) -> Self {
        let mut style = Self::default();

        if let Some(title) = &value.title {
            style.title = title.clone();
        }

        if let Some(font_size) = value.font_size {
            style.text_size = font_size;
        }

        if let Some(margin_left) = value.margin_left {
            style.page_margins.left = Mm(margin_left);
        }
        if let Some(margin_right) = value.margin_right {
            style.page_margins.right = Mm(margin_right);
        }
        if let Some(margin_top) = value.margin_top {
            style.page_margins.top = Mm(margin_top);
        }
        if let Some(margin_bottom) = value.margin_bottom {
            style.page_margins.bottom = Mm(margin_bottom);
        }

        let dict_de = resources::get_decompress(resources::HYP_DE1996);
        let dict_en = resources::get_decompress(resources::HYP_EN_US);

        use hyphenation::Language::{EnglishUS, German1996};
        style.hyphenation = value.hyphenation.map(|hyp| match hyp {
            ArgHyphenationLang::De => {
                Standard::from_reader(German1996, &mut dict_de.as_slice()).unwrap()
            }
            ArgHyphenationLang::En => {
                Standard::from_reader(EnglishUS, &mut dict_en.as_slice()).unwrap()
            }
        });

        style
    }
}
