use drawer::Drawer;
use parser::{Parser, Error};
use std::fmt;
use itertools::Itertools;

#[derive(Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Element {
    Drawer(Drawer),
    Paragraph(Paragraph),
    Comment { text: String },
    FixedWidthArea { text: String },
    // Block(Block)
    // HorizontalRule,
    // Table
}

impl Element {
    pub fn parse_greater(line: &str, parser: &mut Parser) -> Result<Option<Element>, Error> {
        let element = if let Some(text) = parse_area_prefixed(line, parser, "#")? {
            Some(Element::Comment { text })
        } else if let Some(text) = parse_area_prefixed(line, parser, ":")? {
            Some(Element::FixedWidthArea { text })
        // } else if let Some(block) = Block::parse(line, parser)? {
        //     Some(Element::Block(block))
        } else {
            None
        };
        Ok(element)
    }

    pub fn new_paragraph(text: String) -> Self {
        Element::Paragraph(Paragraph { text })
    }
}

impl fmt::Display for Element {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Element::Drawer(drawer) => write!(f, "{}", drawer),
            Element::Paragraph(paragraph) => write!(f, "{}", paragraph.text),
            Element::Comment { text } => write!(f, "{}", prefixed(text, "#")),
            Element::FixedWidthArea { text } => write!(f, "{}", prefixed(text, ":")),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Paragraph {
    text: String
}

impl Paragraph {
    pub fn add_line(&mut self, line: &str) {
        if !self.text.is_empty() {
            self.text += "\n"
        }
        self.text += line;
    }
}

fn prefixed(text: &str, prefix: &str) -> String {
    text.split('\n').map(|line| format!("{} {}", prefix, line)).join("\n")
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

fn parse_area_prefixed(line: &str,  parser: &mut Parser, prefix: &str) -> Result<Option<String>, Error> {
    let area = if let Some(area) = parse_prefixed(line, prefix) {
        let mut area = area.to_string();

        while let Some(line) = parser.peek().map(|s| s.to_string()) {
            if let Some(more_area) = parse_prefixed(&line, prefix) {
                parser.next()?;
                area += "\n";
                area += more_area;
            } else {
                break;
            }
        }
        Some(area)
    } else {
        None
    };
    Ok(area)
}
