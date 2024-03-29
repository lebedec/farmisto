use crate::api::{ActionError, Event};
use crate::inventory::ContainerId;
use crate::math::{TileMath, VectorMath};
use crate::model::{Activity, AssemblyTarget, Farmer, Farmland};
use crate::working::DeviceId;
use crate::{occur, Game};

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
            .position();
        self.ensure_target_reachable(farmer.body, destination)?;
        self.ensure_tile_empty(farmland, destination.to_tile())?;
        let key = assembly.key;
        let (placement, finish_placement) = self.assembling.finish_placement(assembly.placement)?;
        let assembly_kind = self.known.assembly.get(key)?;
        let events = match &assembly_kind.target {
            AssemblyTarget::Door { door } => {
                let position = placement.pivot.position();
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
                let create_input = self.inventory.add_empty_container(input, &cementer.input)?;
                let create_output = self
                    .inventory
                    .add_empty_container(output, &cementer.output)?;
                let use_assembly_kit = self.inventory.use_items_from(farmer.hands)?;
                let position = placement.pivot.position();
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
            AssemblyTarget::Composter { composter } => {
                let [input, output] = self.inventory.containers_id.introduce().many(ContainerId);
                let create_input = self
                    .inventory
                    .add_empty_container(input, &composter.input)?;
                let create_output = self
                    .inventory
                    .add_empty_container(output, &composter.output)?;
                let use_assembly_kit = self.inventory.use_items_from(farmer.hands)?;
                let position = placement.pivot.position();
                let (barrier, create_barrier) = self.physics.create_barrier(
                    farmland.space,
                    composter.barrier.clone(),
                    position,
                    true,
                    false,
                )?;
                let device = self.working.devices_id.introduce().one(DeviceId);
                let create_device = self.working.create_device(device, &composter.device)?;
                occur![
                    use_assembly_kit(),
                    finish_placement(),
                    create_barrier(),
                    create_device(),
                    create_input(),
                    create_output(),
                    self.appear_composter(
                        composter.key,
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
            AssemblyTarget::Rest { rest } => {
                let position = placement.pivot.position();
                let (barrier, create_barrier) = self.physics.create_barrier(
                    farmland.space,
                    rest.barrier.clone(),
                    position,
                    true,
                    false,
                )?;
                let use_assembly_kit = self.inventory.use_items_from(farmer.hands)?;
                occur![
                    use_assembly_kit(),
                    finish_placement(),
                    create_barrier(),
                    self.appear_rest(rest.key, barrier, placement.id),
                    self.universe.vanish_assembly(assembly),
                    self.universe.change_activity(farmer, Activity::Idle),
                ]
            }
        };
        Ok(events)
    }
}
