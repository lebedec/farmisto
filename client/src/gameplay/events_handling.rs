use std::collections::HashMap;

use log::{debug, info};
use rusty_spine::Skin;

use game::api::Event;
use game::assembling::Assembling;
use game::building::{Building, Material};
use game::inventory::Inventory;
use game::landscaping::Landscaping;
use game::math::Array2D;
use game::model::{Activity, AssemblyTarget, Universe};
use game::physics::{Barrier, Physics};
use game::planting::Planting;
use game::raising::Raising;
use game::timing::Timing;
use game::working::Working;

use crate::engine::Frame;
use crate::gameplay::representation::{
    AssemblyRep, AssemblyTargetAsset, BuildingRep, CementerRep, ConstructionRep, CreatureRep,
    CropRep, DoorRep, EquipmentRep, FarmerRep, FarmlandRep, ItemRep, RestRep, StackRep, TreeRep,
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
        }
    }

    pub fn handle_inventory_event(&mut self, frame: &mut Frame, event: Inventory) {
        match event {
            Inventory::ContainerCreated { .. } => {}
            Inventory::ContainerDestroyed { id } => {
                self.items.remove(&id);
            }
            Inventory::ItemAdded {
                id,
                kind,
                container,
                quantity,
            } => {
                info!("item added {:?} to {:?}", id, container);
                let items = self.items.entry(container).or_insert(HashMap::new());
                let kind = self.known.items.get(kind).unwrap();
                let asset = frame.assets.item(&kind.name);
                items.insert(
                    id,
                    ItemRep {
                        id,
                        kind,
                        asset,
                        container,
                        quantity,
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
                            .patch_rect(farmland.kind.land.width, rect, moisture);
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
                        farmland.moisture_capacity.patch_rect(
                            farmland.kind.land.width,
                            rect,
                            moisture_capacity,
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
                            .patch_rect(farmland.kind.land.width, rect, surface);
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

    pub fn handle_raising_event(&mut self, _frame: &mut Frame, event: Raising) {
        match event {
            Raising::AnimalChanged { .. } => {}
            Raising::LeadershipChanged { .. } => {}
            Raising::HerdsmanChanged { .. } => {}
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

    pub fn handle_timing_event(&mut self, frame: &mut Frame, event: Timing) {
        match event {
            Timing::TimeUpdated {
                speed,
                colonization_date,
            } => {
                if speed != self.speed {
                    info!("Set local time speed to {speed}");
                }
                self.speed = speed;
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

                info!("Before insert !!! {}", std::mem::size_of::<FarmlandRep>());
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
                info!("Farmland created !!!")
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
                    let kind = self.known.items.get(item.kind).unwrap();
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
                health,
                position,
                ..
            } => {
                let kind = self.known.creatures.get(entity.key).unwrap();
                let features = entity.animal.variant([4, 3, 3, 3, 3]);
                let [head, tail, c0, c1, c2] = features;
                info!(
                    "Appear {} {:?} at {:?} f{:?}",
                    &kind.name, entity, position, features
                );
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
        }
    }
}
