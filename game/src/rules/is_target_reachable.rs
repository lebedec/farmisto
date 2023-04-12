use crate::api::ActionError;
use crate::math::{Position, VectorMath};
use crate::model::Farmer;
use crate::physics::SpaceId;
use crate::Game;

impl Game {
    pub fn ensure_target_reachable(
        &self,
        space: SpaceId,
        farmer: Farmer,
        target: Position,
    ) -> Result<(), ActionError> {
        let body = self.physics.get_body(farmer.body)?;
        let actor = body.position;
        if self.is_target_reachable(space, actor, target)? {
            Ok(())
        } else {
            Err(ActionError::TargetUnreachable)
        }
    }

    pub fn is_target_reachable(
        &self,
        space: SpaceId,
        actor: Position,
        target: Position,
    ) -> Result<bool, ActionError> {
        if actor.distance(target) > 2.0 {
            Ok(false)
        } else {
            let contacts = self.physics.cast_ray(space, actor, target)?;
            Ok(contacts.is_empty())
        }
    }
}
