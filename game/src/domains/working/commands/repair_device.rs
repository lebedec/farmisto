
use crate::working::Working::DeviceUpdated;
use crate::working::{DeviceId,  Working, WorkingDomain, WorkingError};

impl WorkingDomain {
    pub fn repair_device(
        &mut self,
        id: DeviceId,
    ) -> Result<impl FnOnce() -> Vec<Working> + '_, WorkingError> {
        let device = self.get_device_mut(id)?;
        let command = move || {
            device.deprecation = 0.0;
            device.broken = false;
            vec![DeviceUpdated {
                device: device.id,
                enabled: device.enabled,
                broken: device.broken,
                progress: device.progress,
                input: device.output,
                output: device.output,
                deprecation: device.deprecation,
            }]
        };
        Ok(command)
    }
}
