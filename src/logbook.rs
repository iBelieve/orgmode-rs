use drawer::Drawer;
use timestamp::{Duration, Timestamp, today, TIMESTAMP_REGEX};

pub struct Logbook {
    entries: Vec<ClockEntry>,
}

impl Logbook {
    pub fn from(drawer: Option<&Drawer>) -> Logbook {
        let entries = if let Some(drawer) = drawer {
            drawer
                .contents
                .iter()
                .filter_map(|line| ClockEntry::parse(line))
                .collect()
        } else {
            Vec::new()
        };

        Logbook { entries }
    }

    pub fn time_spent(&self) -> Duration {
        self.entries
            .iter()
            .map(|entry| entry.time_spent())
            .fold(Duration::zero(), |total, duration| total + duration)
    }

    pub fn time_spent_today(&self) -> Duration {
        let today = today();
        self.entries
            .iter()
            .filter(|entry| entry.timestamp == today)
            .map(|entry| entry.time_spent())
            .fold(Duration::zero(), |total, duration| total + duration)
    }

    pub fn was_clocked_to_today(&self) -> bool {
        let today = today();
        self.entries.iter().any(|entry| entry.timestamp == today)
    }
}

pub struct ClockEntry {
    timestamp: Timestamp,
}

impl ClockEntry {
    pub fn time_spent(&self) -> Duration {
        self.timestamp.duration()
    }

    fn parse(line: &str) -> Option<ClockEntry> {
        if line.starts_with("CLOCK:") {
            if let Some(captures) = TIMESTAMP_REGEX.captures(line) {
                Timestamp::parse(&captures[0]).map(|timestamp| ClockEntry { timestamp })
            } else {
                None
            }
        } else {
            None
        }
    }
}
