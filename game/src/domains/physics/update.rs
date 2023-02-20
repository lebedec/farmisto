use crate::math::{collide_circle_to_circle, test_collisions, VectorMath};
use crate::physics::{Hole, Physics, PhysicsDomain};

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
            let sensors = &mut self.sensors[space.id.0];
            for index in 0..bodies.len() {
                let _id = bodies[index].id;

                let body = &bodies[index];
                let delta = body.kind.speed * time;

                let destination = body.destination;

                let distance = body.position.distance(destination);

                if distance > 0.00001 {
                    let body_next_position = if delta > distance {
                        destination
                    } else {
                        let movement = body.position.direction_to(destination).mul(delta);
                        body.position.add(movement)
                    };

                    // Collision Detection
                    let holes = generate_holes(body_next_position, body.kind.radius, &space.holes);
                    let holes_offsets = match test_collisions(&body, body_next_position, &holes) {
                        Some(offsets) => offsets,
                        None => vec![],
                    };
                    if holes_offsets.len() > 1 {
                        // body already blocked by holes, no need to test collision with barriers
                        continue;
                    }

                    let offsets = match test_collisions(&body, body_next_position, &barriers) {
                        None => holes_offsets,
                        Some(mut barrier_offsets) => {
                            barrier_offsets.extend(holes_offsets);
                            barrier_offsets
                        }
                    };

                    if offsets.len() > 1 {
                        // blocked
                        continue;
                    }

                    let position = if offsets.len() == 1 {
                        // apply collision response from one contact point only
                        // simple solution to prevent continuous collision detection
                        body_next_position.add(offsets[0])
                    } else {
                        body_next_position
                    };
                    let body = &mut bodies[index];
                    body.position = position;
                    events.push(Physics::BodyPositionChanged {
                        id: body.id.into(),
                        space: body.space.into(),
                        position: body.position,
                    });
                }
            }

            for index in 0..sensors.len() {
                let sensor = &mut sensors[index];
                sensor.signals = vec![];
                for body in &self.bodies[space.id.0] {
                    let collision = collide_circle_to_circle(
                        sensor.position,
                        sensor.kind.radius,
                        body.position,
                        body.kind.radius,
                    );
                    if let Some(_) = collision {
                        sensor.signals.push(body.position.sub(sensor.position));
                    }
                }
            }
        }

        events
    }
}

pub fn generate_holes(position: [f32; 2], r: f32, holes_map: &Vec<Vec<u8>>) -> Vec<Hole> {
    let [body_x, body_y] = position;
    let (min_x, min_y) = ((body_x - r) as usize, (body_y - r) as usize);
    let (max_x, max_y) = ((body_x + r) as usize, (body_y + r) as usize);

    let mut holes = vec![];
    for hole_y in min_y..=max_y {
        for hole_x in min_x..=max_x {
            if holes_map[hole_y][hole_x] == 1 {
                holes.push(Hole {
                    position: [hole_x as f32 + 0.5, hole_y as f32 + 0.5],
                    bounds: [1.0, 1.0],
                })
            }
        }
    }
    holes
}
