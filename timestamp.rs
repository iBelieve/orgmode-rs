use chrono::prelude::*;
use regex::Regex;
use std::ops::{Add, AddAssign};

lazy_static! {
   static ref ACTIVE_REGEX: Regex = Regex::new("<(.*?)>").unwrap();
   static ref INACTIVE_REGEX: Regex = Regex::new("\\[(.*?)\\]").unwrap();
}

pub fn is_timestamp(text: &str) -> bool {
    ACTIVE_REGEX.is_match(text) || INACTIVE_REGEX.is_match(text)
}

#[derive(Default, Clone)]
pub struct Timestamps {
    pub timestamps: Vec<Timestamp>,
    pub inactive_timestamps: Vec<Timestamp>,
}

impl Timestamps {
    pub fn parse(text: &str) -> Timestamps {
        Timestamps {
            timestamps: ACTIVE_REGEX.captures_iter(text)
                .map(|timestamp| Timestamp::parse(&timestamp[1]))
                .collect(),
            inactive_timestamps: INACTIVE_REGEX.captures_iter(&text)
                .map(|timestamp| Timestamp::parse(&timestamp[1]))
                .collect(),
        }
    }

    pub fn parse_and_append(&mut self, text: &str) {
        *self += Timestamps::parse(text);
    }

    pub fn contains_date(&self, date: &Date<Local>) -> bool {
        self.timestamps.iter().find(|timestamp| timestamp.date == Some(*date)).is_some() ||
            self.inactive_timestamps.iter().find(|timestamp| timestamp.date == Some(*date)).is_some()
    }

    pub fn contains_active_date(&self, date: &Date<Local>) -> bool {
        self.timestamps.iter().find(|timestamp| timestamp.date == Some(*date)).is_some()
    }
}

impl Add for Timestamps {
    type Output = Timestamps;

    fn add(mut self, rhs: Timestamps) -> Self::Output {
        self.timestamps.extend(rhs.timestamps.into_iter());
        self.inactive_timestamps.extend(rhs.inactive_timestamps.into_iter());

        self
    }
}

impl AddAssign for Timestamps {
    fn add_assign(&mut self, rhs: Self) {
        self.timestamps.extend(rhs.timestamps.into_iter());
        self.inactive_timestamps.extend(rhs.inactive_timestamps.into_iter());
    }
}

#[derive(Clone)]
pub struct Timestamp {
    string: String,
    date: Option<Date<Local>>,
    time: Option<NaiveTime>,
    end_time: Option<NaiveTime>,
}

impl Timestamp {
    fn parse(timestamp: &str) -> Self {
        lazy_static! {
            static ref REGEX: Regex = Regex::new(r#"(?x)
                ^
                (?P<year>\d+)-(?P<month>\d+)-(?P<day>\d+)
                (\s+\w+)?
                (\s+(?P<hour>\d+):(?P<minute>\d+)
                    (\s+(?P<pm>AM|PM|am|pm))?)?
                $
            "#).unwrap();
        }

        if let Some(captures) = REGEX.captures(timestamp) {
            let year = captures.name("year").unwrap().as_str().parse().unwrap();
            let month = captures.name("month").unwrap().as_str().parse().unwrap();
            let day = captures.name("day").unwrap().as_str().parse().unwrap();

            let date = Local.ymd(year, month, day);

            let time = if let Some(hour) = captures.name("hour") {
                let mut hour = hour.as_str().parse().unwrap();
                let minute = captures.name("minute").unwrap().as_str().parse().unwrap();

                let am_pm = captures.name("pm");

                if am_pm.is_some() {
                    let am_pm = am_pm.unwrap().as_str();
                    let pm = am_pm == "pm" || am_pm == "PM";

                    if hour == 12 {
                        hour -= 12;
                    }

                    if pm {
                        hour += 12;
                    }
                }

                Some(NaiveTime::from_hms(hour, minute, 0))
            } else {
                None
            };

            Timestamp {
                string: timestamp.to_string(),
                date: Some(date),
                time,
                end_time: None,
            }
        } else {
            Timestamp {
                string: timestamp.to_string(),
                date: None,
                time: None,
                end_time: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date() {
        let timestamp = Timestamp::parse("2018-09-15");

        assert_eq!(timestamp.date, Some(Local.ymd(2018, 9, 15)));
    }

    #[test]
    fn test_parse_date_and_day() {
        let timestamp = Timestamp::parse("2018-09-15 Sat");

        assert_eq!(timestamp.date, Some(Local.ymd(2018, 9, 15)));
    }

    #[test]
    fn test_parse_date_and_time() {
        let timestamp = Timestamp::parse("2018-09-15 Sat 10:48 AM");
        assert_eq!(timestamp.date, Some(Local.ymd(2018, 9, 15)));
        assert_eq!(timestamp.time, Some(NaiveTime::from_hms(10, 48, 0)));

        let timestamp = Timestamp::parse("2018-09-15 12:30 PM");
        assert_eq!(timestamp.time, Some(NaiveTime::from_hms(12, 30, 0)));

        let timestamp = Timestamp::parse("2018-09-15 dss 12:00 AM");
        assert_eq!(timestamp.time, Some(NaiveTime::from_hms(0, 0, 0)));

        let timestamp = Timestamp::parse("2018-09-15 Sat 05:00");
        assert_eq!(timestamp.time, Some(NaiveTime::from_hms(5, 0, 0)));
    }
}
