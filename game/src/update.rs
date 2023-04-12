use rand::thread_rng;

use crate::api::Event;
use crate::math::VectorMath;
use crate::working::{Process, Working};
use crate::{occur, Game};

impl Game {
    pub fn update(&mut self, time: f32) -> Vec<Event> {
        let physics_events = self.physics.update(time);

        let mut destroy_empty_stacks = vec![];
        for stack in self.universe.stacks.clone() {
            let container = self.inventory.get_container(stack.container).unwrap();
            if container.items.is_empty() {
                let destroy_container = self
                    .inventory
                    .destroy_containers(vec![stack.container], false)
                    .unwrap();
                let destroy_barrier = self.physics.destroy_barrier(stack.barrier).unwrap();
                destroy_empty_stacks.extend([
                    destroy_container().into(),
                    destroy_barrier().into(),
                    self.universe.vanish_stack(stack).into(),
                ])
            }
        }

        for crop in &self.universe.crops {
            let sensor = self.physics.get_sensor(crop.sensor).unwrap();
            let mut impact = [0.0, 0.0];
            for signal in &sensor.signals {
                impact = impact.add(*signal);
            }
            impact = impact.normalize().neg();

            self.planting
                .integrate_impact(crop.plant, impact[0])
                .unwrap();
        }

        let mut cementer_events = vec![];
        for cementer in &self.universe.cementers {
            let input = self.inventory.get_container(cementer.input).unwrap();
            if !input.items.is_empty() {
                let updated = self
                    .workingцв
                    .update_device_resource(cementer.device, 1)
                    .unwrap();
                if updated {
                    // TODO: transactional with working
                    let use_items = self.inventory.pop_use_item(cementer.input).unwrap();
                    cementer_events.push(use_items().into());
                }
            }
        }
        let mut random = thread_rng();

        let working_events = self.working.update(time, random);
        for event in &working_events {
            if let Working::ProcessCompleted { device, process } = event {
                match process {
                    Process::CementGeneration => {
                        let cementer = self
                            .universe
                            .cementers
                            .iter()
                            .find(|cementer| &cementer.device == device)
                            .unwrap();
                        let cementer_kind = self.known.cementers.get(cementer.key).unwrap();
                        // TODO: errors
                        let (_, create_item) = self
                            .inventory
                            .create_item(&cementer_kind.cement, cementer.output, 1)
                            .unwrap();
                        cementer_events.push(create_item().into())
                    }
                }
            }
        }

        let mut events = occur![physics_events, self.planting.update(time), working_events,];
        events.extend(cementer_events);
        events.extend(destroy_empty_stacks);
        events
    }
}
