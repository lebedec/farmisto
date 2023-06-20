use rand::Rng;

use crate::working::Working::DeviceUpdated;
use crate::working::{DeviceId, Working, WorkingDomain, WorkingError};

impl WorkingDomain {
    pub fn consume_input(&mut self, id: DeviceId) -> Result<bool, WorkingError> {
        let device = self.get_device_mut(id)?;
        let consumed = if device.enabled && !device.input {
            device.input = true;
            true
        } else {
            false
        };
        Ok(consumed)
    }

    pub fn produce_output(&mut self, id: DeviceId) -> Result<bool, WorkingError> {
        let device = self.get_device_mut(id)?;
        let produced = if device.output {
            device.output = false;
            true
        } else {
            false
        };
        Ok(produced)
    }

    pub fn update(&mut self, time: f32, mut random: impl Rng) -> Vec<Working> {
        let mut events = vec![];
        for device in self.devices.iter_mut() {
            if device.broken || !device.enabled {
                continue;
            }

            // running, but await result to be taken
            if device.output {
                continue;
            }

            // running, but await resources input
            if !device.input {
                continue;
            }

            let jam = random.gen_range(0.0..=1.0);
            let deprecation = device.deprecation / device.kind.durability;
            if deprecation > 0.25 && deprecation >= jam {
                device.broken = true;
            } else {
                device.progress += time;
                device.deprecation += time;
                if device.progress >= device.kind.duration {
                    device.progress -= device.kind.duration;
                    device.output = true;
                    device.input = false;
                }
            }

            events.push(DeviceUpdated {
                device: device.id,
                enabled: device.enabled,
                progress: device.progress,
                input: device.input,
                output: device.output,
                deprecation: device.deprecation,
                broken: device.broken,
            });
        }
        events
    }
}
