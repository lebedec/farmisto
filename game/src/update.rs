use rand::thread_rng;

use crate::api::Event;
use crate::inventory::ItemId;
use crate::math::VectorMath;
use crate::{occur, Game};
use crate::model::Activity;

impl Game {
    pub fn update(&mut self, real_seconds: f32) -> Vec<Event> {
        let mut boosts = vec![];
        for player in self.players.iter() {
            let farmer = self.universe.get_player_farmer(player.id).unwrap();
            let activity = self.universe.get_farmer_activity(farmer).unwrap();
            let boost = match activity {
                Activity::Rest { comfort } => comfort,
                _ => 1
            };
            boosts.push(boost);
        }
        let speed = boosts.iter().max().cloned().unwrap_or(1) as f32;
        let time = real_seconds * speed;

        let timing_events = self.timing.update(real_seconds, speed);
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
            // TODO: transactional with working
            // TODO: errors, unwraps
            let has_input = !self
                .inventory
                .get_container(cementer.input)
                .unwrap()
                .items
                .is_empty();
            if has_input {
                let consumed = self.working.consume_input(cementer.device).unwrap();
                if consumed {
                    let use_items = self.inventory.pop_use_item(cementer.input).unwrap();
                    cementer_events.push(use_items().into());
                }
            }
            let output = self.inventory.get_container(cementer.output).unwrap();
            let can_output = output.items.len() < output.kind.capacity;
            if can_output {
                let produced = self.working.produce_output(cementer.device).unwrap();
                if produced {
                    let cementer_kind = self.known.cementers.get(cementer.key).unwrap();
                    let item = self.inventory.items_id.introduce().one(ItemId);
                    let create_item = self
                        .inventory
                        .create_item(item, &cementer_kind.cement, cementer.output, 1)
                        .unwrap();
                    cementer_events.push(create_item().into())
                }
            }
        }

        let mut random = thread_rng();
        let working_events = self.working.update(time, random.clone());

        let mut events = occur![
            timing_events,
            physics_events,
            self.planting.update(time),
            self.landscaping.update(time, random),
            working_events,
        ];
        events.extend(cementer_events);
        events.extend(destroy_empty_stacks);
        events
    }
}
