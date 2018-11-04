use itertools::Itertools;

pub trait StringUtils {
    fn indent(&self) -> usize;
    fn add_indent(&self, indent: usize) -> String;
    fn capped(&self, cap: &str) -> String;
}

impl<'a> StringUtils for &'a str {
    fn indent(&self) -> usize {
        self.chars().take_while(|c| c.is_whitespace()).count()
    }

    fn add_indent(&self, indent: usize) -> String {
        let indent = " ".repeat(indent);
        let prefix = String::from("\n") + &indent;
        indent + &self.lines().join(&prefix)
    }

    fn capped(&self, cap: &str) -> String {
        String::from(cap) + self + cap
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

    fn capped(&self, cap: &str) -> String {
        String::from(cap) + self + cap
    }
}
