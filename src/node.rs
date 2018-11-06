use drawer::Drawer;
use element::Element;
use headline::Headline;
use logbook::Logbook;
use planning::Planning;
use regex::Regex;
use section::Section;
use std::collections::HashMap;
use std::fmt;
use timestamp::{Date, Duration, Timestamp};
use document::DocumentId;

pub type NodeId = usize;

#[derive(Default, Serialize, Deserialize)]
pub struct Node {
    pub document_id: DocumentId,
    pub id: NodeId,
    #[serde(skip)]
    pub indent: u16,
    #[serde(flatten)]
    pub headline: Headline,
    pub properties: HashMap<String, String>,
    pub section: Section,
    pub scheduled_for: Option<Timestamp>,
    pub deadline: Option<Timestamp>,
    pub closed_at: Option<Timestamp>,
}

impl Node {
    pub fn from_headline(headline: Headline) -> Self {
        Node {
            indent: headline.indent,
            headline,
            ..Node::default()
        }
    }

    pub fn title(&self) -> &str {
        &self.headline.title
    }

    pub fn add_line(&mut self, line: &str) {
        self.section.add_line(line)
    }

    pub fn set_planning(&mut self, planning: Planning, line: &str) {
        if self.has_planning() {
            org_warning!("Planning info already set");
        } else if !self.section.is_empty() || !self.properties.is_empty() {
            org_warning!("Planning info must come immediately after the headline");
            self.add_line(line);
        } else {
            self.deadline = planning.deadline;
            self.scheduled_for = planning.scheduled;
            self.closed_at = planning.closed;

            let timestamps = vec![&self.deadline, &self.scheduled_for, &self.closed_at]
                .into_iter()
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
                closed: self.closed_at.clone(),
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

    pub fn timestamps_for_date<'a>(
        &'a self,
        date: &'a Date,
    ) -> impl Iterator<Item = Timestamp> + 'a {
        self.section.timestamps_for_date(date)
    }

    pub fn is_past_scheduled(&self) -> bool {
        self.scheduled_for
            .as_ref()
            .map(|timestamp| timestamp.is_past())
            .unwrap_or(false)
    }

    pub fn is_past_deadline(&self) -> bool {
        self.deadline
            .as_ref()
            .map(|timestamp| timestamp.is_past())
            .unwrap_or(false)
    }

    pub fn property(&self, name: &str) -> Option<&str> {
        self.properties.get(name).map(|prop| prop.as_str())
    }

    pub fn has_tag(&self, tag: &str) -> bool {
        self.headline.tags.iter().any(|t| t == tag)
    }

    pub fn is_habit(&self) -> bool {
        self.property("STYLE") == Some("habit")
    }

    pub fn effort(&self) -> Option<Duration> {
        lazy_static! {
            static ref REGEX: Regex = Regex::new(
                r#"^(?P<hours>\d+):(?P<minutes>\d+)$"#
            ).unwrap();
        }

        if let Some(captures) = self
            .property("Effort")
            .and_then(|effort| REGEX.captures(effort))
        {
            let hours: i64 = captures.name("hours").unwrap().as_str().parse().unwrap();
            let minutes: i64 = captures.name("minutes").unwrap().as_str().parse().unwrap();

            Some(Duration::hours(hours) + Duration::minutes(minutes))
        } else {
            None
        }
    }

    pub fn drawer(&self, name: &str) -> Option<&Drawer> {
        self.section
            .elements
            .iter()
            .filter_map(|element| match element {
                Element::Drawer(drawer) if drawer.name == name => Some(drawer),
                _ => None,
            }).next()
    }

    // TODO: Avoid reparsing the logbook from the drawer each time it is accessed
    pub fn logbook(&self) -> Logbook {
        Logbook::from(self.drawer("LOGBOOK"))
    }

    pub fn time_spent_today(&self) -> Duration {
        self.logbook().time_spent_today()
    }

    pub fn was_clocked_to_today(&self) -> bool {
        self.logbook().was_clocked_to_today()
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
