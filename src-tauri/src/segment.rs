use chrono::{DateTime, Local, NaiveDate};
use std::cmp::Ordering; // TimeZone, NaiveDateTime

#[derive(Debug, Clone)]
pub struct Segment {
    pub name: String,
    pub start: DateTime<Local>,
    pub end: DateTime<Local>,
    // Serialized to JSON as "type".
    pub type_of_event: Option<String>,
    // Optional metadata. Preserved when loaded from disk, generated for new pomodoros.
    pub uuid: Option<String>,
    pub os_login: Option<String>,
    pub platform: Option<String>,
    pub machine_name: Option<String>,
    pub generated_by: Option<String>,
    pub way_this_info_was_added: Option<String>,
    pub datetime_of_annotation: Option<String>,
    // The date encoded in the file this segment was loaded from (parsed from the
    // `YYYY-MM-DD_pomodoro.json` filename). `None` for freshly created segments
    // that have not been written to disk yet. Used to detect "misfiled" segments
    // whose day file no longer matches their UTC date, so the save logic knows
    // which files actually need rewriting. Not part of equality/ordering.
    pub source_date: Option<NaiveDate>,
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
