use log::info;
use rand::thread_rng;
use std::thread::current;

use crate::api::Event;
use crate::inventory::ItemId;
use crate::math::{Random, TileMath, VectorMath};
use crate::model::Activity;
use crate::{occur, Game};

impl Game {
    pub fn update(&mut self, real_seconds: f32) -> Vec<Event> {
        let mut boosts = vec![];
        for player in self.players.iter() {
            let farmer = self.universe.get_player_farmer(player.id).unwrap();
            let activity = self.universe.get_farmer_activity(farmer).unwrap();
            let boost = match activity {
                Activity::Resting { comfort } => comfort,
                _ => 1,
            };
            boosts.push(boost);
        }
        let game_speed = boosts.iter().max().cloned().unwrap_or(1) as f32;
        let physics_time = real_seconds * game_speed;
        let time = self.timing.get_colonization_date(real_seconds, game_speed);

        let timing_events = self.timing.update(time, game_speed);
        let physics_events = self.physics.update(physics_time);

        // Change farmer activity after item usage
        // HACK: to eliminate boilerplate code from actions before new activity system will created
        let mut to_change = vec![];
        for farmer in self.universe.farmers.iter() {
            if self.universe.get_farmer_activity(*farmer).unwrap() == Activity::Usage {
                let hands = self.inventory.get_container(farmer.hands).unwrap();
                if hands.items.len() == 0 {
                    to_change.push((*farmer, Activity::Idle));
                }
            }
        }
        let mut activity_events = vec![];
        for (farmer, activity) in to_change {
            let event = self.universe.change_activity(farmer, activity);
            activity_events.push(event);
        }

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

        // TODO: optimize by farmland
        for crop in &self.universe.crops {
            let sensor = self.physics.get_sensor(crop.sensor).unwrap();
            let farmland = self.universe.get_farmland_by_space(sensor.space).unwrap();
            let farmland_kind = self.known.farmlands.get(farmland.kind).unwrap();
            let place = sensor.position.to_tile().fit(farmland_kind.land.width);
            let plant = self.planting.get_plant(crop.plant).unwrap();
            let plant_transpiration = plant.kind.transpiration;

            let mut impact = [0.0, 0.0];
            for signal in &sensor.signals {
                impact = impact.add(*signal);
            }
            impact = impact.normalize().neg();

            let consumption = plant_transpiration * physics_time;
            let consumed = self
                .landscaping
                .request_consumption(farmland.land, place, consumption)
                .unwrap();
            let lack = consumption - consumed;
            self.planting
                .integrate_thirst(crop.plant, lack, consumption)
                .unwrap();

            let consumption = (1.0 / (360.0 * 3.0)) * physics_time;
            let consumed = self
                .planting
                .request_fertility_consumption(farmland.soil, place, consumption)
                .unwrap();
            let lack = consumption - consumed;
            self.planting
                .integrate_hunger(crop.plant, lack, consumption)
                .unwrap();

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

        let mut composter_events = vec![];
        for composter in &self.universe.composters {
            // TODO: transactional with working
            // TODO: errors, unwraps
            // TODO: generalize with cementer ?
            let has_input = !self
                .inventory
                .get_container(composter.input)
                .unwrap()
                .items
                .is_empty();
            if has_input {
                let consumed = self.working.consume_input(composter.device).unwrap();
                if consumed {
                    let use_items = self.inventory.pop_use_item(composter.input).unwrap();
                    composter_events.push(use_items().into());
                }
            }
            let output = self.inventory.get_container(composter.output).unwrap();
            let can_output = output.items.len() < output.kind.capacity;
            if can_output {
                let produced = self.working.produce_output(composter.device).unwrap();
                if produced {
                    let composter_kind = self.known.composters.get(composter.key).unwrap();
                    let item = self.inventory.items_id.introduce().one(ItemId);
                    let create_item = self
                        .inventory
                        .create_item(
                            item,
                            &composter_kind.compost,
                            composter.output,
                            composter_kind.compost.max_quantity,
                        )
                        .unwrap();
                    composter_events.push(create_item().into())
                }
            }
        }

        let mut deprecated_random = thread_rng();
        let working_events = self.working.update(physics_time, deprecated_random.clone());
        let mut random = &mut Random::new();

        let raising_events = self.raising.update(time, random);
        let dead_animals = self.raising.take_dead_animals();
        let mut dead_animals_events = vec![];
        for animal in dead_animals {
            let creature = self.universe.get_creature_by_animal(animal.id).unwrap();
            let creature_kind = self.known.creatures.get(creature.key).unwrap();
            let body = self.physics.get_body(creature.body).unwrap();
            let position = body.position;
            let space = body.space;
            let events = self.universe.vanish_creature(creature);
            dead_animals_events.push(events.into());

            let corpse_kind = &creature_kind.corpse;
            let (barrier, create_barrier) = self
                .physics
                .create_barrier(space, corpse_kind.barrier.clone(), position, true, false)
                .unwrap();
            dead_animals_events.extend(occur![
                create_barrier(),
                self.appear_corpse(corpse_kind.id, barrier).unwrap(),
            ]);
        }

        let mut events = occur![
            timing_events,
            activity_events,
            physics_events,
            self.planting.update(physics_time),
            raising_events,
            self.landscaping.update(physics_time, deprecated_random),
            working_events,
        ];
        events.extend(cementer_events);
        events.extend(composter_events);
        events.extend(destroy_empty_stacks);
        events.extend(dead_animals_events);
        events
    }
}
