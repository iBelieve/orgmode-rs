use std::io::{self, BufReader, BufRead, Error as IoError};
use std::fs::File;
use std::iter::Peekable;

macro_rules! org_warning {
    ($($arg:tt)*) => {
        let warning = format!($($arg)*);
        println!("WARNING: {}", warning);
    };
}

pub struct Parser<'a> {
    lines: Peekable<Box<Iterator<Item=Result<String, io::Error>> + 'a>>,
    current_line: u32,
    pub io_error: Option<IoError>
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
            current_line: 1,
            io_error: None
        }
    }

    pub fn next(&mut self) -> Option<String> {
        match self.lines.next() {
            Some(Ok(line)) => {
                self.current_line += 1;
                Some(line)
            },
            Some(Err(error)) => {
                self.io_error = Some(error);
                None
            },
            None => None
        }
    }

    pub fn peek(&mut self) -> Option<&str> {
        match self.lines.peek() {
            Some(Ok(line)) => Some(line.as_str()),
            Some(Err(_)) => None,
            None => None
        }
    }

    pub fn take_until(&mut self, end_line: &str) -> Vec<String> {
        let mut lines = Vec::new();

        while let Some(line) = self.next() {
            if line.trim() == end_line {
                return lines;
            } else {
                lines.push(line);
            }
        }

        org_warning!("Expected `{}` before end of file", end_line);
        lines
    }
}
