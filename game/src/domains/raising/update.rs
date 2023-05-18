use crate::math::Random;
use crate::raising::Raising::{AnimalChanged, AnimalHealthChanged};
use crate::raising::{Animal, Raising, RaisingDomain};
use log::{error, info};
use std::collections::HashSet;
use std::mem::take;

impl RaisingDomain {
    pub fn take_dead_animals(&mut self) -> Vec<Animal> {
        take(&mut self.dead_animals)
    }

    pub fn update(&mut self, time: f32, random: &mut Random) -> Vec<Raising> {
        let mut events = vec![];
        let mut dead_animals = vec![];
        for animal in self.animals.iter_mut() {
            let kind = &animal.kind;
            let extra_hunger = random.max(animal.voracity) * time;
            animal.hunger = (animal.hunger + kind.hunger_speed * time + extra_hunger).min(1.0);
            animal.thirst = (animal.thirst + kind.thirst_speed * time).min(1.0);
            animal.age += time;
            if animal.hunger < 0.35 {
                animal.weight += time / 6.0;
                animal.weight = animal.weight.min(1.0);
            }
            if animal.hunger > 0.75 {
                animal.weight -= time / 6.0;
                animal.weight = animal.weight.max(0.0);
            }
            events.push(AnimalChanged {
                id: animal.id,
                hunger: animal.hunger,
                thirst: animal.thirst,
                age: animal.age,
                weight: animal.weight,
            });

            let mut health_change = false;
            let mut damage = 0.0;
            // if animal.thirst > 0.5 {
            //     damage += animal.thirst * kind.thirst_damage * time;
            // }
            if animal.hunger > 0.5 {
                damage += animal.hunger * kind.hunger_damage * time;
            }
            if animal.health < kind.death_threshold {
                damage += 0.2 * time;
            }
            if damage > 0.0 {
                health_change = true;
                animal.health = (animal.health - damage).max(0.0);
            }

            if animal.hunger < 0.5 && animal.health < 1.0 {
                health_change = true;
                animal.health = (animal.health + 0.2 * time).min(1.0);
            }

            if health_change {
                events.push(AnimalHealthChanged {
                    id: animal.id,
                    health: animal.health,
                })
            }

            if animal.health <= 0.0 {
                dead_animals.push(animal.id);
            }

            // info!(
            //     "Animal h{}+{extra_hunger} t{} health={}",
            //     animal.hunger, animal.thirst, animal.health
            // );
        }
        for id in dead_animals {
            let index = match self.animals.iter().position(|animal| animal.id == id) {
                Some(index) => index,
                None => {
                    error!("Unable to remove dead animal {id:?}, not found");
                    continue;
                }
            };
            let animal = self.animals.remove(index);
            self.dead_animals.push(animal);
        }
        events
    }
}
