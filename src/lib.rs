extern crate chrono;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate failure;
extern crate regex;
extern crate itertools;
extern crate serde;
#[macro_use]
extern crate serde_derive;

mod document;
mod drawer;
mod element;
mod headline;
mod node;
mod parser;
mod planning;
mod section;
mod timestamp;
mod library;
mod agenda;
mod tree;

use std::path::Path;

pub use headline::Headline;
pub use drawer::Drawer;
pub use element::{Element, Paragraph};
pub use node::{Node, NodeId};
pub use document::{Document, DocumentId};
pub use timestamp::{Timestamp, Date, Time, today};
pub use section::Section;
pub use planning::Planning;
pub use parser::{Parser, Error};
pub use library::Library;
pub use agenda::{Agenda, AgendaEntry, AgendaRange};

pub fn open_file(path: &Path) -> Result<Document, Error> {
    Document::open_file(path)
}

pub fn from_string(source: &str) -> Result<Document, Error> {
    Document::from_string(source)
}
