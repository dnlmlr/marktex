mod base_style;
mod cli_args;
mod font_subset;

use std::{fs::File, io::BufReader, path::Path};

use clap::Parser;
use comrak::{arena_tree::NodeEdge, nodes::NodeValue, Arena};
use font_subset::font_subset;
use genpdf::{
    elements::{Image, PaddedElement, PageBreak, Paragraph, UnorderedList, Math},
    fonts::FontData,
    style::{Color, Style, StyledString},
    Alignment, Margins, Scale,
};

use crate::{base_style::DocumentStyle, cli_args::CliArgs};

struct FormatStack {
    styles: Vec<Style>,
    paragraphs: Vec<Paragraph>,
    lists: Vec<UnorderedList>,
    blockquote_active: bool,
}

impl FormatStack {
    pub fn new(default_style: Style) -> Self {
        Self {
            styles: vec![default_style],
            paragraphs: Vec::new(),
            lists: Vec::new(),
            blockquote_active: false,
        }
    }

    pub fn push_style(&mut self, m: impl Fn(&mut Style)) {
        let mut new_style = self.styles.last().unwrap().clone();
        m(&mut new_style);
        self.styles.push(new_style);
    }

    pub fn pop_style(&mut self) {
        self.styles.pop();
    }

    pub fn get_style(&self) -> Style {
        self.styles.last().unwrap().clone()
    }

    pub fn push_paragraph(&mut self, p: Paragraph) {
        self.paragraphs.push(p);
    }

    pub fn pop_paragraph(&mut self) -> Paragraph {
        self.paragraphs.pop().unwrap()
    }

    pub fn get_paragraph_mut(&mut self) -> &mut Paragraph {
        self.paragraphs.last_mut().unwrap()
    }

    pub fn push_list(&mut self, p: UnorderedList) {
        self.lists.push(p);
    }

    pub fn pop_list(&mut self) -> UnorderedList {
        self.lists.pop().unwrap()
    }

    pub fn has_list(&self) -> bool {
        !self.lists.is_empty()
    }

    pub fn get_list_mut(&mut self) -> &mut UnorderedList {
        self.lists.last_mut().unwrap()
    }
}

enum NodeStartEnd {
    Start,
    End,
}

const EMBEDDED_DEFAULT_FONT: [&[u8]; 4] = [
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/TeX-Gyre-Pagella/texgyrepagella-regular.otf")),
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/TeX-Gyre-Pagella/texgyrepagella-bold.otf")),
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/TeX-Gyre-Pagella/texgyrepagella-italic.otf")),
    include_bytes!(concat!(env!("OUT_DIR"), "/fonts/TeX-Gyre-Pagella/texgyrepagella-bolditalic.otf")),
];

fn make_font_family(data: &[u8]) -> genpdf::fonts::FontFamily<FontData> {
    let font = genpdf::fonts::FontData::new(data.to_vec(), None).unwrap();
    genpdf::fonts::FontFamily {
        regular: font.clone(),
        bold: font.clone(),
        italic: font.clone(),
        bold_italic: font,
    }
}

fn main() {
    // Cli Parsing and base style setup
    let cli_args = CliArgs::parse();
    let docstyle = DocumentStyle::from(&cli_args);

    let md = std::fs::read_to_string(&cli_args.input).expect("Can't read input file");

    // PDF document setup
    let [regular, bold, italic, bold_italic] = EMBEDDED_DEFAULT_FONT.map(|font_raw| {
        let font_raw = zstd::decode_all(font_raw).unwrap();
        if cli_args.disable_font_subsetting {
            font_raw.to_vec()
        } else {
            font_subset(&font_raw, &md).unwrap()
        }
    });

    let font = genpdf::fonts::FontFamily {
        regular: FontData::new(regular, None).unwrap(),
        bold: FontData::new(bold, None).unwrap(),
        italic: FontData::new(italic, None).unwrap(),
        bold_italic: FontData::new(bold_italic, None).unwrap(),
    };

    let font_raw: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/fonts/TeX-Gyre-Pagella/texgyrepagella-math.otf"));
    let font_raw = zstd::decode_all(font_raw).unwrap();
    let math_font_family = make_font_family(&font_raw);

    let mut doc = genpdf::Document::new(font);
    doc.set_minimal_conformance();
    let math_font_family = doc.add_font_family(math_font_family);
    doc.enable_math(&font_raw, math_font_family);
    docstyle.apply_base_style(&mut doc);

    // Markdown parsing
    let arena = Arena::new();
    let mut opts = comrak::ComrakOptions::default();
    // opts.extension.tasklist = true;
    opts.extension.strikethrough = true;
    let md_ast = comrak::parse_document(&arena, &md, &opts);

    let mut stylestack = FormatStack::new(Style::default());

    // Markdown AST traversal to create matching PDF outputs to the markdown elements
    for node_edge in md_ast.traverse() {
        use NodeStartEnd::{End, Start};

        let (ast_node, start) = match node_edge {
            NodeEdge::Start(it) => (&it.data, Start),
            NodeEdge::End(it) => (&it.data, End),
        };
        let node = &ast_node.borrow().value;

        // Debug prints for the AST Nodes
        if cli_args.print_ast {
            match start {
                Start => print!("START: "),
                End => print!("END: "),
            }
            match &node {
                NodeValue::Text(t) => println!("Text({})", String::from_utf8_lossy(t)),
                it => println!("{:?}", it),
            }
        }

        match (start, node) {
                (Start, NodeValue::Paragraph) => {
                    let mut p = Paragraph::default();
                    if docstyle.align_justify {
                        p.set_alignment(Alignment::Justified(true));
                    }
                    stylestack.push_paragraph(p);
                }
                (Start, NodeValue::Heading(h)) => {
                    stylestack.push_style(|s| {
                        let font_size = docstyle.get_header_size(h.level);
                        s.set_font_size(font_size);
                        s.set_bold();
                    });
                    stylestack.push_paragraph(Paragraph::default());
                }
                (Start, NodeValue::Text(t)) => {
                    let t = String::from_utf8_lossy(t);
                    let style = stylestack.get_style();
                    stylestack.get_paragraph_mut().push_styled(t, style);
                }
                (Start, NodeValue::Emph) => {
                    stylestack.push_style(|s| {
                        s.set_italic();
                    });
                }
                (Start, NodeValue::Strong) => {
                    stylestack.push_style(|s| {
                        s.set_bold();
                    });
                }
                (Start, NodeValue::Strikethrough) => {
                    stylestack.push_style(|s| {
                        s.set_strikethrough();
                    });
                }
                (Start, NodeValue::List(_lst)) => {
                    stylestack.push_list(UnorderedList::new());
                }
                (Start, NodeValue::BlockQuote) => {
                    stylestack.push_style(|s| {
                        s.set_color(Color::Rgb(40, 60, 60));
                        s.set_italic();
                    });
                    stylestack.blockquote_active = true;
                }
                (Start, NodeValue::Image(node_img)) => {
                    let path = String::from_utf8_lossy(&node_img.url);

                    let mut scale_x = 1.0;
                    let mut scale_y = 1.0;
                    let mut rotation = 0.0;

                    // Title is abused for metadata
                    let title = String::from_utf8_lossy(&node_img.title);
                    let props = title.split(',');
                    for prop in props.filter(|it| !it.trim().is_empty()) {
                        let mut key_value = prop.split('=');
                        let key = key_value.next();
                        let value = key_value.next();

                        match (key, value) {
                            (Some(key), Some(value)) => {
                                match key.trim() {
                                    "scale" => match value.trim().parse() {
                                        Ok(value) => {
                                            println!("Scale value {}", value);
                                            scale_x = value;
                                            scale_y = value;
                                        }
                                        Err(_) => eprintln!("Failed to parse '{}' as scale value", value),
                                    }
                                    "scale-x" => match value.trim().parse() {
                                        Ok(value) => {
                                            scale_x = value;
                                        }
                                        Err(_) => eprintln!("Failed to parse '{}' as scale value", value),
                                    }
                                    "scale-y" => match value.trim().parse() {
                                        Ok(value) => {
                                            scale_y = value;
                                        }
                                        Err(_) => eprintln!("Failed to parse '{}' as scale value", value),
                                    }
                                    "rotate" => match value.trim().parse() {
                                        Ok(value) => rotation = value,
                                        Err(_) => eprintln!("Failed to parse '{}' as rotate value", value),
                                    }
                                    _ => ()
                                }
                            }
                            _ => {
                                eprintln!(
                                    "Failed to parse key value props from image title: '{}'", 
                                    title
                                );
                            }
                        }
                    }

                    match File::open(Path::new(path.as_ref())).map(|reader| Image::from_reader(BufReader::new(reader))) {
                        Ok(Ok(mut img)) => {
                            img.set_scale(Scale::new(scale_x, scale_y));
                            img.set_alignment(Alignment::Center);
                            img.set_clockwise_rotation(rotation);
                            img.set_fit_width(true);
                            doc.push(PaddedElement::new(
                                img, 
                                Margins::trbl(0, 0, docstyle.paragraph_spacing, 0)
                            ));
                        }
                        _ => {
                            eprintln!("Error loading image: {}", String::from_utf8_lossy(&node_img.url));
                        }
                    }
                }
                (Start, NodeValue::LineBreak) => {
                    doc.push(PaddedElement::new(
                        stylestack.pop_paragraph(),
                        Margins::trbl(0, 0, docstyle.paragraph_spacing, 0),
                    ));

                    let mut p = Paragraph::default();
                    if docstyle.align_justify {
                        p.set_alignment(Alignment::Justified(true));
                    }
                    stylestack.push_paragraph(p);
                }
                (Start, NodeValue::SoftBreak) => {
                    let style = stylestack.get_style();
                    stylestack.get_paragraph_mut().push_if_text(
                        StyledString::new(' ', style), 
                        |text| {
                            text.last().map(|w| !w.s.ends_with(' ')).unwrap_or(true)
                        }
                    );
                }
                (Start, NodeValue::ThematicBreak) => {
                    doc.push(PageBreak::new());
                }
                (Start, NodeValue::CodeBlock(cb)) => {
                    if cb.info == b"math" {
                        let math_str = String::from_utf8_lossy(&cb.literal);
                        let lines = math_str.lines();
                        let mut math_lines: Vec<String> = Vec::new();

                        // Lines separated by a fully empty line will be rendered vertically stacked
                        let mut append_to_prev = false;
                        for line in lines {
                            if line.trim().is_empty() {
                                append_to_prev = false;
                            } else {
                                if append_to_prev {
                                    math_lines.last_mut().unwrap().push_str(line);
                                } else {
                                    math_lines.push(line.to_string());
                                }
                                append_to_prev = true;
                            }
                        }

                        for math in math_lines {
                            let mut math_block = match Math::new(&math) {
                                Ok(it) => it,
                                Err(e) => {
                                    eprintln!(
                                        "Error while parsing math block in line ({}): {}", 
                                        ast_node.borrow().start_line,
                                        e
                                    );
                                    eprintln!("    occured in '{}'", math);
                                    continue;
                                }
                            };
                            math_block.set_alignment(Alignment::Center);

                            doc.push(PaddedElement::new(
                                math_block, 
                                Margins::trbl(0, 0, docstyle.paragraph_spacing, 0)
                            ));
                        }
                    }
                }


                (End, NodeValue::Paragraph) => {
                    let new_elem = stylestack.pop_paragraph();

                    match stylestack.has_list() {
                        true => stylestack.get_list_mut().push(new_elem),
                        false => {
                            if stylestack.blockquote_active {
                                // TODO: Do something to better mark block quotes
                            }
                            doc.push(PaddedElement::new(
                                new_elem,
                                Margins::trbl(0, 0, docstyle.paragraph_spacing, 0),
                            ));
                        }
                    }
                }
                (End, NodeValue::Heading(_)) => {
                    doc.push(PaddedElement::new(
                        stylestack.pop_paragraph(),
                        Margins::trbl(docstyle.header_spacing, 0, docstyle.header_spacing, 0),
                    ));
                    stylestack.pop_style();
                }
                (End, NodeValue::Emph | NodeValue::Strong | NodeValue::Strikethrough) => {
                    stylestack.pop_style();
                }
                (End, NodeValue::BlockQuote) => {
                    stylestack.pop_style();
                    stylestack.blockquote_active = false;
                }
                (End, NodeValue::List(_lst)) => {
                    let list = stylestack.pop_list();

                    match stylestack.has_list() {
                        true => {
                            stylestack.get_list_mut().push_no_bullet(list);
                        }
                        false => doc.push(PaddedElement::new(
                            list, 
                            Margins::trbl(0, 0, docstyle.paragraph_spacing, 0)
                        )),
                    }
                }



                (End, NodeValue::SoftBreak) => {
                    // SoftBreak is applied at Start(SoftBreak), nothing to do here
                }
                (End, NodeValue::LineBreak) => {
                    // LineBreak is applied at Start(LineBreak), nothing to do here
                }
                (End, NodeValue::Text(_)) => {
                    // Text is inserted at Start(Text), and commited when the paragraph ends. So
                    // Nothing to do here
                }
                (_, NodeValue::Item(_item)) => {
                    // Items automatically contain a paragraph, so don't do anything here
                }

                _ => ()
                // NodeValue::Document => todo!(),
                // NodeValue::FrontMatter(_) => todo!(),
                // NodeValue::DescriptionList => todo!(),
                // NodeValue::DescriptionItem(_) => todo!(),
                // NodeValue::DescriptionTerm => todo!(),
                // NodeValue::DescriptionDetails => todo!(),
                // NodeValue::CodeBlock(_) => todo!(),
                // NodeValue::HtmlBlock(_) => todo!(),
                // NodeValue::ThematicBreak => todo!(),
                // NodeValue::FootnoteDefinition(_) => todo!(),
                // NodeValue::Table(_) => todo!(),
                // NodeValue::TableRow(_) => todo!(),
                // NodeValue::TableCell => todo!(),
                // NodeValue::TaskItem { checked, symbol } => todo!(),
                // NodeValue::Code(_) => todo!(),
                // NodeValue::HtmlInline(_) => todo!(),
                // NodeValue::Strikethrough => todo!(),
                // NodeValue::Superscript => todo!(),
                // NodeValue::Link(_) => todo!(),
                // NodeValue::FootnoteReference(_) => todo!(),
        }
    }

    doc.render_to_file(&cli_args.output).unwrap();
}
