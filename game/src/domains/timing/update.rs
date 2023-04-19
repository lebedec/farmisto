use crate::timing::{Timing, TimingDomain};

impl TimingDomain {
    pub fn update(&mut self, real_seconds: f32, speed: f32) -> Vec<Timing> {
        let colonization_date_delta = (real_seconds * speed) / self.real_seconds_per_mgm;
        self.colonization_date += colonization_date_delta;
        let mut events = vec![Timing::TimeUpdated {
            colonization_date: self.colonization_date,
            speed,
        }];
        for calendar in self.calendars.iter_mut() {
            calendar.times_of_day += colonization_date_delta / calendar.kind.day_duration.as_f32();
            while calendar.times_of_day >= 1.0 {
                calendar.times_of_day -= 1.0;
            }
            calendar.season_day += colonization_date_delta;
            let season = &calendar.kind.seasons[calendar.season as usize];
            if calendar.season_day >= season.duration.as_f32() {
                calendar.season_day -= season.duration.as_f32();
                calendar.season = (calendar.season + 1) % calendar.kind.seasons.len() as u8;
            }
            events.push(Timing::CalendarUpdated {
                id: calendar.id,
                season: calendar.season,
                season_day: calendar.season_day,
                times_of_day: calendar.times_of_day,
            })
        }
        events
    }
}
