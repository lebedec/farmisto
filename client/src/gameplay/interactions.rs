use crate::gameplay::representation::FarmerRep;
use crate::gameplay::Intention::{Put, Swap, Use};
use crate::gameplay::{Activity, Gameplay, Intention, Target};
use game::api::Action;
use game::building::Marker;
use game::model::Purpose;
use log::error;

impl Gameplay {
    pub fn interact_with(&mut self, farmer: &FarmerRep, target: Target, intention: Intention) {
        match self.activity {
            Activity::Idle => match intention {
                Use => match target {
                    Target::Drop(drop) => {
                        self.send_action(Action::TakeItem { drop });
                        self.activity = Activity::Delivery;
                        // if hands capacity
                    }
                    Target::Construction(construction) => {
                        self.send_action(Action::TakeMaterial { construction });
                        self.activity = Activity::Delivery;
                    }
                    Target::Equipment(equipment) => match equipment.purpose {
                        Purpose::Surveying { .. } => {
                            self.activity = Activity::Surveying {
                                equipment,
                                selection: 0,
                            };
                        }
                        Purpose::Moisture { .. } => {}
                    },
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
                    self.activity = Activity::Instrumenting;
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
                        if self.items.get(&farmer.entity.hands).unwrap().len() == 1 {
                            self.activity = Activity::Idle;
                        }
                    }
                    Target::Drop(drop) => {
                        self.send_action(Action::PutItem { drop });
                        if self.items.get(&farmer.entity.hands).unwrap().len() == 1 {
                            self.activity = Activity::Idle;
                        }
                    }
                    Target::Construction(construction) => {
                        self.send_action(Action::PutMaterial { construction });
                        if self.items.get(&farmer.entity.hands).unwrap().len() == 1 {
                            self.activity = Activity::Idle;
                        }
                    }
                    Target::Equipment(_) => {
                        // beep error
                    }
                    Target::Wall(_) => {}
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
                Put => self.activity = Activity::Idle,
                Swap => {
                    self.activity = Activity::Surveying {
                        equipment,
                        selection: (selection + 1) % 4,
                    }
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
                    _ => {}
                },
                Put => {}
                Swap => {
                    self.send_action(Action::ToggleBackpack);
                    self.activity = Activity::Idle;
                }
            },
            Activity::Installing { item } => match intention {
                Use => match target {
                    Target::Ground(tile) => {
                        self.send_action(Action::Install { item, tile });
                        self.activity = Activity::Idle;
                    }
                    _ => {}
                },
                Put => match target {
                    Target::Ground(tile) => {
                        self.send_action(Action::DropItem { tile });
                        self.activity = Activity::Idle;
                    }
                    Target::Drop(drop) => {
                        self.send_action(Action::PutItem { drop });
                        self.activity = Activity::Idle;
                    }
                    _ => {}
                },
                Swap => {
                    self.send_action(Action::ToggleBackpack);
                    self.activity = Activity::Idle;
                }
            },
        }
    }
}
