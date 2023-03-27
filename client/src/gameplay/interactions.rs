use crate::gameplay::representation::FarmerRep;
use crate::gameplay::Intention::{Move, Put, Swap, Use};
use crate::gameplay::Target::{Construction, Crop, Equipment, Ground, Stack, Wall};
use crate::gameplay::{Gameplay, Intention, Target};
use game::api::FarmerBound;
use game::building::{Marker, Structure};
use game::inventory::Function;
use game::inventory::Function::{Installation, Instrumenting, Material, Product, Seeding};
use game::model::{Activity, CropKey, Purpose};
use log::error;

impl Gameplay {
    pub fn interact_with(
        &mut self,
        farmer: &mut FarmerRep,
        item: Vec<Function>,
        target: Target,
        intention: Intention,
    ) {
        match farmer.activity {
            Activity::Idle => match intention {
                Use => match target {
                    Stack(stack) => {
                        self.send_action(FarmerBound::TakeItem { stack });
                    }
                    Construction(construction) => {
                        self.send_action(FarmerBound::TakeMaterial { construction });
                    }
                    Equipment(equipment) => {
                        self.send_action(FarmerBound::UseEquipment { equipment });
                    }
                    Crop(crop) => {
                        self.send_action(FarmerBound::HarvestCrop { crop });
                    }
                    _ => {}
                },
                Put => match target {
                    Equipment(equipment) => {
                        self.send_action(FarmerBound::Uninstall { equipment });
                    }
                    _ => {}
                },
                Swap => {
                    self.send_action(FarmerBound::ToggleBackpack);
                }
                Move => {}
            },
            Activity::Usage => match intention {
                Use => {
                    for function in item {
                        match (function, target.clone()) {
                            (Seeding { .. }, Ground { tile }) => {
                                self.send_action(FarmerBound::PlantCrop { tile });
                                break;
                            }
                            (Installation { .. }, Ground { tile }) => {
                                self.send_action(FarmerBound::Install { tile });
                                break;
                            }
                            (Instrumenting, Construction(construction)) => {
                                self.send_action(FarmerBound::Build { construction });
                                break;
                            }
                            (Instrumenting, Crop(crop)) => {
                                self.send_action(FarmerBound::WaterCrop { crop });
                                break;
                            }
                            (Material { .. }, Stack(stack)) => {
                                // TODO: generic, not material only
                                self.send_action(FarmerBound::TakeItem { stack });
                                break;
                            }
                            (Material { .. }, Construction(construction)) => {
                                // TODO: generic, not material only, generic container?
                                self.send_action(FarmerBound::TakeMaterial { construction });
                                break;
                            }
                            (Product { kind }, Crop(crop)) => {
                                if CropKey(kind) == crop.key {
                                    self.send_action(FarmerBound::HarvestCrop { crop });
                                }
                            }
                            _ => {}
                        }
                    }
                }
                Put => match target {
                    Ground { tile } => {
                        self.send_action(FarmerBound::DropItem { tile });
                    }
                    Stack(stack) => {
                        self.send_action(FarmerBound::PutItem { stack });
                    }
                    Construction(construction) => {
                        self.send_action(FarmerBound::PutMaterial { construction });
                    }
                    _ => {}
                },
                Swap => {
                    self.send_action(FarmerBound::ToggleBackpack);
                }
                Move => {}
            },
            Activity::Surveying {
                equipment,
                selection,
            } => match intention {
                Use => match target {
                    Ground { tile } => {
                        let structure = match selection {
                            1 => Structure::Door,
                            2 => Structure::Window,
                            3 => Structure::Fence,
                            _ => Structure::Wall,
                        };
                        if let Purpose::Surveying { surveyor } = equipment.purpose {
                            self.send_action(FarmerBound::Survey {
                                surveyor,
                                tile,
                                marker: Marker::Construction(structure),
                            });
                        } else {
                            error!("Not sur")
                        }
                    }
                    Wall(tile) => {
                        let structure = match selection {
                            1 => Structure::Door,
                            2 => Structure::Window,
                            3 => Structure::Fence,
                            _ => Structure::Wall,
                        };
                        if let Purpose::Surveying { surveyor } = equipment.purpose {
                            self.send_action(FarmerBound::Survey {
                                surveyor,
                                tile,
                                marker: Marker::Reconstruction(structure),
                            });
                        } else {
                            error!("Not sur")
                        }
                    }
                    Construction(construction) => {
                        self.send_action(FarmerBound::RemoveConstruction { construction });
                        let structure = match selection {
                            1 => Structure::Door,
                            2 => Structure::Window,
                            3 => Structure::Fence,
                            _ => Structure::Wall,
                        };
                        if let Purpose::Surveying { surveyor } = equipment.purpose {
                            let marker = match construction.marker {
                                Marker::Construction(_) => Marker::Construction(structure),
                                Marker::Reconstruction(_) => Marker::Reconstruction(structure),
                                Marker::Deconstruction => Marker::Construction(structure),
                            };
                            self.send_action(FarmerBound::Survey {
                                surveyor,
                                tile: construction.cell,
                                marker,
                            });
                        } else {
                            error!("Not sur")
                        }
                    }
                    _ => {
                        // beep error
                    }
                },
                Put => match target {
                    Construction(construction) => {
                        self.send_action(FarmerBound::RemoveConstruction { construction });
                    }
                    Wall(tile) => {
                        if let Purpose::Surveying { surveyor } = equipment.purpose {
                            self.send_action(FarmerBound::Survey {
                                surveyor,
                                tile,
                                marker: Marker::Deconstruction,
                            });
                        } else {
                            error!("Not sur")
                        }
                    }
                    _ => {}
                },
                Swap => {
                    self.send_action(FarmerBound::ToggleSurveyingOption);
                }
                Intention::Move => {
                    self.send_action(FarmerBound::CancelActivity);
                    farmer.activity = Activity::Idle;
                }
            },
        }
    }
}
