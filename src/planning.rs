use regex::Regex;
use timestamp::Timestamp;
use parser::Error;

/// See <https://orgmode.org/worg/dev/org-syntax.html#Clock,_Diary_Sexp_and_Planning>
#[derive(Debug, PartialEq)]
pub struct Planning {
    pub line: String,
    pub deadline: Option<Timestamp>,
    pub scheduled: Option<Timestamp>,
    pub closed: Option<Timestamp>
}

impl Planning {
    pub fn parse(line: &str) -> Result<Option<Planning>, Error> {
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

        let planning = if COMBINED_REGEX.is_match(line) {
            let mut planning = Planning {
                line: line.to_string(),
                deadline: None,
                scheduled: None,
                closed: None
            };

            for captures in ONE_REGEX.captures_iter(line) {
                let keyword = captures.name("keyword").unwrap().as_str();
                let timestamp = captures.name("timestamp").unwrap().as_str();
                let timestamp = Timestamp::parse(timestamp)?;

                match keyword {
                    "DEADLINE" => {
                        if planning.deadline.is_some() {
                            println!("WARNING: deadline is already set");
                        }
                        planning.deadline = Some(timestamp);
                    },
                    "SCHEDULED" => {
                        if planning.scheduled.is_some() {
                            println!("WARNING: scheduled is already set");
                        }
                        planning.scheduled = Some(timestamp);
                    },
                    "CLOSED" => {
                        if planning.closed.is_some() {
                            println!("WARNING: closed is already set");
                        }
                        planning.closed = Some(timestamp);
                    },
                    _ => panic!("Unexpected keyword: {}", keyword)
                }
            }

            Some(planning)
        } else {
            None
        };

        Ok(planning)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_all() {
        let line = "CLOSED: [2018-10-27 Sat 20:09] SCHEDULED: <2018-09-15 Sat 10:48 AM> DEADLINE: <2018-10-31 Wed>";

        // NOTE: this assumes that timestamp parsing is tested elsewhere
        assert_eq!(Planning::parse(line).unwrap(), Some(Planning {
            line: line.to_string(),
            deadline: Some(Timestamp::parse("<2018-10-31 Wed>").unwrap()),
            scheduled: Some(Timestamp::parse("<2018-09-15 Sat 10:48 AM>").unwrap()),
            closed: Some(Timestamp::parse("[2018-10-27 Sat 20:09]").unwrap()),
        }))
    }
}
