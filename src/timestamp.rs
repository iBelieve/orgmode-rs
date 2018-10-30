use chrono;
use regex::Regex;
use parser::Error;
use std::fmt;

pub type Date = chrono::NaiveDate;
pub type Time = chrono::NaiveTime;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Timestamp {
    pub date: Date,
    pub end_date: Option<Date>,
    pub time: Option<Time>,
    pub end_time: Option<Time>,
    pub is_active: bool,
    pub repeater: Option<Repeater>,
    pub delay: Option<Delay>
}

pub struct TimestampPart {
    date: Date,
    time: Option<Time>,
    end_time: Option<Time>,
    is_active: bool,
    repeater: Option<Repeater>,
    delay: Option<Delay>
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum TimeUnit {
    Hour,
    Day,
    Week,
    Month,
    Year
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum RepeaterMark {
    Cumulate,
    CatchUp,
    Restart
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Repeater {
    pub mark: RepeaterMark,
    pub value: u32,
    pub unit: TimeUnit
}

#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub enum DelayMark {
    All,
    First
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Delay {
    pub mark: DelayMark,
    pub value: u32,
    pub unit: TimeUnit
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
    pub fn parse(timestamp: &str) -> Result<Self, Error> {
        let (start, end) = if let Some(index) = timestamp.find("--") {
            let (start, end) = timestamp.split_at(index);
            (start, Some(&end[1..]))
        } else {
            (timestamp, None)
        };
        let start = parse_timestamp(start)?;
        let end = if let Some(end) = end {
            let end = parse_timestamp(end)?;
            if end.repeater.is_some() {
                println!("WARNING: Ending timestamp should not have a repeater");
            }
            if end.delay.is_some() {
                println!("WARNING: Ending timestamp should not have a delay");
            }
            Some(end)
        } else {
            None
        };
        Ok(Timestamp {
            date: start.date,
            end_date: end.as_ref().map(|end| end.date),
            time: start.time,
            end_time: end.and_then(|end| end.end_time).or(start.end_time),
            is_active: start.is_active,
            repeater: start.repeater,
            delay: start.delay
        })
    }

    pub fn contains(&self, date: &Date) -> bool {
        if let Some(ref end_date) = self.end_date {
            date >= &self.date && date <= end_date
        } else {
            date == &self.date
        }
    }

    pub fn is_past(&self) -> bool {
        self.date < today()
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", if self.is_active { "<" } else { "[" })?;
        write!(f, "{}", if self.is_active { ">" } else { "]" })?;
        Ok(())
    }
}

fn parse_timestamp(timestamp: &str) -> Result<TimestampPart, Error> {
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

    let captures = DATE_REGEX.captures(timestamp).unwrap();
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
        let value = captures.name("value").unwrap().as_str().parse().unwrap();
        let unit = captures.name("unit").unwrap().as_str();

        let mark = match mark {
            "+" => RepeaterMark::Cumulate,
            "++" => RepeaterMark::CatchUp,
            ".+" => RepeaterMark::Restart,
            _ => panic!("Unexpected repeater mark: {}", mark)
        };
        let unit = parse_unit(unit);

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

    Ok(TimestampPart {
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
