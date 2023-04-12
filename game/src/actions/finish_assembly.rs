use crate::api::{ActionError, Event};
use crate::inventory::ContainerId;
use crate::math::TileMath;
use crate::model::{Activity, AssemblyTarget, Farmer, Farmland};
use crate::working::DeviceId;
use crate::{occur, position_of, Game};

impl Game {
    pub(crate) fn finish_assembly(
        &mut self,
        farmer: Farmer,
        farmland: Farmland,
    ) -> Result<Vec<Event>, ActionError> {
        let activity = self.universe.get_farmer_activity(farmer)?;
        let assembly = activity.as_assembling()?;
        let destination = self
            .assembling
            .get_placement(assembly.placement)?
            .pivot
            .to_position();
        self.ensure_target_reachable(farmland.space, farmer, destination)?;
        let key = assembly.key;
        let assembly_kind = self.known.assembly.get(key)?;
        let (placement, finish_placement) = self.assembling.finish_placement(assembly.placement)?;
        let events = match &assembly_kind.target {
            AssemblyTarget::Door { door } => {
                let position = position_of(placement.pivot);
                let closed = true;
                let (barrier, create_barrier) = self.physics.create_barrier(
                    farmland.space,
                    door.barrier.clone(),
                    position,
                    closed,
                    false,
                )?;
                let use_assembly_kit = self.inventory.use_items_from(farmer.hands)?;
                occur![
                    use_assembly_kit(),
                    finish_placement(),
                    create_barrier(),
                    self.appear_door(door.key, barrier, placement.id),
                    self.universe.vanish_assembly(assembly),
                    self.universe.change_activity(farmer, Activity::Idle),
                ]
            }
            AssemblyTarget::Cementer { cementer } => {
                let [input, output] = self.inventory.containers_id.introduce().many(ContainerId);
                let create_input = self.inventory.add_container(input, &cementer.input)?;
                let create_output = self.inventory.add_container(output, &cementer.output)?;
                let use_assembly_kit = self.inventory.use_items_from(farmer.hands)?;
                let position = position_of(placement.pivot);
                let (barrier, create_barrier) = self.physics.create_barrier(
                    farmland.space,
                    cementer.barrier.clone(),
                    position,
                    true,
                    false,
                )?;
                let device = self.working.devices_id.introduce().one(DeviceId);
                let create_device = self.working.create_device(device, &cementer.device)?;
                occur![
                    use_assembly_kit(),
                    finish_placement(),
                    create_barrier(),
                    create_device(),
                    create_input(),
                    create_output(),
                    self.appear_cementer(
                        cementer.key,
                        barrier,
                        placement.id,
                        input,
                        device,
                        output
                    ),
                    self.universe.vanish_assembly(assembly),
                    self.universe.change_activity(farmer, Activity::Idle),
                ]
            }
        };
        Ok(events)
    }
}
