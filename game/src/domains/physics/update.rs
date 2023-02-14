use crate::math::{move_with_collisions, VectorMath};
use crate::physics::{Physics, PhysicsDomain};

const MAX_ELAPSED_TIME: f32 = 0.03; // 40 ms

impl PhysicsDomain {
    pub fn update(&mut self, mut elapsed_time: f32) -> Vec<Physics> {
        let mut events = vec![];
        loop {
            if elapsed_time > MAX_ELAPSED_TIME {
                elapsed_time -= MAX_ELAPSED_TIME;
                events.extend(self.update_iteration(MAX_ELAPSED_TIME));
            } else {
                events.extend(self.update_iteration(elapsed_time));
                break;
            };
        }
        events
    }

    fn update_iteration(&mut self, time: f32) -> Vec<Physics> {
        let mut events = vec![];

        for space in self.spaces.iter() {
            let bodies = &mut self.bodies[space.id.0];
            let barriers = &mut self.barriers[space.id.0];
            for index in 0..bodies.len() {
                let _id = bodies[index].id;

                let body = &bodies[index];
                let delta = body.kind.speed * time;

                let destination = body.direction;

                let distance = body.position.distance(destination);

                if distance > 0.00001 {
                    let position = if delta > distance {
                        destination
                    } else {
                        let movement = body.position.direction_to(destination).mul(delta);
                        body.position.add(movement)
                    };

                    if let Some(position) = move_with_collisions(&body, position, &barriers) {
                        let body = &mut bodies[index];
                        body.position = position;
                        events.push(Physics::BodyPositionChanged {
                            id: body.id.into(),
                            space: body.space.into(),
                            position: body.position,
                        })
                    }
                }
            }
        }

        events
    }
}
