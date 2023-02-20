use crate::math::VectorMath;

fn collide_aabb_to_circle(
    a_pos: [f32; 2],
    a_bounds: [f32; 2],
    b_pos: [f32; 2],
    r: f32,
) -> Option<Collision> {
    let n = b_pos.sub(a_pos);
    let mut closest = n;
    let x_extent = a_bounds[0] / 2.0;
    let y_extent = a_bounds[1] / 2.0;
    closest[0] = closest[0].clamp(-x_extent, x_extent);
    closest[1] = closest[1].clamp(-y_extent, y_extent);
    let mut inside = false;

    if n == closest {
        // Circle is inside the AABB, so we need to clamp the circle's center
        // to the closest edge
        inside = true;
        if n[0].abs() > n[1].abs() {
            if closest[0] > 0.0 {
                closest[0] = x_extent
            } else {
                closest[0] = -x_extent
            }
        } else {
            if closest[1] > 0.0 {
                closest[1] = y_extent
            } else {
                closest[1] = -y_extent
            }
        }
    }

    let normal = n.sub(closest);
    let mut d = normal.length_squared();

    // Early out of the radius is shorter than distance to closest point and
    // Circle not inside the AABB
    if !inside && d > r * r {
        return None;
    }

    d = d.sqrt();
    if (inside) {
        Some(Collision {
            point: [0.0, 0.0],
            normal: n.neg(),
            penetration: r - d,
        })
    } else {
        Some(Collision {
            point: [0.0, 0.0],
            normal: n,
            penetration: r - d,
        })
    }
}

#[derive(Debug)]
pub struct Collision {
    point: [f32; 2],
    normal: [f32; 2],
    penetration: f32,
}

pub fn collide_circle_to_circle(p1: [f32; 2], r1: f32, p2: [f32; 2], r2: f32) -> Option<Collision> {
    let mindist = r1 + r2;
    let delta = p2.sub(p1);
    let distsq = delta.length_squared();
    if distsq >= mindist * mindist {
        return None;
    }
    if distsq == 0.0 {
        let dist = 0.0;
        let point = p1;
        let normal = [1.0, 0.0];
        let penetration = dist - mindist;
        Some(Collision {
            point,
            normal,
            penetration,
        })
    } else {
        let dist = distsq.sqrt();
        let point = p1.add(delta.mul(0.5 + (r1 - 0.5 * mindist) / dist));
        let normal = delta.mul(1.0 / dist);
        let penetration = dist - mindist;
        Some(Collision {
            point,
            normal,
            penetration,
        })
    }
}

#[inline]
pub fn test_rect_collision(
    a: [f32; 2],
    a_bounds: [f32; 2],
    b: [f32; 2],
    b_bounds: [f32; 2],
) -> bool {
    let [aw, ah] = a_bounds;
    let [ax, ay] = a.sub(a_bounds.mul(0.5));
    let [bw, bh] = b_bounds;
    let [bx, by] = b.sub(b_bounds.mul(0.5));
    (ax + aw >= bx && bx + bw >= ax) && (ay + ah >= by && by + bh >= ay)
}

pub trait Collider {
    fn position(&self) -> [f32; 2];
    fn bounds(&self) -> [f32; 2];
}

pub fn test_collisions(
    body: &impl Collider,
    destination: [f32; 2],
    barriers: &Vec<impl Collider>,
) -> Option<Vec<[f32; 2]>> {
    let mut blocked = false;
    let mut offsets = vec![];
    for barrier in barriers.iter() {
        if let Some(contact) = collide_aabb_to_circle(
            barrier.position(),
            barrier.bounds(),
            destination,
            body.bounds()[0],
        ) {
            let offset = contact.normal.normalize().mul(contact.penetration);
            blocked = true;
            offsets.push(offset);
        }
    }

    if !blocked {
        None
    } else {
        Some(offsets)
    }
}
