use crate::working::{
    DeviceId, DeviceMode, Working, WorkingDomain, WorkingError,
};
use crate::working::Working::DeviceUpdated;

impl WorkingDomain {
    pub fn repair_device(
        &mut self,
        id: DeviceId,
    ) -> Result<impl FnOnce() -> Vec<Working> + '_, WorkingError> {
        let device = self.get_device_mut(id)?;
        let command = move || {
            device.deprecation = 0.0;
            device.mode = DeviceMode::Stopped;
            vec![DeviceUpdated {
                device: device.id,
                mode: device.mode,
                resource: device.resource,
                progress: device.progress,
                deprecation: device.deprecation,
            }]
        };
        Ok(command)
    }
}
