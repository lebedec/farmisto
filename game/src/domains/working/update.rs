use crate::working::Working::{DeviceUpdated, ProcessCompleted};
use crate::working::{DeviceId, DeviceMode, Working, WorkingDomain, WorkingError};
use log::info;
use rand::Rng;

impl WorkingDomain {
    pub fn update_device_resource(
        &mut self,
        id: DeviceId,
        resource: u8,
    ) -> Result<bool, WorkingError> {
        let device = self.get_device_mut(id)?;
        if device.resource == 0 && device.mode == DeviceMode::Pending {
            device.resource += resource;
            device.mode = DeviceMode::Running;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn update(&mut self, time: f32, mut random: impl Rng) -> Vec<Working> {
        let mut events = vec![];
        for device in self.devices.iter_mut() {
            if device.mode == DeviceMode::Stopped {
                continue;
            }

            if device.mode == DeviceMode::Pending {
                continue;
            }

            if device.mode == DeviceMode::Broken {
                continue;
            }

            let jam = random.gen_range(0.0..=1.0);
            let deprecation = device.deprecation / device.kind.durability;
            // info!(
            //     "df{deprecation} >= {jam} {} depr{} dura{}",
            //     deprecation >= jam,
            //     device.deprecation,
            //     device.kind.durability
            // );
            if deprecation > 0.25 && deprecation >= jam {
                device.mode = DeviceMode::Broken;
                events.push(DeviceUpdated {
                    device: device.id,
                    mode: device.mode,
                    resource: device.resource,
                    progress: device.progress,
                    deprecation: device.deprecation,
                });
                continue;
            }

            device.progress += time;
            device.deprecation += time;
            if device.progress >= device.kind.duration {
                device.progress -= device.kind.duration;
                events.push(ProcessCompleted {
                    device: device.id,
                    process: device.kind.process,
                });
                if device.resource > 0 {
                    device.resource -= 1;
                } else {
                    device.progress = 0.0;
                    device.mode = DeviceMode::Pending;
                }
            }
            events.push(DeviceUpdated {
                device: device.id,
                mode: device.mode,
                resource: device.resource,
                progress: device.progress,
                deprecation: device.deprecation,
            });
        }
        events
    }
}
