use drawer::Drawer;
use element::Element;
use std::fmt;
use itertools::Itertools;

#[derive(Default)]
pub struct Section {
    pub elements: Vec<Element>
}

impl Section {
    pub fn new() -> Self {
        Section {
            elements: Vec::new()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn add_drawer(&mut self, drawer: Drawer) {
        self.elements.push(Element::Drawer(drawer))
    }

    pub fn add_line(&mut self, line: String) {
        if let Some(Element::Paragraph(paragraph)) = self.elements.last_mut() {
            paragraph.add_line(&line);
            return;
        }

        self.elements.push(Element::new_paragraph(line));
    }
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.elements.iter().join("\n"))
    }
}
