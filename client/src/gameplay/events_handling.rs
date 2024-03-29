use std::collections::HashMap;

use log::{debug, info};
use rusty_spine::Skin;

use game::api::Event;
use game::assembling::Assembling;
use game::building::{Building, Material};
use game::inventory::Inventory;
use game::landscaping::Landscaping;
use game::math::Array;
use game::model::{Activity, AssemblyTarget, Universe};
use game::physics::{Barrier, Physics};
use game::planting::Planting;
use game::raising::Raising;
use game::timing::Timing;
use game::working::Working;

use crate::engine::Frame;
use crate::gameplay::representation::{
    AssemblyRep, AssemblyTargetAsset, BuildingRep, CementerRep, ComposterRep, ConstructionRep,
    CorpseRep, CreatureRep, CropRep, DoorRep, EquipmentRep, FarmerRep, FarmlandRep, ItemRep,
    RestRep, StackRep, TheodoliteRep, TreeRep,
};
use crate::gameplay::Gameplay;

impl Gameplay {
    pub fn handle_event(&mut self, frame: &mut Frame, event: Event) {
        match event {
            Event::TimingStream(events) => {
                for event in events {
                    self.handle_timing_event(frame, event);
                }
            }
            Event::UniverseStream(events) => {
                for event in events {
                    self.handle_universe_event(frame, event);
                }
            }
            Event::PhysicsStream(events) => {
                for event in events {
                    self.handle_physics_event(frame, event);
                }
            }
            Event::BuildingStream(events) => {
                for event in events {
                    self.handle_building_event(frame, event);
                }
            }
            Event::InventoryStream(events) => {
                for event in events {
                    self.handle_inventory_event(frame, event);
                }
            }
            Event::PlantingStream(events) => {
                for event in events {
                    self.handle_planting_event(frame, event);
                }
            }
            Event::RaisingStream(events) => {
                for event in events {
                    self.handle_raising_event(frame, event);
                }
            }
            Event::AssemblingStream(events) => {
                for event in events {
                    self.handle_assembling_event(frame, event);
                }
            }
            Event::WorkingStream(events) => {
                for event in events {
                    self.handle_working_event(frame, event);
                }
            }
            Event::LandscapingStream(events) => {
                for event in events {
                    self.handle_landscaping_event(frame, event);
                }
            }
        }
    }

    pub fn handle_building_event(&mut self, _frame: &mut Frame, event: Building) {
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
            Building::SurveyorModeChanged { id, mode } => {
                for theodolite in self.theodolites.values_mut() {
                    if theodolite.entity.surveyor == id {
                        theodolite.mode = mode;
                        break;
                    }
                }
            }
        }
    }

    pub fn handle_inventory_event(&mut self, frame: &mut Frame, event: Inventory) {
        match event {
            Inventory::ContainerCreated { .. } => {}
            Inventory::ContainerDestroyed { id } => {
                self.items.remove(&id);
            }
            Inventory::ItemsAdded { items } => {
                for item in items {
                    let container = self.items.entry(item.container).or_insert(HashMap::new());
                    let kind = self.known.items.get(item.key).unwrap();
                    let asset = frame.assets.item(&kind.name);
                    container.insert(
                        item.id,
                        ItemRep {
                            id: item.id,
                            kind,
                            asset,
                            container: item.container,
                            quantity: item.quantity,
                        },
                    );
                }
            }
            Inventory::ItemRemoved { item, container } => {
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

    pub fn handle_landscaping_event(&mut self, _frame: &mut Frame, event: Landscaping) {
        match event {
            // Landscaping::MoistureUpdate { land, moisture } => {
            //     for farmland in self.farmlands.values_mut() {
            //         if farmland.entity.land == land {
            //             farmland.moisture = Box::new(moisture);
            //             break;
            //         }
            //     }
            // }
            // Landscaping::MoistureCapacityUpdate {
            //     land,
            //     moisture_capacity,
            // } => {
            //     for farmland in self.farmlands.values_mut() {
            //         if farmland.entity.land == land {
            //             farmland.moisture_capacity = Box::new(moisture_capacity);
            //             break;
            //         }
            //     }
            // }
            // Landscaping::SurfaceUpdate { land, surface } => {
            //     info!("Surface update {land:?}");
            //     for farmland in self.farmlands.values_mut() {
            //         if farmland.entity.land == land {
            //             farmland.surface = Box::new(surface);
            //             break;
            //         }
            //     }
            // }
            Landscaping::MoistureInspected {
                land,
                rect,
                moisture,
            } => {
                for farmland in self.farmlands.values_mut() {
                    if farmland.entity.land == land {
                        farmland
                            .moisture
                            .paste(farmland.kind.land.width, rect, &moisture);
                        break;
                    }
                }
            }
            Landscaping::MoistureCapacityInspected {
                land,
                rect,
                moisture_capacity,
            } => {
                for farmland in self.farmlands.values_mut() {
                    if farmland.entity.land == land {
                        farmland.moisture_capacity.paste(
                            farmland.kind.land.width,
                            rect,
                            &moisture_capacity,
                        );
                        break;
                    }
                }
            }
            Landscaping::SurfaceInspected {
                surface,
                rect,
                land,
            } => {
                // info!("Surface update {land:?}");
                for farmland in self.farmlands.values_mut() {
                    if farmland.entity.land == land {
                        farmland
                            .surface
                            .paste(farmland.kind.land.width, rect, &surface);
                        break;
                    }
                }
            }
        }
    }

    pub fn handle_planting_event(&mut self, _frame: &mut Frame, event: Planting) {
        match event {
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
                        crop.synchronize_hunger(hunger);
                        crop.synchronize_growth(growth);
                        break;
                    }
                }
            }
            Planting::PlantFruitsChanged { id, fruits } => {
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
            Planting::SoilFertilityInspected {
                soil,
                fertility,
                rect,
            } => {
                for farmland in self.farmlands.values_mut() {
                    if farmland.entity.soil == soil {
                        farmland
                            .fertility
                            .paste(farmland.kind.soil.width, rect, &fertility);
                        break;
                    }
                }
            }
        }
    }

    pub fn handle_raising_event(&mut self, _frame: &mut Frame, event: Raising) {
        match event {
            Raising::AnimalChanged {
                id,
                age,
                thirst: _,
                hunger: _,
                weight,
            } => {
                for creature in self.creatures.values_mut() {
                    if creature.entity.animal == id {
                        creature.age = age;
                        creature.weight = weight;
                        break;
                    }
                }
            }
            Raising::AnimalHealthChanged { id, health } => {
                for creature in self.creatures.values_mut() {
                    if creature.entity.animal == id {
                        creature.health = health;
                        break;
                    }
                }
            }
            Raising::LeadershipChanged { .. } => {}
            Raising::HerdsmanChanged { .. } => {}
            Raising::BehaviourChanged { id, behaviour } => {
                for creature in self.creatures.values_mut() {
                    if creature.entity.animal == id {
                        creature.play(behaviour, behaviour);
                        break;
                    }
                }
            }
            Raising::BehaviourTriggered {
                id,
                behaviour,
                trigger,
            } => {
                for creature in self.creatures.values_mut() {
                    if creature.entity.animal == id {
                        creature.play(trigger, behaviour);
                        break;
                    }
                }
            }
            Raising::AnimalTied { id, tether } => {
                for creature in self.creatures.values_mut() {
                    if creature.entity.animal == id {
                        creature.tether = Some(tether);
                        break;
                    }
                }
            }
            Raising::AnimalUntied { id, tether: _ } => {
                for creature in self.creatures.values_mut() {
                    if creature.entity.animal == id {
                        creature.tether = None;
                        break;
                    }
                }
            }
        }
    }

    pub fn handle_working_event(&mut self, _frame: &mut Frame, event: Working) {
        match event {
            Working::DeviceUpdated {
                device,
                enabled,
                broken,
                progress,
                input,
                output,
                deprecation,
            } => {
                for cementer in self.cementers.values_mut() {
                    if cementer.entity.device == device {
                        cementer.progress = progress;
                        cementer.enabled = enabled;
                        cementer.broken = broken;
                        cementer.input = input;
                        cementer.output = output;
                        cementer.deprecation = deprecation;
                        return;
                    }
                }
            }
        }
    }

    pub fn handle_physics_event(&mut self, _frame: &mut Frame, event: Physics) {
        match event {
            Physics::BodyPositionChanged {
                id,
                position,
                destination,
                ..
            } => {
                for farmer in self.farmers.values_mut() {
                    if farmer.entity.body == id {
                        farmer.synchronize_position(position, destination);
                        return;
                    }
                }
                for creature in self.creatures.values_mut() {
                    if creature.entity.body == id {
                        creature.synchronize_position(position);
                        return;
                    }
                }
            }
            Physics::BarrierCreated {
                id,
                position,
                space,
                active,
                key,
            } => {
                let kind = self.known.barriers.get(key).unwrap();
                self.barriers_hint.push(Barrier {
                    id,
                    kind,
                    position,
                    space,
                    active,
                });
            }
            Physics::BarrierChanged { id, active, .. } => {
                for barrier in self.barriers_hint.iter_mut() {
                    if barrier.id == id {
                        barrier.active = active;
                        break;
                    }
                }
            }
            Physics::BarrierDestroyed { id, .. } => {
                if let Some(index) = self
                    .barriers_hint
                    .iter()
                    .position(|barrier| barrier.id == id)
                {
                    self.barriers_hint.remove(index);
                }
            }
            Physics::SpaceUpdated { id, holes } => {
                for farmland in self.farmlands.values_mut() {
                    if farmland.entity.space == id {
                        farmland.holes = holes;
                        return;
                    }
                }
            }
        }
    }

    pub fn handle_assembling_event(&mut self, _frame: &mut Frame, event: Assembling) {
        match event {
            Assembling::PlacementUpdated {
                placement,
                rotation,
                pivot,
                valid,
            } => {
                for assembly in self.assembly.values_mut() {
                    if assembly.entity.placement == placement {
                        assembly.rotation = rotation;
                        assembly.pivot = pivot;
                        assembly.valid = valid;
                        return;
                    }
                }
            }
        }
    }

    pub fn handle_timing_event(&mut self, _frame: &mut Frame, event: Timing) {
        match event {
            Timing::TimeUpdated {
                speed,
                colonization_date,
            } => {
                if speed != self.game_speed {
                    info!("Set local time speed to {speed}");
                }
                self.game_speed = speed;
                self.colonization_date = colonization_date;
            }
            Timing::CalendarUpdated {
                id,
                times_of_day,
                season_day,
                season,
            } => {
                for farmland in self.farmlands.values_mut() {
                    if farmland.entity.calendar == id {
                        farmland.times_of_day = times_of_day;
                        farmland.season_day = season_day;
                        farmland.season = season;
                        break;
                    }
                }
            }
        }
    }

    pub fn handle_universe_event(&mut self, frame: &mut Frame, event: Universe) {
        let assets = &mut frame.assets;
        let renderer = &mut frame.scene;
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
                cells,
                rooms,
                holes,
                season,
                season_day,
                times_of_day,
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
                            .scene
                            .instantiate_tilemap(asset.floor.share(), asset.floor_sampler.share()),
                        roof: frame
                            .scene
                            .instantiate_tilemap(asset.roof.share(), asset.roof_sampler.share()),
                        asset,
                    };
                    buildings.insert(index, rep);
                }

                let construction = BuildingRep {
                    floor: frame.scene.instantiate_tilemap(
                        construction.floor.share(),
                        construction.floor_sampler.share(),
                    ),
                    roof: frame.scene.instantiate_tilemap(
                        construction.roof.share(),
                        construction.roof_sampler.share(),
                    ),
                    asset: construction,
                };
                let reconstruction = BuildingRep {
                    floor: frame.scene.instantiate_tilemap(
                        reconstruction.floor.share(),
                        reconstruction.floor_sampler.share(),
                    ),
                    roof: frame.scene.instantiate_tilemap(
                        reconstruction.roof.share(),
                        reconstruction.roof_sampler.share(),
                    ),
                    asset: reconstruction,
                };
                let deconstruction = BuildingRep {
                    floor: frame.scene.instantiate_tilemap(
                        deconstruction.floor.share(),
                        deconstruction.floor_sampler.share(),
                    ),
                    roof: frame.scene.instantiate_tilemap(
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
                        moisture: vec![0.0; 128 * 128],
                        moisture_capacity: vec![0.0; 128 * 128],
                        surface: vec![0; 128 * 128],
                        surface_tilemap: frame.scene.instantiate_tilemap(
                            assets.texture("./assets/texture/tiles-waterbody.png"),
                            assets.sampler("pixel-perfect"),
                        ),
                        fertility: vec![0.0; 128 * 128],
                        cells,
                        rooms,
                        holes,
                        construction,
                        reconstruction,
                        deconstruction,
                        buildings,
                        season,
                        season_day,
                        times_of_day,
                    },
                );
            }
            Universe::FarmlandVanished(id) => {
                self.farmlands.remove(&id);
            }
            Universe::FarmerAppeared {
                farmer,
                position,
                player,
            } => {
                let kind = self.known.farmers.get(farmer.kind).unwrap();
                info!("Appear {:?} at {:?}", farmer, position);
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
                debug!("Appear {:?} at {:?}", entity, position,);
                self.stacks.insert(entity, StackRep { entity, position });
            }
            Universe::StackVanished(entity) => {
                self.stacks.remove(&entity);
            }
            Universe::ConstructionAppeared {
                id: entity,
                cell: tile,
                marker,
            } => {
                self.constructions.insert(
                    entity,
                    ConstructionRep {
                        entity,
                        tile,
                        marker,
                    },
                );
            }
            Universe::ConstructionVanished { id } => {
                self.constructions.remove(&id);
            }

            Universe::TheodoliteAppeared { id, position, mode } => {
                let kind = self.known.theodolites.get(id.key).unwrap();
                let item = assets.item(&kind.item.name);
                self.theodolites.insert(
                    id,
                    TheodoliteRep {
                        entity: id,
                        position,
                        mode,
                        item,
                    },
                );
            }
            Universe::TheodoliteVanished { id } => {
                self.theodolites.remove(&id);
            }
            Universe::EquipmentAppeared { entity, position } => {
                info!("Appear {:?} at {:?}", entity, position);
                let kind = self.known.equipments.get(entity.key).unwrap();
                let item = assets.item(&kind.item.name);
                self.equipments.insert(
                    entity,
                    EquipmentRep {
                        entity,
                        position,
                        kind,
                        item,
                    },
                );
            }
            Universe::EquipmentVanished(equipment) => {
                self.equipments.remove(&equipment);
            }
            Universe::ActivityChanged { farmer, activity } => {
                self.farmers.get_mut(&farmer).unwrap().activity = activity;
            }
            Universe::CorpseAppeared { position, entity } => {
                let kind = self.known.corpses.get(entity.key).unwrap();
                let asset = assets.corpse(&kind.name);
                let representation = CorpseRep {
                    entity,
                    asset,
                    position,
                };
                self.corpses.insert(entity, representation);
            }
            Universe::CorpseVanished(corpse) => {
                self.corpses.remove(&corpse);
            }
            Universe::CreatureAppeared {
                entity,
                farmland: _,
                health,
                hunger: _,
                weight,
                position,
                behaviour,
                age,
            } => {
                let kind = self.known.creatures.get(entity.key).unwrap();
                let asset = assets.creature(&kind.name);
                // TODO: move to asset creation
                let mut head_skins = vec![];
                let mut tail_skins = vec![];
                let mut ear_left_skins = vec![];
                let mut ear_right_skins = vec![];
                for skin in asset.spine.skeleton.skins() {
                    let name = skin.name().to_string();
                    if name.starts_with("ear-left/") {
                        ear_left_skins.push(name);
                    } else if name.starts_with("ear-right/") {
                        ear_right_skins.push(name);
                    } else if name.starts_with("female/head/") {
                        head_skins.push(name);
                    } else if name.starts_with("tail/") {
                        tail_skins.push(name);
                    }
                }

                let features = entity.animal.variant([
                    head_skins.len(),
                    ear_left_skins.len(),
                    ear_right_skins.len(),
                    tail_skins.len(),
                    3,
                    3,
                    3,
                ]);
                let [head, ear_l, ear_r, tail, c0, c1, c2] = features;
                info!(
                    "Appear {} {:?} at {:?} f{:?}",
                    &kind.name, entity, position, features
                );

                let palette = [
                    [1.0, 0.94, 0.9, 1.0],
                    [0.43, 0.36, 0.31, 1.0],
                    [0.78, 0.47, 0.25, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                ];
                let colors = [palette[c0], palette[c1], palette[c2], palette[3]];
                let mut spine = renderer.instantiate_animal(&asset.spine, colors);
                spine
                    .skeleton
                    .animation_state
                    .set_animation_by_name(CreatureRep::ANIMATION_BASE, "idle", true)
                    .unwrap();

                spine
                    .skeleton
                    .animation_state
                    .set_animation_by_name(CreatureRep::ANIMATION_WALK, "move", true)
                    .unwrap();

                spine
                    .skeleton
                    .animation_state
                    .set_animation_by_name(CreatureRep::ANIMATION_AGE, "age", false)
                    .unwrap();

                spine
                    .skeleton
                    .animation_state
                    .set_animation_by_name(CreatureRep::ANIMATION_WEIGHT, "weight", false)
                    .unwrap();

                // animal variants
                let mut skin = Skin::new("lama-dynamic-848");
                let head = asset.spine.skeleton.find_skin(&head_skins[head]).unwrap();
                let ear_l = asset
                    .spine
                    .skeleton
                    .find_skin(&ear_left_skins[ear_l])
                    .unwrap();
                let ear_r = asset
                    .spine
                    .skeleton
                    .find_skin(&ear_right_skins[ear_r])
                    .unwrap();
                let tail = asset.spine.skeleton.find_skin(&tail_skins[tail]).unwrap();
                skin.add_skin(&head);
                skin.add_skin(&ear_l);
                skin.add_skin(&ear_r);
                skin.add_skin(&tail);
                spine.skeleton.skeleton.set_skin(&skin);
                let mut representation = CreatureRep {
                    entity,
                    asset,
                    kind,
                    health,
                    age,
                    weight,
                    estimated_position: position,
                    rendering_position: position,
                    last_sync_position: position,
                    spine,
                    direction: [1.0, 0.0],
                    velocity: [0.0, 0.0],
                    behaviour,
                    tether: None,
                };
                representation.play(behaviour, behaviour);
                self.creatures.insert(entity, representation);
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
                    position,
                    impact,
                    thirst,
                    hunger,
                    growth,
                    health,
                    fruits,
                };
                representation.synchronize_fruits(fruits);
                self.crops.insert(entity, representation);
            }
            Universe::CropVanished(crop) => {
                self.crops.remove(&crop);
            }
            Universe::AssemblyAppeared {
                entity,
                pivot,
                rotation,
                valid,
            } => {
                let kind = self.known.assembly.get(entity.key).unwrap();
                let asset = match &kind.target {
                    AssemblyTarget::Door { door } => {
                        let door = assets.door(&door.name);
                        AssemblyTargetAsset::Door { door }
                    }
                    AssemblyTarget::Cementer { cementer } => {
                        let asset = assets.cementer(&cementer.name);
                        AssemblyTargetAsset::Cementer {
                            cementer: asset,
                            kind: cementer.clone(),
                        }
                    }
                    AssemblyTarget::Composter { composter } => {
                        let asset = assets.composter(&composter.name);
                        AssemblyTargetAsset::Composter {
                            composter: asset,
                            kind: composter.clone(),
                        }
                    }
                    AssemblyTarget::Rest { rest } => {
                        let rest = assets.rest(&rest.name);
                        AssemblyTargetAsset::Rest { rest }
                    }
                };
                let representation = AssemblyRep {
                    entity,
                    asset,
                    rotation,
                    pivot,
                    valid,
                };
                self.assembly.insert(entity, representation);
            }
            Universe::AssemblyUpdated { .. } => {}
            Universe::AssemblyVanished(entity) => {
                self.assembly.remove(&entity);
            }
            Universe::DoorAppeared {
                entity,
                position,
                open,
                rotation,
            } => {
                info!("Appear {entity:?} {open}");
                let kind = self.known.doors.get(entity.key).unwrap();
                let asset = assets.door(&kind.name);
                let representation = DoorRep {
                    entity,
                    asset,
                    open,
                    rotation,
                    position,
                };
                self.doors.insert(entity, representation);
            }
            Universe::DoorChanged { entity, open } => {
                info!("DoorChanged {entity:?} {open}");
                self.doors.get_mut(&entity).unwrap().open = open;
            }
            Universe::DoorVanished(door) => {
                self.doors.remove(&door);
            }
            Universe::CementerAppeared {
                entity,
                rotation,
                position,
                enabled,
                broken,
                input,
                output,
                progress,
            } => {
                info!("Appear {entity:?} e{enabled:?} b{broken} i{input} o{output} p{progress}");
                let kind = self.known.cementers.get(entity.key).unwrap();
                let asset = assets.cementer(&kind.name);
                let representation = CementerRep {
                    entity,
                    kind,
                    asset,
                    rotation,
                    position,
                    enabled,
                    broken,
                    input,
                    output,
                    deprecation: 0.0,
                    progress,
                };
                self.cementers.insert(entity, representation);
            }
            Universe::CementerVanished(entity) => {
                self.cementers.remove(&entity);
            }
            Universe::RestAppeared {
                entity,
                position,
                rotation,
            } => {
                let kind = self.known.rests.get(entity.key).unwrap();
                let asset = assets.rest(&kind.name);
                let representation = RestRep {
                    entity,
                    asset,
                    rotation,
                    position,
                };
                self.rests.insert(entity, representation);
            }
            Universe::RestVanished(entity) => {
                self.rests.remove(&entity);
            }
            Universe::ComposterInspected {
                entity,
                rotation,
                position,
                enabled,
                broken,
                input,
                output,
                progress,
            } => {
                info!("Appear {entity:?} e{enabled:?} b{broken} i{input} o{output} p{progress}");
                let kind = self.known.composters.get(entity.key).unwrap();
                let asset = assets.composter(&kind.name);
                let representation = ComposterRep {
                    entity,
                    kind,
                    asset,
                    rotation,
                    position,
                    enabled,
                    broken,
                    input,
                    output,
                    deprecation: 0.0,
                    progress,
                };
                self.composters.insert(entity, representation);
            }
            Universe::ComposterVanished(entity) => {
                self.composters.remove(&entity);
            }
        }
    }
}
