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
