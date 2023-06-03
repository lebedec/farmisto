use crate::collections::Shared;
use crate::timing::{Calendar, CalendarId, CalendarKind, Timing, TimingDomain, TimingError};

impl TimingDomain {
    pub fn create_calendar(
        &mut self,
        kind: &Shared<CalendarKind>,
    ) -> Result<(CalendarId, impl FnOnce() -> Vec<Timing> + '_), TimingError> {
        let id = self.calendars_id.introduce().one(CalendarId);
        let space = Calendar {
            id,
            kind: kind.clone(),
            season: 0,
            season_day: 0.0,
            times_of_day: 0.0,
        };
        let command = move || {
            let events = vec![];
            self.calendars_id.register(id.0);
            self.calendars.push(space);
            events
        };
        Ok((id, command))
    }
}
