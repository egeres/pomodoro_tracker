use chrono::{DateTime, Local};
use std::cmp::Ordering; // TimeZone, NaiveDateTime

#[derive(Debug, Clone)]
pub struct Segment {
    pub name: String,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    pub type_of_event: Option<String>,
}

impl Ord for Segment {
    // https://doc.rust-lang.org/std/cmp/trait.Ord.html
    fn cmp(&self, other: &Self) -> Ordering {
        self.start.cmp(&other.start)
    }
}

impl PartialOrd for Segment {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Segment {
    fn eq(&self, other: &Self) -> bool {
        (self.start == other.start)
            && (self.name == other.name)
            && (self.end == other.end)
            && (self.type_of_event == other.type_of_event)
    }
}

impl Eq for Segment {}
