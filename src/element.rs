use drawer::Drawer;
use itertools::Itertools;
use list::List;
use parser::Parser;
use std::fmt;
use table::Table;
use text::Text;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Element {
    Drawer(Drawer),
    Paragraph(Text),
    Comment { text: String },
    FixedWidthArea { text: String },
    // Block(Block)
    HorizontalRule,
    Table(Table),
    List(List),
}

impl Element {
    pub fn parse(line: &str, parser: &mut Parser) -> Option<Element> {
        if let Some(text) = parse_area_prefixed(line, parser, "#") {
            Some(Element::Comment { text })
        } else if let Some(text) = parse_area_prefixed(line, parser, ":") {
            Some(Element::FixedWidthArea { text })
        // } else if let Some(block) = Block::parse(line, parser)? {
        //     Some(Element::Block(block))
        } else if is_horizontal_rule(line) {
            Some(Element::HorizontalRule)
        } else if let Some(table) = Table::parse(line, parser) {
            Some(Element::Table(table))
        } else if let Some(list) = List::parse(line, parser) {
            Some(Element::List(list))
        } else {
            None
        }
    }

    pub fn new_paragraph(text: &str) -> Self {
        Element::Paragraph(Text::new(text))
    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Element::Drawer(drawer) => write!(f, "{}", drawer),
            Element::Paragraph(paragraph) => write!(f, "{}", paragraph),
            Element::Comment { text } => write!(f, "{}", prefixed(text, "#")),
            Element::FixedWidthArea { text } => write!(f, "{}", prefixed(text, ":")),
            Element::HorizontalRule => write!(f, "{}", "-".repeat(5)),
            Element::Table(table) => write!(f, "{}", table),
            Element::List(list) => write!(f, "{}", list),
        }
    }
}

fn prefixed(text: &str, prefix: &str) -> String {
    text.split('\n')
        .map(|line| format!("{} {}", prefix, line))
        .join("\n")
}

fn parse_prefixed<'a>(line: &'a str, prefix: &str) -> Option<&'a str> {
    let line = line.trim();

    if line == prefix {
        Some("")
    } else if line.starts_with(&(prefix.to_string() + " ")) {
        Some(&line[prefix.len() + 1..])
    } else {
        None
    }
}

fn parse_area_prefixed(line: &str, parser: &mut Parser, prefix: &str) -> Option<String> {
    if let Some(area) = parse_prefixed(line, prefix) {
        let mut area = area.to_string();

        while let Some(line) = parser.peek().map(|s| s.to_string()) {
            if let Some(more_area) = parse_prefixed(&line, prefix) {
                parser.next();
                area += "\n";
                area += more_area;
            } else {
                break;
            }
        }
        Some(area)
    } else {
        None
    }
}

fn is_horizontal_rule(line: &str) -> bool {
    line.len() >= 5 && line.chars().all(|c| c == '-')
}
