use std::ops::{Add, AddAssign};
use regex::Regex;
use timestamp::{Timestamp, Date};

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct Timestamps {
    pub timestamps: Vec<Timestamp>
}

impl Timestamps {
    pub fn parse(text: &str) -> Timestamps {
        lazy_static! {
            static ref REGEX: Regex = Regex::new(r#"(?x)
                [<\[] \d+-\d+-\d+ [^>\]]+ [>\]]
                (-- [<\[] \d+-\d+-\d+ [^>\]]+ [>\]])?
            "#).unwrap();
        }

        let mut timestamps: Vec<Timestamp> = REGEX.captures_iter(text)
            .filter_map(|timestamp| Timestamp::parse(&timestamp[0]))
            .collect();
        timestamps.sort();

        Timestamps {
            timestamps
        }
    }

    pub fn parse_and_append(&mut self, text: &str) {
        *self += Timestamps::parse(text);
    }

    pub fn matches_date(&self, date: &Date) -> bool {
        self.timestamps.iter().find(|timestamp| timestamp.matches(date)).is_some()
    }

    pub fn timestamps_for_date<'a>(&'a self, date: &'a Date) -> impl Iterator<Item=Timestamp> + 'a {
        self.timestamps.iter().filter_map(move |timestamp| timestamp.timestamp_for_date(date))
    }
}

impl Add for Timestamps {
    type Output = Timestamps;

    fn add(mut self, rhs: Timestamps) -> Self::Output {
        self.timestamps.extend(rhs.timestamps.into_iter());
        self.timestamps.sort();

        self
    }
}

impl AddAssign for Timestamps {
    fn add_assign(&mut self, rhs: Self) {
        self.timestamps.extend(rhs.timestamps.into_iter());
        self.timestamps.sort();
    }
}
