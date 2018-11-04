use chrono;
use regex::Regex;
use std::fmt;
use std::cmp::Ordering;
use chrono::Datelike;

pub type Date = chrono::NaiveDate;
pub type Time = chrono::NaiveTime;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timestamp {
    pub date: Date,
    pub end_date: Option<Date>,
    pub time: Option<Time>,
    pub end_time: Option<Time>,
    pub kind: TimestampKind,
    pub repeater: Option<Repeater>,
    pub delay: Option<Delay>
}

impl Ord for Timestamp {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.date == other.date {
            let time = self.time.unwrap_or_else(|| Time::from_hms(0, 0, 0));
            let other_time = other.time.unwrap_or_else(|| Time::from_hms(0, 0, 0));

            time.cmp(&other_time)
        } else {
            self.date.cmp(&other.date)
        }
    }
}

impl PartialOrd for Timestamp {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Timestamp {
    fn eq(&self, other: &Self) -> bool {
        (&self.date, &self.time, &self.end_date, &self.end_time) ==
            (&other.date, &other.time, &other.end_date, &other.end_time)
    }
}

impl PartialEq<Date> for Timestamp {
    fn eq(&self, other: &Date) -> bool {
        if let Some(ref end_date) = self.end_date {
            other >= &self.date && other <= end_date
        } else {
            other == &self.date
        }
    }
}

impl Eq for Timestamp {}

pub struct TimestampPart {
    date: Date,
    time: Option<Time>,
    end_time: Option<Time>,
    is_active: bool,
    repeater: Option<Repeater>,
    delay: Option<Delay>
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub enum TimeUnit {
    Hour,
    Day,
    Week,
    Month,
    Year
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum RepeaterMark {
    Cumulate,
    CatchUp,
    Restart
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Repeater {
    pub mark: RepeaterMark,
    pub value: u32,
    pub unit: TimeUnit
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum DelayMark {
    All,
    First
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Delay {
    pub mark: DelayMark,
    pub value: u32,
    pub unit: TimeUnit
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum TimestampKind {
    Scheduled,
    Deadline,
    Closed,
    Active,
    Inactive
}

/// Valid timestamps:
///
/// <%%(SEXP)>                                                     (diary)
/// <DATE TIME REPEATER-OR-DELAY>                                  (active)
/// [DATE TIME REPEATER-OR-DELAY]                                  (inactive)
/// <DATE TIME REPEATER-OR-DELAY>--<DATE TIME REPEATER-OR-DELAY>   (active range)
/// <DATE TIME-TIME REPEATER-OR-DELAY>                             (active range)
/// [DATE TIME REPEATER-OR-DELAY]--[DATE TIME REPEATER-OR-DELAY]   (inactive range)
/// [DATE TIME-TIME REPEATER-OR-DELAY]                             (inactive range)
impl Timestamp {
    pub fn parse(timestamp: &str) -> Option<Self> {
        let (start, end) = if let Some(index) = timestamp.find("--") {
            let (start, end) = timestamp.split_at(index);
            (start, Some(&end[2..]))
        } else {
            (timestamp, None)
        };
        let mut start = parse_timestamp(start)?;
        let end = if let Some(end) = end {
            let end = parse_timestamp(end)?;
            if start.repeater.is_some() || end.repeater.is_some() {
                org_warning!("Multi-day repeating timestamps are not supported");
                start.repeater = None;
            }
            if end.delay.is_some() {
                org_warning!("Ending timestamp should not have a delay");
            }
            Some(end)
        } else {
            None
        };
        Some(Timestamp {
            date: start.date,
            end_date: end.as_ref().map(|end| end.date),
            time: start.time,
            end_time: end.and_then(|end| end.end_time).or(start.end_time),
            kind: if start.is_active { TimestampKind::Active } else { TimestampKind::Inactive },
            repeater: start.repeater,
            delay: start.delay
        })
    }

    pub fn matches(&self, date: &Date) -> bool {
        self.timestamp_for_date(date).is_some()
    }

    pub fn timestamp_for_date(&self, date: &Date) -> Option<Timestamp> {
        if !self.is_active() || date < &self.date {
            return None;
        }

        let today = today();

        if date <= &today && (self.kind == TimestampKind::Scheduled || self.kind == TimestampKind::Deadline) {
            if self == date {
                Some(self.clone())
            } else {
                None
            }
        } else {
            let on_today = if let Some(ref repeater) = self.repeater {
                if self.end_date.is_some() {
                    self == date
                } else {
                    let duration = date.signed_duration_since(self.date).num_days() as u32;

                    match repeater.unit {
                        TimeUnit::Year => {
                            self.date.month() == date.month() && self.date.day() == date.day() &&
                                ((date.year() - self.date.year()) as u32 % repeater.value == 0)
                        },
                        TimeUnit::Month => {
                            self.date.day() == date.day() &&
                                ((12 * (date.year() - self.date.year()) as u32 +
                                 (date.month() - self.date.month()) as u32) % repeater.value == 0)
                        },
                        TimeUnit::Day => {
                            duration % repeater.value == 0
                        },
                        TimeUnit::Week => {
                            duration % (7 * repeater.value) == 0
                        }
                        TimeUnit::Hour => panic!("Hourly repeaters are not supported"),
                    }
                }
            } else {
                self == date
            };

            if on_today {
                let mut timestamp = self.clone();
                timestamp.date = date.clone();
                Some(timestamp)
            } else {
                None
            }
        }
    }

    pub fn is_past(&self) -> bool {
        self.date < today()
    }

    pub fn is_active(&self) -> bool {
        match self.kind {
            TimestampKind::Scheduled | TimestampKind::Deadline | TimestampKind::Active => true,
            TimestampKind::Closed | TimestampKind::Inactive => false
        }
    }

    fn symbols(&self) -> (&str, &str) {
        if self.is_active() {
            ("<", ">")
        } else {
            ("[", "]")
        }
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (start, end) = self.symbols();
        write!(f, "{}", start)?;
        write!(f, "{}", self.date.format("%Y-%m-%d %a"))?;

        if let Some(time) = self.time {
            write!(f, " {}", time.format("%H:%M"))?;

            if let Some(end_time) = self.end_time {
                if self.end_date.is_none() {
                    write!(f, "-{}", end_time.format("%H:%M"))?;
                }
            }
        }

        write!(f, "{}", end)?;
        if let Some(end_date) = self.end_date {
            write!(f, "--{}", start)?;
            write!(f, "{}", end_date.format("%Y-%m-%d %a"))?;

            if let Some(end_time) = self.end_time {
                write!(f, "-{}", end_time.format("%H:%M"))?;
            }
            write!(f, "{}", end)?;
        }
        Ok(())
    }
}

fn parse_timestamp(timestamp: &str) -> Option<TimestampPart> {
    lazy_static! {
        static ref DATE_REGEX: Regex = Regex::new(r#"(?x)
            ^
            (?P<type>[<\[])
            (?P<year>\d+)-(?P<month>\d+)-(?P<day>\d+)
            (\s+(?P<dayname>[^ +\-\]>0-9]))?
        "#).unwrap();
        static ref TIME_REGEX: Regex = Regex::new(r#"(?x)
            (?P<hour>\d+):(?P<minute>\d+)
            (\s+(?P<pm>AM|PM|am|pm))?
            (
                -
                (?P<end_hour>\d+):(?P<end_minute>\d+)
                (\s+(?P<end_pm>AM|PM|am|pm))?
            )?
        "#).unwrap();
        // + (cumulate type), ++ (catch-up type) or .+ (restart type) for a repeater
        static ref REPEATER_REGEX: Regex = Regex::new(r#"(?x)
            (?P<mark>\+|\+\+|\.\+)
            (?P<value>\d+)
            (?P<unit>h|d|w|m|y)
        "#).unwrap();
        // - (all type) or -- (first type) for warning delays
        static ref DELAY_REGEX: Regex = Regex::new(r#"(?x)
            (?P<mark>-|--)
            (?P<value>\d+)
            (?P<unit>h|d|w|m|y)
        "#).unwrap();
    }

    let captures = match DATE_REGEX.captures(timestamp) {
        Some(captures) => captures,
        None => {
            org_warning!("Invalid date: {}", timestamp);
            return None;
        }
    };
    let is_active = captures.name("type").unwrap().as_str() == "<";
    let year = captures.name("year").unwrap().as_str().parse().unwrap();
    let month = captures.name("month").unwrap().as_str().parse().unwrap();
    let day = captures.name("day").unwrap().as_str().parse().unwrap();

    let date = Date::from_ymd(year, month, day);

    let (time, end_time) = if let Some(captures) = TIME_REGEX.captures(timestamp) {
        let start_time = time(captures.name("hour").unwrap().as_str(),
                        captures.name("minute").unwrap().as_str(),
                        captures.name("pm").map(|c|c.as_str()));

        let end_time = if let Some(end_hour) = captures.name("end_hour") {
            Some(time(end_hour.as_str(),
                      captures.name("end_minute").unwrap().as_str(),
                      captures.name("end_pm").map(|c|c.as_str())))
        } else {
            None
        };
        (Some(start_time), end_time)
    } else {
        (None, None)
    };

    let repeater = if let Some(captures) = REPEATER_REGEX.captures(timestamp) {
        let mark = captures.name("mark").unwrap().as_str();
        let mut value = captures.name("value").unwrap().as_str().parse().unwrap();
        let unit = captures.name("unit").unwrap().as_str();

        let mark = match mark {
            "+" => RepeaterMark::Cumulate,
            "++" => RepeaterMark::CatchUp,
            ".+" => RepeaterMark::Restart,
            _ => panic!("Unexpected repeater mark: {}", mark)
        };
        let mut unit = parse_unit(unit);

        if unit == TimeUnit::Hour {
            org_warning!("Hourly repeaters are not supported");
            unit = TimeUnit::Day;
            value = 1;
        }

        Some(Repeater {
            mark,
            value,
            unit
        })
    } else {
        None
    };

    let delay = if let Some(captures) = DELAY_REGEX.captures(timestamp) {
        let mark = captures.name("mark").unwrap().as_str();
        let value = captures.name("value").unwrap().as_str().parse().unwrap();
        let unit = captures.name("unit").unwrap().as_str();

        let mark = match mark {
            "-" => DelayMark::All,
            "--" => DelayMark::First,
            _ => panic!("Unexpected delay mark: {}", mark)
        };
        let unit = parse_unit(unit);

        Some(Delay {
            mark,
            value,
            unit
        })
    } else {
        None
    };

    Some(TimestampPart {
        date,
        time,
        end_time,
        is_active,
        repeater,
        delay
    })
}

fn parse_unit(unit: &str) -> TimeUnit {
    match unit {
        "h" => TimeUnit::Hour,
        "d" => TimeUnit::Day,
        "w" => TimeUnit::Week,
        "m" => TimeUnit::Month,
        "y" => TimeUnit::Year,
        _ => panic!("Unexpected repeater/delay unit: {}", unit)
    }
}

fn time(hour: &str, minute: &str, am_pm: Option<&str>) -> Time {
    let mut hour = hour.parse().unwrap();
    let minute = minute.parse().unwrap();

    if let Some(am_pm) = am_pm {
        let pm = am_pm == "pm" || am_pm == "PM";

        if hour == 12 {
            hour -= 12;
        }

        if pm {
            hour += 12;
        }
    }

    Time::from_hms(hour, minute, 0)
}

pub fn today() -> Date {
    chrono::Local::today().naive_local()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_date() {
        let timestamp = Timestamp::parse("<2018-09-15>").unwrap();

        assert_eq!(timestamp.date, Date::from_ymd(2018, 9, 15));
    }

    #[test]
    fn test_parse_date_and_day() {
        let timestamp = Timestamp::parse("[2018-09-15 Sat]").unwrap();

        assert_eq!(timestamp.date, Date::from_ymd(2018, 9, 15));
    }

    #[test]
    fn test_parse_date_and_time() {
        let timestamp = Timestamp::parse("<2018-09-15 Sat 10:48 AM>").unwrap();
        assert_eq!(timestamp.date, Date::from_ymd(2018, 9, 15));
        assert_eq!(timestamp.time, Some(Time::from_hms(10, 48, 0)));

        let timestamp = Timestamp::parse("<2018-09-15 12:30 PM>").unwrap();
        assert_eq!(timestamp.time, Some(Time::from_hms(12, 30, 0)));

        let timestamp = Timestamp::parse("[2018-09-15 dss 12:00 AM]").unwrap();
        assert_eq!(timestamp.time, Some(Time::from_hms(0, 0, 0)));

        let timestamp = Timestamp::parse("[2018-09-15 Sat 05:00]").unwrap();
        assert_eq!(timestamp.time, Some(Time::from_hms(5, 0, 0)));
    }
}
