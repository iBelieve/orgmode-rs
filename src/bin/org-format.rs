/// org-format is a simple command-line interface for reformatting org mode files

extern crate orgmode;

use std::env;
use orgmode::Document;

fn main() {
    for argument in env::args().skip(1) {
        let document = Document::open_file(&argument)
            .expect("Unable to open file");
        println!("{}", document);
    }
}
