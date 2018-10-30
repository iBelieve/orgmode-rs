use library::Library;
use document::{Document, DocumentId};
use node::{Node, NodeId};
use std::collections::HashMap;
use timestamp::{Date, Timestamp};
use headline::Headline;
use chrono::{Datelike, Duration};

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
            agenda.past_scheduled = document
                .nodes_past_scheduled()
                .map(|node| AgendaEntry::from_node(document, node))
                .collect();
            agenda.past_deadline = document
                .nodes_past_deadline()
                .map(|node| AgendaEntry::from_node(document, node))
                .collect();

            for date in &dates {
                let entries = agenda.entries.entry(*date).or_insert_with(Vec::new);
                let new_entries: Vec<AgendaEntry> = document
                    .nodes_for_date(*date)
                    .map(|node| AgendaEntry::from_node(document, node))
                    .collect();
                entries.extend(new_entries);
            }
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

pub struct AgendaEntry {
    pub doc_id: DocumentId,
    pub node_id: NodeId,
    pub headline: Headline,
    pub category: String,
    pub scheduled_for: Option<Timestamp>,
    pub deadline: Option<Timestamp>,
    pub closed_at: Option<Timestamp>
}

impl AgendaEntry {
    fn from_node(document: &Document, node: &Node) -> Self {
        AgendaEntry {
            doc_id: document.id,
            node_id: node.id,
            headline: node.headline.clone(),
            category: document.node_category(node.id).unwrap_or("").to_string(),
            scheduled_for: node.scheduled_for.clone(),
            deadline: node.deadline.clone(),
            closed_at: node.closed_at.clone()
        }
    }
}
