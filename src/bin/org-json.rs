/// org-json is a simple command-line interface for debugging the structure of an org mode document
extern crate orgmode;
extern crate serde_json;

use std::env;
use std::path::Path;
use orgmode::Library;

fn main() {
    let mut library = Library::new();

    for argument in env::args().skip(1) {
        library.open(Path::new(&argument))
            .expect("Unable to open path");
    }

    println!("{}", serde_json::to_string(&library).unwrap());
}
