use library::Library;
use document::{Document, DocumentId};
use node::{Node, NodeId};
use std::collections::HashMap;
use timestamp::{Date, Timestamp, TimestampKind};
use headline::Headline;
use chrono::{Datelike, Duration};
use std::cmp::Ordering;

pub enum AgendaRange {
    Day,
    Week
}

pub struct Agenda {
    pub start_date: Date,
    pub range: AgendaRange,
    pub past_scheduled: Vec<AgendaEntry>,
    pub past_deadline: Vec<AgendaEntry>,
    entries: HashMap<Date, Vec<AgendaEntry>>
}

impl Agenda {
    pub fn new(library: &Library, date: Date, range: AgendaRange) -> Self {
        let start_date = match range {
            AgendaRange::Day => date,
            AgendaRange::Week => date - Duration::days(date.weekday().number_from_monday() as i64 - 1)
        };
        let mut agenda = Agenda {
            start_date,
            range,
            past_scheduled: Vec::new(),
            past_deadline: Vec::new(),
            entries: HashMap::new()
        };
        let dates: Vec<Date> = agenda.dates().collect();
        for document in library.documents() {
            // agenda.past_scheduled = document
            //     .nodes_past_scheduled()
            //     .map(|node| AgendaEntry::from_node(document, node, Some(TimestampKind::Scheduled)))
            //     .collect();
            // agenda.past_deadline = document
            //     .nodes_past_deadline()
            //     .map(|node| AgendaEntry::from_node(document, node, Some(TimestampKind::Deadline)))
            //     .collect();

            for date in &dates {
                let entries = agenda.entries.entry(*date).or_insert_with(Vec::new);
                let new_entries: Vec<AgendaEntry> = document
                    .nodes_for_date(date)
                    .filter(|(_timestamp, node)| !node.is_habit())
                    .map(|(timestamp, node)| AgendaEntry::from_node(document, node, timestamp))
                    .collect();
                entries.extend(new_entries);
            }
        }
        for date in &dates {
            let entries = agenda.entries.get_mut(date).unwrap();
            entries.sort();
        }
        agenda
    }

    pub fn dates(&self) -> impl Iterator<Item=Date> + '_ {
        let range = match self.range {
            AgendaRange::Day => (0..1),
            AgendaRange::Week => (0..7)
        };
        range.map(move |offset| {
            let date = self.start_date.clone();
            date + Duration::days(offset)
        })
    }

    pub fn entries(&self, date: &Date) -> &[AgendaEntry] {
        self.entries.get(date).map(|vec| vec.as_slice()).unwrap_or(&[])
    }
}

#[derive(PartialEq, Eq)]
pub struct AgendaEntry {
    pub doc_id: DocumentId,
    pub node_id: NodeId,
    pub headline: Headline,
    pub category: String,
    pub timestamp: Timestamp,
    pub kind: AgendaEntryKind,
    pub time_spent: Duration,
    pub effort: Option<Duration>
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub enum AgendaEntryKind {
    Scheduled,
    Deadline,
    Normal
}

impl AgendaEntry {
    fn from_node(document: &Document, node: &Node, timestamp: Timestamp) -> Self {
        let category = document.node_category(node.id).unwrap_or("").to_string();
        let kind = match timestamp.kind {
            TimestampKind::Scheduled => AgendaEntryKind::Scheduled,
            TimestampKind::Deadline => AgendaEntryKind::Deadline,
            TimestampKind::Active => AgendaEntryKind::Normal,
            ref kind => panic!("Unexpected timestamp kind '{:?}' for \"{}\"", kind, node.headline.title)
        };

        AgendaEntry {
            doc_id: document.id,
            node_id: node.id,
            headline: node.headline.clone(),
            category,
            timestamp,
            kind,
            time_spent: document.node_time_spent(node.id),
            effort: node.effort()
        }
    }
}

impl Ord for AgendaEntry {
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.kind, &self.timestamp, &self.category)
            .cmp(&(&other.kind, &other.timestamp, &other.category))
    }
}

impl PartialOrd for AgendaEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
