use crate::api::{ActionError, Event};
use crate::building::SurveyorId;
use crate::model::{Activity, Equipment, EquipmentKey, Farmer, Purpose, UniverseError};
use crate::physics::BarrierId;
use crate::{occur, Game};

impl Game {
    pub(crate) fn toggle_surveying_option(
        &mut self,
        farmer: Farmer,
        option: u8,
    ) -> Result<Vec<Event>, ActionError> {
        let activity = self.universe.get_farmer_activity(farmer)?;
        if let Activity::Surveying {
            equipment,
            mut selection,
        } = activity
        {
            selection = option as usize % 4;
            let activity = Activity::Surveying {
                equipment,
                selection,
            };
            let events = self.universe.change_activity(farmer, activity);
            Ok(occur![events,])
        } else {
            // TODO: rework expected activity attribute
            return Err(UniverseError::FarmerActivityMismatch {
                actual: activity,
                expected: Activity::Surveying {
                    equipment: Equipment {
                        id: 0,
                        key: EquipmentKey(0),
                        purpose: Purpose::Surveying {
                            surveyor: SurveyorId(0),
                        },
                        barrier: BarrierId(0),
                    },
                    selection: 0,
                },
            }
            .into());
        }
    }
}
