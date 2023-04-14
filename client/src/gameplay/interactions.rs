use log::error;

use game::api::FarmerBound;
use game::assembling::Rotation;
use game::building::{Marker, Structure};
use game::inventory::Function;
use game::inventory::Function::{Assembly, Installation, Instrumenting, Product, Seeding, Shovel};
use game::model::{Activity, CropKey, Purpose};

use crate::gameplay::representation::FarmerRep;
use crate::gameplay::Intention::{Aim, Move, Put, QuickSwap, Swap, Use};
use crate::gameplay::Target::{
    Cementer, CementerContainer, Construction, Crop, Door, Equipment, Ground, Stack, Wall,
};
use crate::gameplay::{Gameplay, Intention, Target};

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
                        self.send_action(FarmerBound::TakeItemFromStack { stack });
                    }
                    Construction(construction) => {
                        self.send_action(FarmerBound::TakeItemFromConstruction { construction });
                    }
                    CementerContainer(cementer, container) => {
                        self.send_action(FarmerBound::TakeItemFromCementer {
                            cementer,
                            container,
                        });
                    }
                    Equipment(equipment) => {
                        self.send_action(FarmerBound::UseEquipment { equipment });
                    }
                    Crop(crop) => {
                        self.send_action(FarmerBound::HarvestCrop { crop });
                    }
                    Door(door) => {
                        self.send_action(FarmerBound::ToggleDoor { door });
                    }
                    Cementer(cementer) => {
                        self.send_action(FarmerBound::ToggleDevice {
                            device: cementer.device,
                        });
                    }
                    _ => {}
                },
                Put => match target {
                    Equipment(equipment) => {
                        self.send_action(FarmerBound::Uninstall { equipment });
                    }
                    Door(door) => {
                        self.send_action(FarmerBound::DisassembleDoor { door });
                    }
                    Cementer(cementer) => {
                        self.send_action(FarmerBound::DisassembleCementer { cementer });
                    }
                    _ => {}
                },
                Swap => {
                    self.send_action(FarmerBound::ToggleBackpack);
                }
                Move => {}
                QuickSwap(_) => {}
                Aim(_) => {}
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
                            (Shovel, Ground { tile }) => {
                                self.send_action(FarmerBound::PlowFarmland { place: tile });
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
                            (Instrumenting, Cementer(cementer)) => {
                                self.send_action(FarmerBound::RepairDevice {
                                    device: cementer.device,
                                });
                                break;
                            }
                            (Product(kind), Crop(crop)) => {
                                if CropKey(kind) == crop.key {
                                    self.send_action(FarmerBound::HarvestCrop { crop });
                                }
                            }
                            (Assembly(_kind), Ground { tile }) => {
                                self.send_action(FarmerBound::StartAssembly {
                                    pivot: tile,
                                    rotation: Rotation::A000,
                                });
                            }
                            (Assembly(_kind), Wall(cell)) => {
                                self.send_action(FarmerBound::StartAssembly {
                                    pivot: cell,
                                    rotation: Rotation::A000,
                                });
                            }

                            (_, Stack(stack)) => {
                                self.send_action(FarmerBound::TakeItemFromStack { stack });
                            }
                            (_, Construction(construction)) => {
                                self.send_action(FarmerBound::TakeItemFromConstruction {
                                    construction,
                                });
                            }
                            (_, CementerContainer(cementer, container)) => {
                                self.send_action(FarmerBound::TakeItemFromCementer {
                                    cementer,
                                    container,
                                });
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
                        self.send_action(FarmerBound::PutItemIntoStack { stack });
                    }
                    Construction(construction) => {
                        self.send_action(FarmerBound::PutItemIntoConstruction { construction });
                    }
                    CementerContainer(cementer, container) => {
                        self.send_action(FarmerBound::PutItemIntoCementer {
                            cementer,
                            container,
                        });
                    }
                    _ => {}
                },
                Swap => {
                    self.send_action(FarmerBound::ToggleBackpack);
                }
                Move => {}
                QuickSwap(_) => {}
                Aim(_) => {}
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
                    self.send_action(FarmerBound::ToggleSurveyingOption {
                        option: selection as u8 + 1,
                    });
                }
                QuickSwap(option) => {
                    self.send_action(FarmerBound::ToggleSurveyingOption { option });
                }
                Move => {
                    self.send_action(FarmerBound::CancelActivity);
                    farmer.activity = Activity::Idle;
                }
                Aim(_) => {}
            },
            Activity::Assembling { assembly } => {
                let assembly = self.assembly.get(&assembly).unwrap();
                match intention {
                    Use => match target {
                        Ground { tile } => {
                            self.send_action(FarmerBound::FinishAssembly {
                                pivot: tile,
                                rotation: assembly.rotation,
                            });
                        }
                        Wall(cell) => {
                            self.send_action(FarmerBound::FinishAssembly {
                                pivot: cell,
                                rotation: assembly.rotation,
                            });
                        }
                        _ => {}
                    },
                    Aim(tile) => {
                        self.send_action(FarmerBound::MoveAssembly {
                            pivot: tile,
                            rotation: assembly.rotation,
                        });
                    }
                    Put => {
                        self.send_action(FarmerBound::CancelAssembly);
                    }
                    Swap => {
                        self.send_action(FarmerBound::MoveAssembly {
                            pivot: assembly.pivot,
                            rotation: assembly.rotation.next(),
                        });
                    }
                    Move => {}
                    QuickSwap(index) => {
                        self.send_action(FarmerBound::MoveAssembly {
                            pivot: assembly.pivot,
                            rotation: Rotation::from_index(index),
                        });
                    }
                }
            }
        }
    }
}
