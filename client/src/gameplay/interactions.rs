use crate::gameplay::representation::FarmerRep;
use crate::gameplay::Intention::{Put, Swap, Use};
use crate::gameplay::{Gameplay, Intention, Target};
use game::api::Action;
use game::building::Marker;
use game::model::{Activity, Purpose};
use log::error;

impl Gameplay {
    pub fn interact_with(&mut self, farmer: &mut FarmerRep, target: Target, intention: Intention) {
        match farmer.activity {
            Activity::Idle => match intention {
                Use => match target {
                    Target::Drop(drop) => {
                        self.send_action(Action::TakeItem { drop });
                    }
                    Target::Construction(construction) => {
                        self.send_action(Action::TakeMaterial { construction });
                    }
                    Target::Equipment(equipment) => {
                        self.send_action(Action::UseEquipment { equipment });
                    }
                    _ => {}
                },
                Put => match target {
                    Target::Equipment(equipment) => {
                        self.send_action(Action::Uninstall { equipment });
                    }
                    _ => {}
                },
                Swap => {
                    self.send_action(Action::ToggleBackpack);
                }
            },
            Activity::Delivery => match intention {
                Use => match target {
                    Target::Drop(drop) => {
                        self.send_action(Action::TakeItem { drop });
                        // if hands capacity
                    }
                    Target::Construction(construction) => {
                        self.send_action(Action::TakeMaterial { construction });
                    }
                    _ => {}
                },
                Put => match target {
                    Target::Ground(tile) => {
                        self.send_action(Action::DropItem { tile });
                    }
                    Target::Drop(drop) => {
                        self.send_action(Action::PutItem { drop });
                    }
                    Target::Construction(construction) => {
                        self.send_action(Action::PutMaterial { construction });
                    }
                    Target::Equipment(_) => {
                        // beep error
                    }
                    Target::Wall(_) => {}
                    _ => {}
                },
                Swap => {
                    // swap cargos (usefull for different jobs)
                }
            },
            Activity::Surveying {
                equipment,
                selection,
            } => match intention {
                Use => match target {
                    Target::Ground(tile) => {
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
                    Target::Construction(construction) => {
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
            Activity::Instrumenting => match intention {
                Use => match target {
                    Target::Construction(construction) => {
                        self.send_action(Action::Construct { construction });
                    }
                    Target::Wall(tile) => {
                        self.send_action(Action::Deconstruct { tile });
                    }
                    Target::Ground(tile) => {
                        self.send_action(Action::PlantCrop { tile });
                    }
                    Target::Crop(crop) => {
                        self.send_action(Action::WaterCrop { crop });
                    }
                    _ => {}
                },
                Put => {}
                Swap => {
                    self.send_action(Action::ToggleBackpack);
                }
            },
            Activity::Installing { item } => match intention {
                Use => match target {
                    Target::Ground(tile) => {
                        self.send_action(Action::Install { item, tile });
                    }
                    _ => {}
                },
                Put => match target {
                    Target::Ground(tile) => {
                        self.send_action(Action::DropItem { tile });
                    }
                    Target::Drop(drop) => {
                        self.send_action(Action::PutItem { drop });
                    }
                    _ => {}
                },
                Swap => {
                    self.send_action(Action::ToggleBackpack);
                }
            },
        }
    }
}
