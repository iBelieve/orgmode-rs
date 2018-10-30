/// org-agenda is a simple command-line application for displaying today's or this week's agenda

extern crate orgmode;
extern crate colored;
extern crate chrono;

use std::env;
use std::path::Path;
use orgmode::{Library, Agenda, AgendaRange, Timestamp, today};
use colored::Colorize;

fn main() {
    let mut library = Library::new();

    for argument in env::args().skip(1) {
        library.open(Path::new(&argument))
            .expect("Unable to open path");
    }

    let agenda = library.agenda_this_week();

    print_agenda(&agenda);
}

fn print_agenda(agenda: &Agenda) {
    let title = match agenda.range {
        AgendaRange::Day => "Daily",
        AgendaRange::Week => "Weekly",
    };
    println!("{}", format!("==================== {} Agenda ====================", title).white().bold());
    let mut first = true;
    let today = today();
    for date in agenda.dates() {
        let date_format = if first {
            first = false;
            "%_d %B %Y W%W"
        } else {
            "%_d %B %Y"
        };
        let color = if date == today { "green" } else { "normal" };
        println!("{}", format!("{:11}{}", date.format("%A").to_string(),
                                          date.format(date_format)).bold().color(color));

        if date == today {
            for entry in agenda.past_scheduled.iter() {

            }
            for entry in agenda.past_deadline.iter() {

            }
        }

        for entry in agenda.entries(&date) {
            let is_scheduled = entry.scheduled_for.as_ref()
                .map(|timestamp| timestamp.contains(&date))
                .unwrap_or(false);
            let is_deadline = entry.deadline.as_ref()
                .map(|timestamp| timestamp.contains(&date))
                .unwrap_or(false);

            print!("  {:10}", format!("{}:", entry.category));

            if is_deadline {
                print_time(entry.deadline.as_ref().unwrap());
                print!(" Deadline:  ");
            } else if is_scheduled {
                print_time(entry.scheduled_for.as_ref().unwrap());
                print!(" Scheduled: ");
            }
            if let Some(ref keyword) = entry.headline.keyword {
                print!(" {}", keyword.blue());
            }
            if let Some(ref priority) = entry.headline.priority {
                print!(" {}", format!("[#{}]", priority).red());
            }
            println!(" {}", entry.headline.title);
        }
    }
}

fn print_time(timestamp: &Timestamp) {
    if let Some(start_time) = timestamp.time {
        print!(" {}", start_time.format("%_H:%M"));

        if let Some(end_time) = timestamp.end_time {
            print!("-{}", end_time.format("%_H:%M"));
        } else {
            print!("......");
        }
    }
}
