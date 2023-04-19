use crate::timing::{Calendar, CalendarId, TimingDomain, TimingError};

impl TimingDomain {
    pub fn get_calendar(&self, id: CalendarId) -> Result<&Calendar, TimingError> {
        self.calendars
            .iter()
            .find(|calendar| calendar.id == id)
            .ok_or(TimingError::CalendarNotFound { id })
    }
}
