use headline::Headline;
use section::Section;
use std::collections::HashMap;
use planning::Planning;
use timestamp::Timestamp;

#[derive(Default)]
pub struct Node {
    pub indent: u16,
    pub headline: Headline,
    pub properties: HashMap<String, String>,
    pub section: Section,
    pub deadline: Option<Timestamp>,
    pub scheduled_at: Option<Timestamp>,
    pub closed_at: Option<Timestamp>
}

impl Node {
    pub fn from_headline(headline: Headline) -> Self {
        Node {
            indent: headline.indent,
            headline,
            ..Node::default()
        }
    }

    pub fn add_line(&mut self, line: String) {
        self.section.add_line(line)
    }

    pub fn set_planning(&mut self, planning: Planning) {
        if self.deadline.is_some() || self.scheduled_at.is_some() || self.closed_at.is_some() {
            println!("WARNING: Planning info already set");
        } else if !self.section.is_empty() || !self.properties.is_empty() {
            println!("WARNING: Planning info must come immediately after the headline");
            self.add_line(planning.line);
        } else {
            self.deadline = planning.deadline;
            self.scheduled_at = planning.scheduled;
            self.closed_at = planning.closed;
        }
    }
}
