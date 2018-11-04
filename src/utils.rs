use itertools::Itertools;

pub trait StringUtils {
    fn indent(&self) -> usize;
    fn add_indent(&self, indent: usize) -> String;
}

impl StringUtils for &str {
    fn indent(&self) -> usize {
        self.chars().take_while(|c| c.is_whitespace()).count()
    }

    fn add_indent(&self, indent: usize) -> String {
        let indent = " ".repeat(indent);
        let prefix = String::from("\n") + &indent;
        indent + &self.lines().join(&prefix)
    }
}



impl StringUtils for String {
    fn indent(&self) -> usize {
        self.chars().take_while(|c| c.is_whitespace()).count()
    }

    fn add_indent(&self, indent: usize) -> String {
        let indent = " ".repeat(indent);
        let prefix = String::from("\n") + &indent;
        indent + &self.lines().join(&prefix)
    }
}
