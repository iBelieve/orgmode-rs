use chrono::prelude::*;
use regex::Regex;
use parser::Error;

#[derive(Clone, PartialEq, Debug)]
pub struct Timestamp {
    string: String,
    date: Option<Date<Local>>,
    end_date: Option<Date<Local>>,
    time: Option<NaiveTime>,
    end_time: Option<NaiveTime>,
    is_active: bool,
    // repeater,
    // warning_delay,
}

impl Timestamp {
    pub fn parse(timestamp: &str) -> Result<Self, Error> {
        // TODO: Implement me!
        Ok(Timestamp {
            string: String::new(),
            date: None,
            end_date: None,
            time: None,
            end_time: None,
            is_active: false
        })
    }
}
