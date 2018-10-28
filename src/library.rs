use document::Document;
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use parser::Error;
use timestamp::{Date, today};
use agenda::{Agenda, AgendaRange};

pub type DocumentId = usize;

pub struct Library {
    next_id: DocumentId,
    documents: HashMap<DocumentId, Document>
}

impl Library {
    pub fn new() -> Self {
        Library {
            next_id: 0,
            documents: HashMap::new()
        }
    }

    pub fn add(&mut self, document: Document) -> (DocumentId, &Document) {
        let id = self.next_id;
        self.next_id += 1;
        self.documents.insert(id, document);
        (id, &self.documents[&id])
    }

    pub fn open(&mut self, path: &Path) -> Result<(), Error> {
        if path.is_dir() {
            for entry in fs::read_dir(path).map_err(Error::IoError)? {
                let path = entry.map_err(Error::IoError)?.path();
                if path.is_dir() || path.ends_with(".org") {
                    self.open(&path)?;
                }
            }
        } else {
            self.open_file(path)?;
        }
        Ok(())
    }

    pub fn open_file(&mut self, path: &Path) -> Result<(DocumentId, &Document), Error> {
        Ok(self.add(Document::open_file(path)?))
    }

    pub fn agenda(&self, range: AgendaRange, start_date: Date) -> Agenda {
        Agenda {
            start_date,
            range,
            entries: HashMap::new()
        }
    }

    pub fn agenda_today(&self) -> Agenda {
        self.agenda(AgendaRange::Day, today())
    }

    pub fn agenda_this_week(&self) -> Agenda {
        self.agenda(AgendaRange::Week, today())
    }
}
