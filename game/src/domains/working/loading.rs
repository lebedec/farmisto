use crate::working::{Device, WorkingDomain};

impl WorkingDomain {
    pub fn load_devices(&mut self, devices: Vec<Device>, sequence: usize) {
        self.devices_id.set(sequence);
        self.devices.extend(devices);
    }
}
