use crate::api::ActionError;
use crate::math::{Position, VectorMath};
use crate::model::Farmer;
use crate::physics::{Body, BodyId, SpaceId};
use crate::Game;

impl Game {
    pub fn ensure_target_reachable(
        &self,
        body: BodyId,
        target: Position,
    ) -> Result<(), ActionError> {
        let body = self.physics.get_body(body)?;
        let actor = body.position;
        if self.is_target_reachable(body.space, actor, target)? {
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
            let contacts = self.physics.cast_ray(space, actor, target, &[1])?;
            Ok(contacts.is_empty())
        }
    }
}
