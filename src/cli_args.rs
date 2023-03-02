use clap::Parser;
use genpdf::Mm;

use crate::base_style::DocumentStyle;

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

    #[arg(long)]
    pub print_ast: bool,
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

        style
    }
}
