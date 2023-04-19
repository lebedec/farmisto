use crate::timing::{Calendar, TimingDomain};

impl TimingDomain {
    pub fn load_calendars(&mut self, calendars: Vec<Calendar>) {
        for calendar in &calendars {
            self.calendars_id.register(calendar.id.0);
        }
        self.calendars.extend(calendars);
    }
}
