use crate::engine::Frame;
use crate::gameplay::representation::{
    BarrierHint, BuildingRep, ConstructionRep, CreatureRep, CropRep, EquipmentRep, FarmerRep,
    FarmlandRep, StackRep, TreeRep,
};
use crate::gameplay::Gameplay;
use game::api::Event;
use game::building::{Building, Material};
use game::inventory::{Function, Inventory};
use game::model::{Activity, ItemRep, Universe};
use game::physics::Physics;
use game::planting::Planting;
use game::raising::Raising;
use log::{error, info};
use rusty_spine::Skin;
use std::collections::HashMap;

impl Gameplay {
    pub fn handle_event(&mut self, frame: &mut Frame, event: Event) {
        match event {
            Event::Universe(events) => {
                for event in events {
                    self.handle_universe_event(frame, event);
                }
            }
            Event::Physics(events) => {
                for event in events {
                    self.handle_physics_event(frame, event);
                }
            }
            Event::Building(events) => {
                for event in events {
                    self.handle_building_event(frame, event);
                }
            }
            Event::Inventory(events) => {
                for event in events {
                    self.handle_inventory_event(frame, event);
                }
            }
            Event::Planting(events) => {
                for event in events {
                    self.handle_planting_event(frame, event);
                }
            }
            Event::Raising(events) => {
                for event in events {
                    self.handle_raising_event(frame, event);
                }
            }
        }
    }

    pub fn handle_building_event(&mut self, frame: &mut Frame, event: Building) {
        let assets = &mut frame.assets;
        match event {
            Building::GridChanged { grid, cells, rooms } => {
                for (farmland, behaviour) in self.farmlands.iter_mut() {
                    if farmland.grid == grid {
                        behaviour.cells = cells;
                        behaviour.rooms = rooms;
                        break;
                    }
                }
            }
            Building::SurveyorCreated { .. } => {}
            Building::SurveyorDestroyed { .. } => {}
        }
    }

    pub fn handle_inventory_event(&mut self, frame: &mut Frame, event: Inventory) {
        match event {
            Inventory::ContainerCreated { id } => {}
            Inventory::ContainerDestroyed { id } => {
                self.items.remove(&id);
            }
            Inventory::ItemAdded {
                id,
                kind,
                container,
                functions,
                quantity,
            } => {
                info!("item added {:?} to {:?}", id, container);
                let items = self.items.entry(container).or_insert(HashMap::new());
                items.insert(
                    id,
                    ItemRep {
                        id,
                        kind,
                        container,
                        quantity,
                        functions,
                    },
                );
            }
            Inventory::ItemRemoved { item, container } => {
                info!("item removed {:?} from {:?}", item, container);
                self.items.entry(container).and_modify(|items| {
                    items.remove(&item);
                });
            }
            Inventory::ItemQuantityChanged {
                id,
                container,
                quantity,
            } => {
                self.items.entry(container).and_modify(|items| {
                    items.entry(id).and_modify(|item| item.quantity = quantity);
                });
            }
        }
    }

    pub fn handle_planting_event(&mut self, frame: &mut Frame, event: Planting) {
        let assets = &mut frame.assets;
        match event {
            Planting::SoilChanged { soil, map } => {
                for (farmland, behaviour) in self.farmlands.iter_mut() {
                    if farmland.soil == soil {
                        behaviour.soil_map = map;
                        break;
                    }
                }
            }
            Planting::PlantUpdated {
                id,
                impact,
                thirst,
                hunger,
                growth,
            } => {
                for crop in self.crops.values_mut() {
                    if crop.entity.plant == id {
                        crop.synchronize_impact(impact);
                        crop.synchronize_thirst(thirst);
                        crop.synchronize_thirst(hunger);
                        crop.synchronize_growth(growth);
                        break;
                    }
                }
            }
            Planting::PlantHarvested { id, fruits } => {
                for crop in self.crops.values_mut() {
                    if crop.entity.plant == id {
                        crop.synchronize_fruits(fruits);
                        break;
                    }
                }
            }
            Planting::PlantDamaged { id, health } => {
                for crop in self.crops.values_mut() {
                    if crop.entity.plant == id {
                        crop.synchronize_health(health);
                        break;
                    }
                }
            }
        }
    }

    pub fn handle_raising_event(&mut self, frame: &mut Frame, event: Raising) {
        let assets = &mut frame.assets;
        match event {
            Raising::AnimalChanged { .. } => {}
            Raising::LeadershipChanged { .. } => {}
            Raising::HerdsmanChanged { .. } => {}
        }
    }

    pub fn handle_physics_event(&mut self, frame: &mut Frame, event: Physics) {
        let assets = &mut frame.assets;
        match event {
            Physics::BodyPositionChanged {
                id,
                position,
                space,
            } => {
                for farmer in self.farmers.values_mut() {
                    if farmer.entity.body != id {
                        continue;
                    }
                    farmer.synchronize_position(position);
                }
                for creature in self.creatures.values_mut() {
                    if creature.entity.body == id {
                        creature.synchronize_position(position);
                        break;
                    }
                }
            }
            Physics::BarrierCreated {
                id,
                space,
                position,
                bounds,
            } => {
                self.barriers.push(BarrierHint {
                    id,
                    position,
                    bounds,
                });
            }
            Physics::BarrierDestroyed { id, .. } => {
                if let Some(index) = self.barriers.iter().position(|barrier| barrier.id == id) {
                    self.barriers.remove(index);
                }
            }
            Physics::SpaceUpdated { id, holes } => {
                for farmland in self.farmlands.values_mut() {
                    if farmland.entity.space == id {
                        farmland.holes = holes;
                        break;
                    }
                }
            }
        }
    }

    pub fn handle_universe_event(&mut self, frame: &mut Frame, event: Universe) {
        let assets = &mut frame.assets;
        let renderer = &mut frame.sprites;
        match event {
            Universe::TreeAppeared {
                tree,
                position,
                growth,
            } => {
                let kind = self.known.trees.get(tree.kind).unwrap().clone();
                info!(
                    "Appear {:?} kind='{}' at {:?} (g {})",
                    tree, kind.name, position, growth
                );

                let prefab = assets.tree(&kind.name);

                self.trees.insert(
                    tree,
                    TreeRep {
                        tree,
                        kind,
                        asset: prefab,
                        position,
                        direction: [0.0, 0.0],
                    },
                );
            }
            Universe::TreeVanished(id) => {
                info!("Vanish tree {:?}", id);
                self.trees.remove(&id);
                // self.barriers.remove(&id.into());
            }
            Universe::FarmlandAppeared {
                farmland,
                map,
                cells,
                rooms,
                holes,
            } => {
                let kind = self.known.farmlands.get(farmland.kind).unwrap().clone();
                info!("Appear {:?} kind='{}'", farmland, kind.name);

                let asset = assets.farmland(&kind.name);

                self.current_farmland = Some(farmland);

                let construction = assets.building("construction");
                let reconstruction = assets.building("reconstruction");
                let deconstruction = assets.building("deconstruction");

                let mut buildings = HashMap::new();
                let buildings_mapping = [
                    (Material::UNKNOWN, "template"),
                    (Material::CONCRETE, "concrete"),
                    (Material::WOOD, "wood"),
                    (Material::PLANKS, "planks"),
                    (Material::GLASS, "glass"),
                ];
                for (index, asset) in buildings_mapping {
                    let asset = assets.building(asset);
                    let rep = BuildingRep {
                        floor: frame
                            .sprites
                            .instantiate_tilemap(asset.floor.share(), asset.floor_sampler.share()),
                        roof: frame
                            .sprites
                            .instantiate_tilemap(asset.roof.share(), asset.roof_sampler.share()),
                        asset,
                    };
                    buildings.insert(index, rep);
                }

                let construction = BuildingRep {
                    floor: frame.sprites.instantiate_tilemap(
                        construction.floor.share(),
                        construction.floor_sampler.share(),
                    ),
                    roof: frame.sprites.instantiate_tilemap(
                        construction.roof.share(),
                        construction.roof_sampler.share(),
                    ),
                    asset: construction,
                };
                let reconstruction = BuildingRep {
                    floor: frame.sprites.instantiate_tilemap(
                        reconstruction.floor.share(),
                        reconstruction.floor_sampler.share(),
                    ),
                    roof: frame.sprites.instantiate_tilemap(
                        reconstruction.roof.share(),
                        reconstruction.roof_sampler.share(),
                    ),
                    asset: reconstruction,
                };
                let deconstruction = BuildingRep {
                    floor: frame.sprites.instantiate_tilemap(
                        deconstruction.floor.share(),
                        deconstruction.floor_sampler.share(),
                    ),
                    roof: frame.sprites.instantiate_tilemap(
                        deconstruction.roof.share(),
                        deconstruction.roof_sampler.share(),
                    ),
                    asset: deconstruction,
                };

                self.farmlands.insert(
                    farmland,
                    FarmlandRep {
                        entity: farmland,
                        kind,
                        asset,
                        soil_map: map,
                        cells,
                        rooms,
                        holes,
                        construction,
                        reconstruction,
                        deconstruction,
                        buildings,
                    },
                );
            }
            Universe::FarmlandVanished(id) => {
                info!("Vanish farmland {:?}", id);
                self.farmlands.remove(&id);
            }
            Universe::FarmerAppeared {
                farmer,
                position,
                player,
            } => {
                let kind = self.known.farmers.get(farmer.kind).unwrap();
                info!("Appear {:?} at {:?}", farmer, position);
                // let asset = assets.spine(&kind.name);
                let max_y = 7 * 2;
                let max_x = 14 * 2;
                let colors = [
                    [1.00, 1.00, 1.00, 1.0],
                    [0.64, 0.49, 0.40, 1.0],
                    [0.45, 0.40, 0.36, 1.0],
                    [0.80, 0.52, 0.29, 1.0],
                ];
                let pool = 256;
                let mut variant = 0;
                let asset = assets.farmer(&kind.name);
                let body = self.known.bodies.get(kind.body).unwrap();
                let is_controlled = player == self.client.player;

                self.farmers.insert(
                    farmer,
                    FarmerRep {
                        entity: farmer,
                        kind,
                        body,
                        player,
                        is_controlled,
                        asset,
                        estimated_position: position,
                        rendering_position: position,
                        last_sync_position: position,
                        activity: Activity::Idle,
                    },
                );
            }
            Universe::FarmerVanished(id) => {
                info!("Vanish farmer {:?}", id);
                self.farmers.remove(&id);
            }
            Universe::StackAppeared {
                stack: entity,
                position,
            } => {
                info!("Appear {:?} at {:?}", entity, position,);
                self.stacks.insert(entity, StackRep { entity, position });
            }
            Universe::StackVanished(entity) => {
                self.stacks.remove(&entity);
            }
            Universe::ConstructionAppeared { id: entity, cell } => {
                info!("Appear {:?} at {:?}", entity, cell);
                self.constructions
                    .insert(entity, ConstructionRep { entity, tile: cell });
            }
            Universe::ConstructionVanished { id } => {
                self.constructions.remove(&id);
            }
            Universe::ItemsAppeared { items } => {
                for item in items {
                    let container = self.items.entry(item.container).or_insert(HashMap::new());
                    container.insert(item.id, item);
                }
            }
            Universe::EquipmentAppeared { entity, position } => {
                info!("Appear {:?} at {:?}", entity, position);
                self.equipments
                    .insert(entity, EquipmentRep { entity, position });
            }
            Universe::EquipmentVanished(equipment) => {
                self.equipments.remove(&equipment);
            }
            Universe::ActivityChanged { farmer, activity } => {
                self.farmers.get_mut(&farmer).unwrap().activity = activity;
            }
            Universe::CreatureAppeared {
                entity,
                space,
                health,
                hunger,
                position,
            } => {
                let kind = self.known.creatures.get(entity.key).unwrap();
                let features = entity.animal.variant([4, 3, 3, 3, 3]);
                let [head, tail, c0, c1, c2] = features;
                info!(
                    "Appear {} {:?} at {:?} f{:?}",
                    &kind.name, entity, position, features
                );
                let body = self.known.bodies.get(kind.body).unwrap();
                let animal = self.known.animals.get(kind.animal).unwrap();
                let asset = assets.creature(&kind.name);
                let palette = [
                    [1.0, 1.0, 1.0, 1.0],
                    [0.43, 0.36, 0.31, 1.0],
                    [0.78, 0.47, 0.25, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                ];
                let colors = [palette[c0], palette[c1], palette[c2], palette[3]];
                let mut spine = renderer.instantiate_animal(&asset.spine, colors);
                spine
                    .skeleton
                    .animation_state
                    .set_animation_by_name(CreatureRep::ANIMATION_TRACK_IDLE, "idle", true)
                    .unwrap();

                spine
                    .skeleton
                    .animation_state
                    .set_animation_by_name(CreatureRep::ANIMATION_TRACK_WALK, "move", true)
                    .unwrap();

                // animal variants
                let mut skin = Skin::new("lama-dynamic-848");
                let head = asset
                    .spine
                    .skeleton
                    .find_skin(&format!("head/head-{}", head))
                    .unwrap();
                let tail = asset
                    .spine
                    .skeleton
                    .find_skin(&format!("tail/tail-{}", tail))
                    .unwrap();
                skin.add_skin(&head);
                skin.add_skin(&tail);
                spine.skeleton.skeleton.set_skin(&skin);

                self.creatures.insert(
                    entity,
                    CreatureRep {
                        entity,
                        asset,
                        kind,
                        body,
                        animal,
                        health,
                        estimated_position: position,
                        rendering_position: position,
                        last_sync_position: position,
                        spine,
                        direction: [1.0, 0.0],
                        velocity: [0.0, 0.0],
                    },
                );
            }
            Universe::CreatureEats { entity } => {
                let creature = self.creatures.get_mut(&entity).unwrap();
                creature.play_eat();
            }
            Universe::CreatureVanished(creature) => {
                self.creatures.remove(&creature);
            }
            Universe::CropAppeared {
                entity,
                position,
                impact,
                thirst,
                hunger,
                growth,
                health,
                fruits,
            } => {
                let kind = self.known.crops.get(entity.key).unwrap();
                info!(
                    "Appear {} {:?} at {:?} g{} f{}",
                    &kind.name, entity, position, growth, fruits
                );
                let asset = assets.crop(&kind.name);
                let colors = [
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                ];

                let mut spines = vec![
                    renderer.instantiate_plant(&asset.sprout, colors),
                    renderer.instantiate_plant(&asset.vegetating, colors),
                    renderer.instantiate_plant(&asset.flowering, colors),
                    renderer.instantiate_plant(&asset.ripening, colors),
                    renderer.instantiate_plant(&asset.withering, colors),
                ];
                for spine in spines.iter_mut() {
                    spine
                        .skeleton
                        .animation_state
                        .set_animation_by_name(CropRep::ANIMATION_TRACK_GROWTH, "growth", true)
                        .unwrap();
                }
                let spine = 0;

                // let mut spine = renderer.instantiate_spine(&asset.spine, colors);
                //let spine_data = spine.skeleton.skeleton.data();

                // SPECIFICATION:
                // let growth = spine_data.animation_at_index(3).unwrap();
                // spine
                //     .skeleton
                //     .animation_state
                //     .add_animation(3, growth.as_ref(), false, 0.0);
                //
                // let development = spine_data.animation_at_index(1).unwrap();
                // spine
                //     .skeleton
                //     .animation_state
                //     .add_animation(1, development.as_ref(), false, 0.0);
                //
                // let drying = spine_data.animation_at_index(2).unwrap();
                // spine
                //     .skeleton
                //     .animation_state
                //     .add_animation(2, drying.as_ref(), false, 0.0);

                // set skin
                // let [head, tail] = features;
                // let mut skin = Skin::new("lama-dynamic-848");
                // let head = spine.skeleton.find_skin(&head).unwrap();
                // let tail = spine.skeleton.find_skin(&tail).unwrap();
                // skin.add_skin(&head);
                // skin.add_skin(&tail);
                // skeleton.skeleton.set_skin(&skin);

                // spine
                //     .skeleton
                //     .skeleton
                //     .set_skin_by_name("test-skin")
                //     .unwrap();

                let mut representation = CropRep {
                    entity,
                    asset,
                    spines,
                    spine,
                    position,
                    impact,
                    thirst,
                    hunger,
                    growth,
                    health,
                    fruits: 0,
                };
                representation.synchronize_fruits(fruits);
                self.crops.insert(entity, representation);
            }
            Universe::CropVanished(crop) => {
                self.crops.remove(&crop);
            }
        }
    }
}
