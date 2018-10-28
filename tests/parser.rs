extern crate orgmode;
extern crate petgraph;
extern crate chrono;

// use chrono::prelude::*;

#[test]
fn test_parsing_notes() {
    let text = include_str!("../../NOTES.org");
    let document = orgmode::parse(text).unwrap();

    assert!(document.child_ids().len() > 0);
    // assert!(document.nodes_for_date(Local.ymd(2018, 9, 15)).count() > 0);
}
