use regex::Regex;
use timestamp::{Timestamp, TimestampKind};
use std::fmt;

/// See <https://orgmode.org/worg/dev/org-syntax.html#Clock,_Diary_Sexp_and_Planning>
#[derive(Debug, PartialEq)]
pub struct Planning {
    pub scheduled: Option<Timestamp>,
    pub deadline: Option<Timestamp>,
    pub closed: Option<Timestamp>
}

impl Planning {
    pub fn parse(line: &str) -> Option<Planning> {
        lazy_static! {
            static ref ONE_REGEX: Regex = Regex::new(r#"(?x)
                (?P<keyword>DEADLINE|SCHEDULED|CLOSED):
                \s+
                (?P<timestamp>[<\[][^>\]]*[>\]](--[<\[][^>\]]*[>\]])?)
            "#).unwrap();
            static ref COMBINED_REGEX: Regex = Regex::new(r#"(?x)
                ^
                (
                    \b
                    (DEADLINE|SCHEDULED|CLOSED):
                    \s+
                    [<\[][^>\]]*[>\]](--[<\[][^>\]]*[>\]])?
                    \s*
                )+
                $
            "#).unwrap();
        }

        if COMBINED_REGEX.is_match(line) {
            let mut planning = Planning {
                deadline: None,
                scheduled: None,
                closed: None
            };

            for captures in ONE_REGEX.captures_iter(line) {
                let keyword = captures.name("keyword").unwrap().as_str();
                let timestamp = captures.name("timestamp").unwrap().as_str();
                let mut timestamp = match Timestamp::parse(timestamp) {
                    Some(timestamp) => timestamp,
                    None => continue
                };

                match keyword {
                    "DEADLINE" => {
                        if planning.deadline.is_some() {
                            org_warning!("deadline is already set");
                        }
                        timestamp.kind = TimestampKind::Deadline;
                        planning.deadline = Some(timestamp);
                    },
                    "SCHEDULED" => {
                        if planning.scheduled.is_some() {
                            org_warning!("scheduled is already set");
                        }
                        timestamp.kind = TimestampKind::Scheduled;
                        planning.scheduled = Some(timestamp);
                    },
                    "CLOSED" => {
                        if planning.closed.is_some() {
                            org_warning!("closed is already set");
                        }
                        timestamp.kind = TimestampKind::Closed;
                        planning.closed = Some(timestamp);
                    },
                    _ => panic!("Unexpected keyword: {}", keyword)
                }
            }

            Some(planning)
        } else {
            None
        }
    }
}

impl fmt::Display for Planning {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut line = Vec::new();
        if let Some(ref scheduled) = self.scheduled {
            line.push(format!("SCHEDULED: {}", scheduled));
        }
        if let Some(ref deadline) = self.deadline {
            line.push(format!("DEADLINE: {}", deadline));
        }
        if let Some(ref closed) = self.closed {
            line.push(format!("CLOSED: {}", closed));
        }
        write!(f, "{}", line.join(" "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_all() {
        let line = "CLOSED: [2018-10-27 Sat 20:09] SCHEDULED: <2018-09-15 Sat 10:48 AM> DEADLINE: <2018-10-31 Wed>";

        // NOTE: this assumes that timestamp parsing is tested elsewhere
        assert_eq!(Planning::parse(line), Some(Planning {
            deadline: Some(Timestamp::parse("<2018-10-31 Wed>").unwrap()),
            scheduled: Some(Timestamp::parse("<2018-09-15 Sat 10:48 AM>").unwrap()),
            closed: Some(Timestamp::parse("[2018-10-27 Sat 20:09]").unwrap()),
        }))
    }
}
