use std::{fs::File, io::BufReader, path::Path, time::Instant};

use comrak::{arena_tree::NodeEdge, nodes::NodeValue, Arena};
use genpdf::{
    elements::{Image, PaddedElement, Paragraph, UnorderedList},
    style::{Color, Style},
    Alignment, Margins, PaperSize, Scale,
};
use hyphenation::Load;

struct FormatStack {
    stack: Vec<Style>,
}

impl FormatStack {
    pub fn new(default: Style) -> Self {
        Self {
            stack: vec![default],
        }
    }

    pub fn push_modify(&mut self, m: impl Fn(&mut Style)) {
        let mut new_style = self.stack.last().unwrap().clone();
        m(&mut new_style);
        self.stack.push(new_style);
    }

    pub fn pop(&mut self) {
        self.stack.pop();
    }

    pub fn get(&self) -> Style {
        self.stack.last().unwrap().clone()
    }
}

#[derive(Debug, Clone)]
struct DocumentStyle {
    text_size: u8,
    h1_size: u8,
    h2_size: u8,
    h3_size: u8,
    h4_size: u8,
    h5_size: u8,
    h6_size: u8,

    line_spacing: f64,
    paragraph_spacing: f64,
    header_spacing: f64,

    paper_size: PaperSize,
    page_margins: Margins,

    align_justify: bool,
    hyphenation: bool,
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
}

fn main() {
    let docstyle = DocumentStyle::default();

    let font = genpdf::fonts::from_files("fonts", "DroidSerif", None).unwrap();

    let mut doc = genpdf::Document::new(font);

    if docstyle.hyphenation {
        doc.set_hyphenator(
            hyphenation::Standard::from_embedded(hyphenation::Language::German1996).unwrap(),
        );
    }
    doc.set_font_size(docstyle.text_size);
    doc.set_line_spacing(docstyle.line_spacing);
    doc.set_paper_size(docstyle.paper_size);
    doc.set_title("My PDF File");

    let mut deco = genpdf::SimplePageDecorator::new();
    deco.set_margins(docstyle.page_margins);
    doc.set_page_decorator(deco);

    let mut stylestack = FormatStack::new(Style::default());
    let mut pstack: Vec<Paragraph> = Vec::new();
    let mut nested_stack: Vec<UnorderedList> = Vec::new();
    let mut blockquote = false;

    let arena = Arena::new();

    let md = include_str!("text.md");
    let opts = comrak::ComrakOptions::default();

    let t1 = Instant::now();

    let md_ast = comrak::parse_document(&arena, md, &opts);

    for node_edge in md_ast.traverse() {
        match node_edge {
            NodeEdge::Start(it) => match &it.data.borrow().value {
                NodeValue::Text(t) => println!("<Text({})>", String::from_utf8_lossy(t)),
                it => println!("<{:?}>", it),
            },
            NodeEdge::End(it) => match &it.data.borrow().value {
                NodeValue::Text(t) => println!("</Text({})>", String::from_utf8_lossy(t)),
                it => println!("</{:?}>", it),
            },
        }

        match node_edge {
            NodeEdge::Start(start) => match &start.data.borrow().value {
                NodeValue::Paragraph => {
                    let mut p = Paragraph::default();
                    if docstyle.align_justify {
                        p.set_alignment(Alignment::Justified);
                    }
                    pstack.push(p);
                }
                NodeValue::Heading(h) => {
                    stylestack.push_modify(|s| {
                        let font_size = docstyle.get_header_size(h.level);
                        s.set_font_size(font_size);
                        s.set_bold();
                    });
                    pstack.push(Paragraph::default());
                }
                NodeValue::Text(t) => {
                    let t = String::from_utf8_lossy(t);
                    pstack.last_mut().unwrap().push_styled(t, stylestack.get());
                }
                NodeValue::Emph => {
                    stylestack.push_modify(|s| {
                        s.set_italic();
                    });
                }
                NodeValue::Strong => {
                    stylestack.push_modify(|s| {
                        s.set_bold();
                    });
                }
                NodeValue::SoftBreak => {
                    pstack.last_mut().unwrap().push_styled(" ", stylestack.get());
                }
                NodeValue::LineBreak => {}
                NodeValue::List(_lst) => {
                    nested_stack.push(UnorderedList::new());
                }
                NodeValue::Item(_item) => {
                    // Items automatically contain a paragraph, so don't do anything here
                }
                NodeValue::BlockQuote => {
                    stylestack.push_modify(|s| {
                        s.set_color(Color::Rgb(40, 60, 60));
                        s.set_italic();
                    });
                    blockquote = true;
                }
                NodeValue::Image(img) => {
                    let path = String::from_utf8_lossy(&img.url);
                    match File::open(Path::new(path.as_ref())).map(|reader| Image::from_reader(BufReader::new(reader))) {
                        Ok(Ok(mut img)) => {
                            img.set_scale(Scale::new(0.5, 0.5));
                            doc.push(img);
                        }
                        _ => {
                            eprintln!("Error loading image: {}", String::from_utf8_lossy(&img.url));
                        }
                    }
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
            },
            NodeEdge::End(end) => match &end.data.borrow().value {
                NodeValue::Paragraph => {
                    let active_list = nested_stack.last_mut();
                    let new_elem = pstack.pop().unwrap();

                    match active_list {
                        Some(active_list) => active_list.push(new_elem),
                        None => {
                            if blockquote {
                                // TODO: Do something to better mark block quotes
                            }
                            doc.push(PaddedElement::new(
                                new_elem,
                                Margins::trbl(0, 0, docstyle.paragraph_spacing, 0),
                            ));
                        }
                    }
                }
                NodeValue::Heading(_) => {
                    doc.push(PaddedElement::new(
                        pstack.pop().unwrap(),
                        Margins::trbl(0, 0, docstyle.header_spacing, 0),
                    ));
                    stylestack.pop();
                }
                NodeValue::Text(_) => {}
                NodeValue::Emph | NodeValue::Strong => {
                    stylestack.pop();
                }
                NodeValue::BlockQuote => {
                    stylestack.pop();
                    blockquote = false;
                }
                NodeValue::LineBreak => {
                    doc.push(PaddedElement::new(
                        pstack.pop().unwrap(),
                        Margins::trbl(0, 0, docstyle.paragraph_spacing, 0),
                    ));

                    let mut p = Paragraph::default();
                    if docstyle.align_justify {
                        p.set_alignment(Alignment::Justified);
                    }
                    pstack.push(p);
                }
                NodeValue::List(_lst) => {
                    let list = nested_stack.pop().unwrap();
                    let active_list = nested_stack.last_mut();

                    match active_list {
                        Some(active_list) => {
                            active_list.push_no_bullet(list);
                        }
                        None => doc.push(list),
                    }
                }
                NodeValue::Item(_item) => {
                    // Items automatically contain a paragraph, so don't do anything here
                }
                _ => (),
            },
        }
    }

    let dt_building = t1.elapsed();

    doc.render_to_file("output.pdf").unwrap();

    let dt_render_to_file = t1.elapsed();

    println!("In memory build took {} sec", dt_building.as_secs_f32());
    println!(
        "Render to file took {} sec",
        (dt_render_to_file - dt_building).as_secs_f32()
    );
}
