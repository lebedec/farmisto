use crate::math::{TileMath, VectorMath};
use crate::physics::{BarrierId, PhysicsDomain, PhysicsError, SpaceId};
use log::info;

impl PhysicsDomain {
    pub fn cast_ray(
        &self,
        space: SpaceId,
        start: [f32; 2],
        end: [f32; 2],
    ) -> Result<Vec<[f32; 2]>, PhysicsError> {
        let mut contacts = vec![];
        let line = rasterize_line(start.to_tile(), end.to_tile());
        let space = self.get_space(space)?;
        for point in line {
            if let Some(_barrier) = self.get_barrier_at(space.id, point.to_position()) {
                // if barrier.id != exclude {
                //     info!("barrier {:?}", barrier.position);
                //     contacts.push(barrier.position);
                // }
            } else {
                let [x, y] = point;
                if space.holes[y][x] == 1 {
                    contacts.push(point.to_position())
                }
            }
        }
        Ok(contacts)
    }
}

fn rasterize_line(start: [usize; 2], end: [usize; 2]) -> Vec<[usize; 2]> {
    let [x, y] = start;
    let mut x = x as isize;
    let mut y = y as isize;
    let [x2, y2] = end;
    let x2 = x2 as isize;
    let y2 = y2 as isize;
    let w = x2 - x;
    let h = y2 - y;
    let mut dx1 = 0;
    let mut dy1 = 0;
    let mut dx2 = 0;
    let mut dy2 = 0;
    if w < 0 {
        dx1 = -1
    } else if w > 0 {
        dx1 = 1
    }
    if h < 0 {
        dy1 = -1
    } else if h > 0 {
        dy1 = 1
    }
    if w < 0 {
        dx2 = -1
    } else if w > 0 {
        dx2 = 1
    }
    let mut longest = w.abs();
    let mut shortest = h.abs();
    if !(longest > shortest) {
        longest = h.abs();
        shortest = w.abs();
        if h < 0 {
            dy2 = -1
        } else if h > 0 {
            dy2 = 1
        }
        dx2 = 0;
    }
    let mut result = vec![];
    let mut numerator = longest >> 1;
    for _ in 0..=longest {
        result.push([x as usize, y as usize]);
        numerator += shortest;
        if !(numerator < longest) {
            numerator -= longest;
            x += dx1;
            y += dy1;
        } else {
            x += dx2;
            y += dy2;
        }
    }
    result
}
