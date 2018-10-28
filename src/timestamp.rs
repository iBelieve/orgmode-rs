use chrono;
// use regex::Regex;
use parser::Error;
use std::fmt;

pub type Date = chrono::Date<chrono::Local>;
pub type Time = chrono::NaiveTime;

#[derive(Clone, PartialEq, Debug)]
pub struct Timestamp {
    string: String,
    date: Option<Date>,
    end_date: Option<Date>,
    time: Option<Time>,
    end_time: Option<Time>,
    is_active: bool,
    // repeater,
    // warning_delay,
}

impl Timestamp {
    pub fn parse(_timestamp: &str) -> Result<Self, Error> {
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

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", if self.is_active { "<" } else { "[" })?;
        write!(f, "{}", if self.is_active { ">" } else { "]" })?;
        Ok(())
    }
}

pub fn today() -> Date {
    chrono::Local::today()
}
