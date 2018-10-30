use headline::Headline;
use section::Section;
use std::collections::HashMap;
use planning::Planning;
use timestamp::{Date, Timestamp};
use std::fmt;
use drawer::Drawer;

pub type NodeId = usize;

#[derive(Default, Serialize, Deserialize)]
pub struct Node {
    pub id: NodeId,
    #[serde(skip)]
    pub indent: u16,
    #[serde(flatten)]
    pub headline: Headline,
    pub properties: HashMap<String, String>,
    pub section: Section,
    pub scheduled_for: Option<Timestamp>,
    pub deadline: Option<Timestamp>,
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

    pub fn set_planning(&mut self, planning: Planning, line: String) {
        if self.has_planning() {
            println!("WARNING: Planning info already set");
        } else if !self.section.is_empty() || !self.properties.is_empty() {
            println!("WARNING: Planning info must come immediately after the headline");
            self.add_line(line);
        } else {
            self.deadline = planning.deadline;
            self.scheduled_for = planning.scheduled;
            self.closed_at = planning.closed;

            let timestamps = vec![&self.deadline, &self.scheduled_for, &self.closed_at].into_iter()
                .filter_map(|some| some.clone().clone());

            self.section.timestamps.timestamps.extend(timestamps);
            self.section.timestamps.timestamps.sort();
        }
    }

    fn has_planning(&self) -> bool {
        self.deadline.is_some() || self.scheduled_for.is_some() || self.closed_at.is_some()
    }

    fn planning(&self) -> Option<Planning> {
        if self.has_planning() {
            Some(Planning {
                scheduled: self.scheduled_for.clone(),
                deadline: self.deadline.clone(),
                closed: self.closed_at.clone()
            })
        } else {
            None
        }
    }

    fn has_properties(&self) -> bool {
        !self.properties.is_empty()
    }

    fn properties_drawer(&self) -> Option<Drawer> {
        if self.has_properties() {
            Some(Drawer::from_properties(&self.properties))
        } else {
            None
        }
    }

    pub fn matches_date(&self, date: &Date) -> bool {
        self.section.matches_date(date)
    }

    pub fn timestamps_for_date<'a>(&'a self, date: &'a Date) -> impl Iterator<Item=&'a Timestamp> {
        self.section.timestamps_for_date(date)
    }

    pub fn is_past_scheduled(&self) -> bool {
        self.scheduled_for.as_ref()
            .map(|timestamp| timestamp.is_past())
            .unwrap_or(false)
    }

    pub fn is_past_deadline(&self) -> bool {
        self.deadline.as_ref()
            .map(|timestamp| timestamp.is_past())
            .unwrap_or(false)
    }
}

impl fmt::Display for Node {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.headline)?;
        if let Some(planning) = self.planning() {
            write!(f, "\n{}", planning)?;
        }
        if let Some(drawer) = self.properties_drawer() {
            write!(f, "\n{}", drawer)?;
        }
        write!(f, "\n{}", self.section)?;

        Ok(())
    }
}
