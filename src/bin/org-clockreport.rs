extern crate colored;
extern crate itertools;
extern crate orgmode;

use colored::Colorize;
use itertools::Itertools;
use orgmode::{format_duration, Document, Duration, Library, Node};
use std::env;
use std::path::Path;

fn main() {
    let mut library = Library::new();

    for argument in env::args().skip(1) {
        library
            .open(Path::new(&argument))
            .expect("Unable to open path");
    }

    let mut total_time_spent = Duration::zero();

    for (project_name, nodes) in library
        .nodes_clocked_to_today()
        .group_by(|node| get_project_name(&library[node.document_id], node))
        .into_iter()
    {
        let nodes: Vec<&Node> = nodes.collect();
        let time_spent = nodes
            .iter()
            .map(|node| node.time_spent_today())
            .fold(Duration::zero(), |total, duration| total + duration);

        total_time_spent = total_time_spent + time_spent;

        println!(
            "{}",
            format!(
                "{}\t[{}]",
                project_name,
                format_duration(&round_duration(&time_spent))
            ).bold()
            .blue()
        );

        let mut other = false;
        let mut other_duration = Duration::zero();

        for node in nodes.iter() {
            if node.title() == project_name {
                other = true;
                other_duration = other_duration + node.time_spent_today();
            } else {
                println!(
                    " - {}\t{}",
                    node.title(),
                    format!("[{}]", format_duration(&node.time_spent_today())).green()
                );
            }
        }

        if other {
            println!(
                " - General tasks\t{}",
                format!("[{}]", format_duration(&other_duration)).green()
            );
        }
        println!();
    }

    println!(
        "{}",
        format!(
            "Total time spent\t[{}]",
            format_duration(&round_duration(&total_time_spent))
        ).bold()
        .white()
    );
}

fn round_duration(duration: &Duration) -> Duration {
    Duration::minutes(15 * (duration.num_minutes() / 15))
}

fn get_project_name<'a>(document: &'a Document, node: &'a Node) -> &'a str {
    document
        .parent_with_tag(node.id, "PROJECT")
        .map(|node| node.title())
        .unwrap_or_else(|| &document.title)
}
