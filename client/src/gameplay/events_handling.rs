use crate::engine::Frame;
use crate::gameplay::representation::{
    BarrierHint, ConstructionRep, CropRep, DropRep, EquipmentRep, FarmerRep, FarmlandRep, TreeRep,
};
use crate::gameplay::Gameplay;
use game::api::Event;
use game::building::Building;
use game::inventory::{Function, Inventory};
use game::model::{Activity, ItemRep, Universe};
use game::physics::Physics;
use game::planting::Planting;
use log::{error, info};
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
            Planting::LandChanged { land, map } => {
                for (farmland, behaviour) in self.farmlands.iter_mut() {
                    if farmland.land == land {
                        behaviour.map = map;
                        break;
                    }
                }
            }
            Planting::PlantUpdated { id, impact, thirst } => {
                for crop in self.crops.values_mut() {
                    if crop.entity.plant != id {
                        continue;
                    }
                    crop.synchronize_impact(impact);
                    crop.synchronize_thirst(thirst);
                }
            }
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
            Physics::BarrierDestroyed { id } => {
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
                    "Appear tree {:?} kind='{}' at {:?} (g {})",
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
                info!("Appear farmland {:?} kind='{}'", farmland, kind.name);

                let asset = assets.farmland(&kind.name);

                self.current_farmland = Some(farmland);

                let floor = frame.sprites.instantiate_tilemap(
                    self.tilemap_texture.share(),
                    self.tilemap_sampler.share(),
                );

                let roof = frame.sprites.instantiate_tilemap(
                    self.tilemap_roof_texture.share(),
                    self.tilemap_sampler.share(),
                );

                self.farmlands.insert(
                    farmland,
                    FarmlandRep {
                        entity: farmland,
                        kind,
                        asset,
                        map,
                        cells,
                        rooms,
                        holes,
                        floor,
                        roof,
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
                info!("Appear farmer {:?} at {:?}", farmer, position);
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
                // for y in 0..max_y {
                //     for x in 0..max_x {
                //         let c0 = variant / 64;
                //         let c1 = (variant % 64) / 16;
                //         let c2 = (variant % 16) / 4;
                //         let c3 = variant % 4;
                //         variant = ((5 * variant) + 1) % pool;
                //         let asset = asset.share();
                //         let variant = x + y * max_x;
                //         let head = format!("head/head-{}", variant % 4);
                //         let tile = format!("tail/tail-{}", variant % 3);
                //         let sprite = frame.sprites.instantiate(
                //             &asset,
                //             [head, tile],
                //             [colors[c0], colors[c1], colors[c2], colors[c3]],
                //         );
                //         let position = [
                //             64.0 + 128.0 + 128.0 * x as f32,
                //             64.0 + 256.0 + 128.0 * y as f32,
                //         ];
                //         self.farmers2d.push(Farmer2d {
                //             asset,
                //             sprite,
                //             position,
                //             variant,
                //         });
                //     }
                // }

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
            Universe::BarrierHintAppeared {
                id,
                kind,
                position,
                bounds,
            } => {
                self.barriers.push(BarrierHint {
                    id,
                    position,
                    bounds,
                });
            }
            Universe::DropAppeared { drop, position } => {
                info!("Appear drop {:?} at {:?}", drop, position,);
                self.drops.insert(
                    drop,
                    DropRep {
                        entity: drop,
                        position,
                    },
                );
            }
            Universe::DropVanished(drop) => {
                self.drops.remove(&drop);
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
            Universe::CropAppeared {
                entity,
                position,
                impact,
                thirst,
            } => {
                let kind = self.known.crops.get(entity.key).unwrap();
                info!("Appear {} {:?} at {:?}", &kind.name, entity, position);
                let asset = assets.crop(&kind.name);
                let colors = [
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                    [1.0, 1.0, 1.0, 1.0],
                ];

                info!("instantiates spine data");

                let mut spines = vec![
                    renderer.instantiate_spine(&asset.sprout, colors),
                    renderer.instantiate_spine(&asset.vegetating, colors),
                    renderer.instantiate_spine(&asset.flowering, colors),
                    renderer.instantiate_spine(&asset.ripening, colors),
                    renderer.instantiate_spine(&asset.withering, colors),
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

                info!("spine data");

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

                info!("sets skin");

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
                    growth: 0.0,
                    health: 0.0,
                    fruits: 0,
                };
                representation.synchronize_fruits(0);
                self.crops.insert(entity, representation);
            }
            Universe::CropVanished(crop) => {
                self.crops.remove(&crop);
            }
        }
    }
}
