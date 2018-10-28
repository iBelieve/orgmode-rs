use drawer::Drawer;
use parser::{Parser, Error};
use regex::Regex;
use std::fmt;
use itertools::Itertools;

pub enum Element {
    Drawer(Drawer),
    Paragraph(Paragraph),
    Comment(String),
    FixedWidthBlock(String),
    // HorizontalRule,
    // Table
}

impl Element {
    pub fn parse_greater(line: &str, parser: &mut Parser) -> Result<Option<Element>, Error> {
        let element = if let Some(comment) = parse_block_prefixed(line, parser, "#")? {
            Some(Element::Comment(comment))
        } else if let Some(block) = parse_block_prefixed(line, parser, ":")? {
            Some(Element::FixedWidthBlock(block))
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
            Element::Comment(comment) => write!(f, "{}", prefixed(comment, "#")),
            Element::FixedWidthBlock(block) => write!(f, "{}", prefixed(block, ":")),
        }
    }
}

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
    if line == prefix {
        Some("")
    } else if line.starts_with(&(prefix.to_string() + " ")) {
        Some(&line[prefix.len() + 1..])
    } else {
        None
    }
}

fn parse_block_prefixed(line: &str,  parser: &mut Parser, prefix: &str) -> Result<Option<String>, Error> {
    let block = if let Some(block) = parse_prefixed(line, prefix) {
        let mut block = block.to_string();

        while let Some(line) = parser.peek().map(|s| s.to_string()) {
            if let Some(more_block) = parse_prefixed(&line, prefix) {
                parser.next()?;
                block += "\n";
                block += more_block;
            } else {
                break;
            }
        }
        Some(block)
    } else {
        None
    };
    Ok(block)
}
