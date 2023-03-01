mod base_style;
mod cli_args;

use std::{fs::File, io::BufReader, path::Path};

use clap::Parser;
use comrak::{arena_tree::NodeEdge, nodes::NodeValue, Arena};
use genpdf::{
    elements::{Image, PaddedElement, Paragraph, UnorderedList},
    style::{Color, Style},
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

fn main() {
    // Cli Parsing and base style setup
    let cli_args = CliArgs::parse();
    let docstyle = DocumentStyle::from(&cli_args);

    // PDF document setup
    let font = genpdf::fonts::from_files("fonts", "DroidSerif", None).unwrap();
    let mut doc = genpdf::Document::new(font);
    docstyle.apply_base_style(&mut doc);

    // Markdown parsing
    let arena = Arena::new();
    let md = std::fs::read_to_string(&cli_args.input).expect("Can't read input file");
    let opts = comrak::ComrakOptions::default();
    let md_ast = comrak::parse_document(&arena, &md, &opts);

    let mut stylestack = FormatStack::new(Style::default());

    // Markdown AST traversal to create matching PDF outputs to the markdown elements
    for node_edge in md_ast.traverse() {
        let (node, start) = match node_edge {
            NodeEdge::Start(it) => (&it.data, true),
            NodeEdge::End(it) => (&it.data, false),
        };
        let node = &node.borrow().value;

        // Debug prints for the AST Nodes
        match start {
            true => print!("START: "),
            false => print!("END: "),
        }
        match &node {
            NodeValue::Text(t) => println!("Text({})", String::from_utf8_lossy(t)),
            it => println!("{:?}", it),
        }

        match (start, node) {
                (true, NodeValue::Paragraph) => {
                    let mut p = Paragraph::default();
                    if docstyle.align_justify {
                        p.set_alignment(Alignment::Justified);
                    }
                    stylestack.push_paragraph(p);
                }
                (true, NodeValue::Heading(h)) => {
                    stylestack.push_style(|s| {
                        let font_size = docstyle.get_header_size(h.level);
                        s.set_font_size(font_size);
                        s.set_bold();
                    });
                    stylestack.push_paragraph(Paragraph::default());
                }
                (true, NodeValue::Text(t)) => {
                    let t = String::from_utf8_lossy(t);
                    let style = stylestack.get_style();
                    stylestack.get_paragraph_mut().push_styled(t, style);
                }
                (true, NodeValue::Emph) => {
                    stylestack.push_style(|s| {
                        s.set_italic();
                    });
                }
                (true, NodeValue::Strong) => {
                    stylestack.push_style(|s| {
                        s.set_bold();
                    });
                }
                (true, NodeValue::List(_lst)) => {
                    stylestack.push_list(UnorderedList::new());
                }
                (true, NodeValue::BlockQuote) => {
                    stylestack.push_style(|s| {
                        s.set_color(Color::Rgb(40, 60, 60));
                        s.set_italic();
                    });
                    stylestack.blockquote_active = true;
                }
                (true, NodeValue::Image(node_img)) => {
                    let path = String::from_utf8_lossy(&node_img.url);

                    let mut scale_x = 1.0;
                    let mut scale_y = 1.0;
                    let mut rotation = 0.0;

                    // Title is abused for metadata
                    let title = String::from_utf8_lossy(&node_img.title);
                    let props = title.split(',');
                    for prop in props {
                        let mut key_value = prop.split('=');
                        let key = key_value.next();
                        let value = key_value.next();

                        match (key, value) {
                            (Some(key), Some(value)) => {
                                match key.trim() {
                                    "scale" => match value.trim().parse() {
                                        Ok(value) => {
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
                            doc.push(img);
                        }
                        _ => {
                            eprintln!("Error loading image: {}", String::from_utf8_lossy(&node_img.url));
                        }
                    }
                }
                (true, NodeValue::LineBreak) => {
                    doc.push(PaddedElement::new(
                        stylestack.pop_paragraph(),
                        Margins::trbl(0, 0, docstyle.paragraph_spacing, 0),
                    ));

                    let mut p = Paragraph::default();
                    if docstyle.align_justify {
                        p.set_alignment(Alignment::Justified);
                    }
                    stylestack.push_paragraph(p);
                }
                (true, NodeValue::SoftBreak) => {
                    let style = stylestack.get_style();
                    stylestack.get_paragraph_mut().push_styled(" ", style);
                }

                (false, NodeValue::Paragraph) => {
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
                (false, NodeValue::Heading(_)) => {
                    doc.push(PaddedElement::new(
                        stylestack.pop_paragraph(),
                        Margins::trbl(0, 0, docstyle.header_spacing, 0),
                    ));
                    stylestack.pop_style();
                }
                (false, NodeValue::Emph | NodeValue::Strong) => {
                    stylestack.pop_style();
                }
                (false, NodeValue::BlockQuote) => {
                    stylestack.pop_style();
                    stylestack.blockquote_active = false;
                }
                (false, NodeValue::List(_lst)) => {
                    let list = stylestack.pop_list();

                    match stylestack.has_list() {
                        true => {
                            stylestack.get_list_mut().push_no_bullet(list);
                        }
                        false => doc.push(list),
                    }
                }



                (false, NodeValue::SoftBreak) => {
                    // SoftBreak is applied at Start(SoftBreak), nothing to do here
                }
                (false, NodeValue::LineBreak) => {
                    // LineBreak is applied at Start(LineBreak), nothing to do here
                }
                (false, NodeValue::Text(_)) => {
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
