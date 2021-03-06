use document::{Document, DocumentId};
use std::collections::HashMap;
use std::path::Path;
use std::fs;
use std::fmt;
use timestamp::{Date, today};
use agenda::{Agenda, AgendaRange};
use std::io::Error as IoError;
use std::ops::{Index, IndexMut};
use std::ffi::OsStr;
use node::Node;

#[derive(Serialize)]
pub struct Library {
    #[serde(skip)]
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

    pub fn add(&mut self, mut document: Document) -> (DocumentId, &Document) {
        let id = self.next_id;
        self.next_id += 1;
        document.set_id(id);
        self.documents.insert(id, document);
        (id, &self.documents[&id])
    }

    pub fn open(&mut self, path: &Path) -> Result<(), IoError> {
        if path.is_dir() {
            for entry in fs::read_dir(path)? {
                let path = entry?.path();
                if path.is_dir() || path.extension() == Some(OsStr::new("org")) {
                    self.open(&path)?;
                }
            }
        } else {
            self.open_file(path)?;
        }
        Ok(())
    }

    pub fn open_file(&mut self, path: &Path) -> Result<(DocumentId, &Document), IoError> {
        Ok(self.add(Document::open_file(path)?))
    }

    pub fn agenda(&self, range: AgendaRange, start_date: Date) -> Agenda {
        Agenda::new(self, start_date, range)
    }

    pub fn agenda_today(&self) -> Agenda {
        self.agenda(AgendaRange::Day, today())
    }

    pub fn agenda_this_week(&self) -> Agenda {
        self.agenda(AgendaRange::Week, today())
    }

    pub fn document(&self, id: DocumentId) -> Option<&Document> {
        self.documents.get(&id)
    }

    pub fn document_mut(&mut self, id: DocumentId) -> Option<&mut Document> {
        self.documents.get_mut(&id)
    }

    pub fn documents(&self) -> impl Iterator<Item=&Document> {
        self.documents.values()
    }

    pub fn nodes_clocked_to_today(&self) -> impl Iterator<Item = &Node> {
        self.documents()
            .flat_map(|document| document.nodes_clocked_to_today())
    }
}

impl Index<DocumentId> for Library {
    type Output = Document;

    fn index(&self, id: DocumentId) -> &Document {
        self.document(id).unwrap()
    }
}

impl IndexMut<DocumentId> for Library {
    fn index_mut(&mut self, id: DocumentId) -> &mut Document {
        self.document_mut(id).unwrap()
    }
}

impl fmt::Debug for Library {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("Library")
           .field("documents", &self.documents.values())
           .finish()
    }
}
