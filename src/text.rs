use itertools::Itertools;
use std::fmt;
use textwrap::fill;
use utils::StringUtils;
use LINE_LENGTH;

#[derive(Default, Serialize, Deserialize)]
pub struct Text {
    text: String,
    indent: usize,
    leading_blank_lines: usize,
    trailing_blank_lines: usize,
}

impl Text {
    pub fn new(line: &str) -> Self {
        let mut text = Text::default();
        for line in line.lines() {
            text.add_line(line);
        }
        text
    }

    pub fn add_line(&mut self, line: &str) {
        if line.is_empty() {
            if self.text.is_empty() {
                self.leading_blank_lines += 1;
            } else {
                self.trailing_blank_lines += 1;
            }
        } else if self.text.is_empty() {
            self.indent = line.indent();
            self.text = line.trim().to_string();
        } else {
            self.text += " ";
            self.text += line.trim();
        }
    }

    pub fn at_end(&self) -> bool {
        self.trailing_blank_lines >= 2
    }

    pub fn format(&self, indent: usize) -> String {
        let text = fill(self.text.trim(), LINE_LENGTH - (indent + self.indent));
        let line_prefix = String::from("\n") + &" ".repeat(indent + self.indent);

        "\n".repeat(self.leading_blank_lines)
            + &" ".repeat(self.indent)
            + &text.split("\n").join(&line_prefix)
            + &"\n".repeat(self.trailing_blank_lines)
    }
}

impl fmt::Display for Text {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format() {
        let text = Text::new(
            "  This is a paragraph\nthat needs to be formatted nicely and should wrap\nto the \
             next line",
        );
        assert_eq!(
            &text.format(0),
            "  This is a paragraph that needs to be formatted nicely and should wrap to the\n  \
             next line"
        );
    }
}
