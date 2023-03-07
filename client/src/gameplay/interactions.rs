use crate::gameplay::representation::FarmerRep;
use crate::gameplay::Intention::{Put, Swap, Use};
use crate::gameplay::Target::{Construction, Crop, Drop, Equipment, Ground, Wall};
use crate::gameplay::{Gameplay, Intention, Target};
use game::api::Action;
use game::building::Marker;
use game::inventory::Function;
use game::inventory::Function::{Installation, Instrumenting, Material, Seeding};
use game::model::{Activity, ItemRep, Purpose};
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
                    Drop(drop) => {
                        self.send_action(Action::TakeItem { drop });
                    }
                    Construction(construction) => {
                        self.send_action(Action::TakeMaterial { construction });
                    }
                    Equipment(equipment) => {
                        self.send_action(Action::UseEquipment { equipment });
                    }
                    _ => {}
                },
                Put => match target {
                    Equipment(equipment) => {
                        self.send_action(Action::Uninstall { equipment });
                    }
                    _ => {}
                },
                Swap => {
                    self.send_action(Action::ToggleBackpack);
                }
            },
            Activity::Usage => match intention {
                Use => {
                    for function in item {
                        match (function, target.clone()) {
                            (Seeding { .. }, Ground { tile }) => {
                                self.send_action(Action::PlantCrop { tile });
                                break;
                            }
                            (Installation { .. }, Ground { tile }) => {
                                self.send_action(Action::Install { tile });
                                break;
                            }
                            (Instrumenting, Construction(construction)) => {
                                self.send_action(Action::Construct { construction });
                                break;
                            }
                            (Instrumenting, Wall(tile)) => {
                                self.send_action(Action::Deconstruct { tile });
                                break;
                            }
                            (Instrumenting, Crop(crop)) => {
                                self.send_action(Action::WaterCrop { crop });
                                break;
                            }
                            (Material { .. }, Drop(drop)) => {
                                // TODO: generic, not material only
                                self.send_action(Action::TakeItem { drop });
                                break;
                            }
                            (Material { .. }, Construction(construction)) => {
                                // TODO: generic, not material only, generic container?
                                self.send_action(Action::TakeMaterial { construction });
                                break;
                            }
                            _ => {}
                        }
                    }
                }
                Put => match target {
                    Ground { tile } => {
                        self.send_action(Action::DropItem { tile });
                    }
                    Drop(drop) => {
                        self.send_action(Action::PutItem { drop });
                    }
                    Construction(construction) => {
                        self.send_action(Action::PutMaterial { construction });
                    }
                    _ => {}
                },
                Swap => {
                    self.send_action(Action::ToggleBackpack);
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
                            self.send_action(Action::Survey {
                                surveyor,
                                tile,
                                marker,
                            });
                        } else {
                            error!("Not sur")
                        }
                    }
                    Construction(construction) => {
                        self.send_action(Action::RemoveConstruction { construction });
                    }
                    _ => {
                        // beep error
                    }
                },
                Put => {
                    self.send_action(Action::CancelActivity);
                }
                Swap => {
                    self.send_action(Action::ToggleSurveyingOption);
                }
            },
        }
    }
}
