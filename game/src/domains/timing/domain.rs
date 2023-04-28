pub use crate::collections::{Sequence, Shared};
use serde::{Deserialize, Serialize};

pub struct TimingDomain {
    pub real_seconds_per_mgm: f32,
    pub colonization_date: f32,
    pub speed: f32,
    pub calendars_id: Sequence,
    pub calendars: Vec<Calendar>,
}

impl Default for TimingDomain {
    fn default() -> Self {
        Self {
            real_seconds_per_mgm: 60.0,
            colonization_date: 0.0,
            speed: 1.0,
            calendars_id: Sequence::default(),
            calendars: vec![],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CalendarKey(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub struct MinGameMinute(pub u8);

impl MinGameMinute {
    pub fn as_f32(&self) -> f32 {
        self.0 as f32
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Season {
    pub duration: MinGameMinute,
    pub key: String,
}

pub struct CalendarKind {
    pub id: CalendarKey,
    pub name: String,
    pub day_duration: MinGameMinute,
    pub seasons: Vec<Season>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CalendarId(pub usize);

pub struct Calendar {
    pub id: CalendarId,
    pub kind: Shared<CalendarKind>,
    pub season: u8,
    pub season_day: f32,
    pub times_of_day: f32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Timing {
    TimeUpdated {
        colonization_date: f32,
        speed: f32,
    },
    CalendarUpdated {
        id: CalendarId,
        season: u8,
        season_day: f32,
        times_of_day: f32,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TimingError {
    CalendarNotFound { id: CalendarId },
}
