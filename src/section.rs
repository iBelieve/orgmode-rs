use drawer::Drawer;
use element::Element;
use std::fmt;
use itertools::Itertools;
use timestamp::{Timestamp, Date};
use timestamps::Timestamps;

#[derive(Default, Serialize, Deserialize)]
pub struct Section {
    pub elements: Vec<Element>,
    pub(crate) timestamps: Timestamps
}

impl Section {
    pub fn new() -> Self {
        Section {
            elements: Vec::new(),
            timestamps: Timestamps::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    pub fn add_drawer(&mut self, drawer: Drawer) {
        self.elements.push(Element::Drawer(drawer))
    }

    pub fn add_line(&mut self, line: String) {
        self.timestamps.parse_and_append(&line);

        if let Some(Element::Paragraph(paragraph)) = self.elements.last_mut() {
            paragraph.add_line(&line);
            return;
        }

        self.elements.push(Element::new_paragraph(line));
    }

    pub fn matches_date(&self, date: &Date) -> bool {
        self.timestamps.matches_date(date)
    }

    pub fn timestamps_for_date<'a>(&'a self, date: &'a Date) -> impl Iterator<Item=Timestamp> + 'a {
        self.timestamps.timestamps_for_date(date)
    }
}

impl fmt::Display for Section {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.elements.iter().join("\n"))
    }
}
