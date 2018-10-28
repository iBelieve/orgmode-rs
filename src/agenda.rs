use library::DocumentId;
use document::NodeId;
use std::collections::HashMap;
use timestamp::Date;
use headline::Headline;

pub enum AgendaRange {
    Day,
    Week
}

pub struct Agenda {
    pub start_date: Date,
    pub range: AgendaRange,
    pub entries: HashMap<Date, Vec<AgendaEntry>>
}

impl Agenda {
    pub fn dates(&self) -> impl Iterator<Item=&Date> {
        self.entries.keys()
    }

    pub fn entries(&self, date: Date) -> &[AgendaEntry] {
        self.entries.get(&date).map(|vec| vec.as_slice()).unwrap_or(&[])
    }
}

pub struct AgendaEntry {
    pub doc_id: DocumentId,
    pub node_id: NodeId,
    pub headline: Headline
}
