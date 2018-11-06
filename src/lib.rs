extern crate chrono;
#[macro_use]
extern crate lazy_static;
extern crate itertools;
extern crate regex;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate textwrap;

#[macro_use]
mod parser;

mod agenda;
mod document;
mod drawer;
mod element;
mod headline;
mod library;
mod list;
mod logbook;
mod node;
mod planning;
mod section;
mod table;
mod text;
mod timestamp;
mod timestamps;
mod tree;
mod utils;

use std::io::Error as IoError;
use std::path::Path;

pub use agenda::{Agenda, AgendaEntry, AgendaEntryKind, AgendaRange};
pub use document::{Document, DocumentId};
pub use drawer::Drawer;
pub use element::Element;
pub use headline::Headline;
pub use library::Library;
pub use list::List;
pub use logbook::Logbook;
pub use node::{Node, NodeId};
pub use parser::Parser;
pub use planning::Planning;
pub use section::Section;
pub use table::Table;
pub use timestamp::{today, format_duration, Date, Duration, Time, Timestamp};

pub const LINE_LENGTH: usize = 80;

pub fn open_file(path: &Path) -> Result<Document, IoError> {
    Document::open_file(path)
}

pub fn from_string(source: &str) -> Document {
    Document::from_string(source)
}
