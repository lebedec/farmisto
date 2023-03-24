use crate::gameplay::representation::FarmerRep;
use crate::gameplay::Intention::{Put, Swap, Use};
use crate::gameplay::Target::{Construction, Crop, Equipment, Ground, Stack, Wall};
use crate::gameplay::{Gameplay, Intention, Target};
use game::api::FarmerBound;
use game::building::Marker;
use game::inventory::Function;
use game::inventory::Function::{Installation, Instrumenting, Material, Product, Seeding};
use game::model::{Activity, CropKey, ItemRep, Purpose};
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
                                self.send_action(FarmerBound::Construct { construction });
                                break;
                            }
                            (Instrumenting, Wall(tile)) => {
                                self.send_action(FarmerBound::Deconstruct { tile });
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
            },
            Activity::Surveying {
                equipment,
                selection,
            } => match intention {
                Use => match target {
                    Ground { tile } => {
                        let marker = match selection {
                            0 => Marker::Wall,
                            1 => Marker::Door,
                            2 => Marker::Window,
                            _ => Marker::Wall,
                        };
                        if let Purpose::Surveying { surveyor } = equipment.purpose {
                            self.send_action(FarmerBound::Survey {
                                surveyor,
                                tile,
                                marker,
                            });
                        } else {
                            error!("Not sur")
                        }
                    }
                    Construction(construction) => {
                        self.send_action(FarmerBound::RemoveConstruction { construction });
                    }
                    _ => {
                        // beep error
                    }
                },
                Put => {
                    self.send_action(FarmerBound::CancelActivity);
                }
                Swap => {
                    self.send_action(FarmerBound::ToggleSurveyingOption);
                }
            },
        }
    }
}
