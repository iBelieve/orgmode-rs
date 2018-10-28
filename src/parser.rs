use document::Document;
use headline::Headline;
use drawer::Drawer;
use planning::Planning;
use element::Element;
use std::io::{self, BufReader, BufRead};
use std::fs::File;
use std::iter::Peekable;

#[derive(Debug, Fail)]
#[fail(display = "Error on line {}: {}", line, description)]
pub struct ParseError {
    description: String,
    line: u32,
    column: Option<u32>
}

#[derive(Debug, Fail)]
pub enum Error {
    #[fail(display = "Parsing error: {}", _0)]
    ParseError(#[cause] ParseError),
    #[fail(display = "I/O error: {}", _0)]
    IoError(#[cause] io::Error)
}

pub struct Parser<'a> {
    lines: Peekable<Box<Iterator<Item=Result<String, io::Error>> + 'a>>,
    current_line: u32
}

impl<'a> Parser<'a> {
    pub fn from_string(source: &'a str) -> Self {
        Parser::new(Box::new(source.lines().map(|line| Ok(line.to_string()))))
    }

    pub fn from_file(file: File) -> Self {
        Parser::new(Box::new(BufReader::new(file).lines()))
    }

    pub fn new(iter: Box<Iterator<Item=Result<String, io::Error>> + 'a>) -> Self {
        Parser {
            lines: iter.peekable(),
            current_line: 1
        }
    }

    pub fn next(&mut self) -> Result<Option<String>, Error> {
        match self.lines.next() {
            Some(Ok(line)) => {
                self.current_line += 1;
                Ok(Some(line))
            },
            Some(Err(error)) => Err(Error::IoError(error)),
            None => Ok(None)
        }
    }

    pub fn peek(&mut self) -> Option<&str> {
        match self.lines.peek() {
            Some(Ok(line)) => Some(line.as_str()),
            Some(Err(_)) => None,
            None => None
        }
    }

    pub fn error(&self, description: impl Into<String>) -> Error {
        Error::ParseError(ParseError {
            description: description.into(),
            line: self.current_line,
            column: None
        })
    }

    pub fn error_at_column(&self, column: u32, description: impl Into<String>) -> Error {
        Error::ParseError(ParseError {
            description: description.into(),
            line: self.current_line,
            column: Some(column)
        })
    }

    pub fn take_until(&mut self, end_line: &str) -> Result<Vec<String>, Error> {
        let mut lines = Vec::new();

        while let Some(line) = self.next()? {
            if line == end_line {
                return Ok(lines);
            } else {
                lines.push(line);
            }
        }

        Err(self.error(format!("Expected `{}` before end of file", end_line)))
    }
}

pub fn parse(file: &str) -> Result<Document, Error> {
    let mut todo_keywords = vec!["TODO".to_string(), "DONE".to_string()];

    let mut document = Document::new();
    let mut current_id = document.root_id();

    let mut parser = Parser::from_string(file);

    while let Some(line) = parser.next()? {
        if let Some(headline) = Headline::parse(&line, &todo_keywords) {
            current_id = Some(document.add_new_node(current_id, headline));
        } else if let Some(drawer) = Drawer::parse(&line, &mut parser)? {
            if let Some(properties) = drawer.as_properties() {
                if let Some(current_id) = current_id {
                    document.node_mut(current_id).properties = properties;
                } else {
                    document.properties.extend(properties);
                }
            } else {
                document.section_mut(current_id).add_drawer(drawer);
            }
        } else if let Some(planning) = Planning::parse(&line)? {
            if let Some(current_id) = current_id {
                document.node_mut(current_id).set_planning(planning, line);
            } else {
                println!("WARNING: planning info found above first headline");
                document.section_mut(current_id).add_line(line);
            }
        } else if let Some(element) = Element::parse_greater(&line, &mut parser)? {
            document.section_mut(current_id).elements.push(element);
        } else {
            document.section_mut(current_id).add_line(line);
        }
    }

    Ok(document)
}
