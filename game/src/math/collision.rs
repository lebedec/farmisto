use crate::math::VectorMath;

fn collide_aabb_circle(
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

// bool AABBvsCircle( Manifold *m )
// {
// // Setup a couple pointers to each object
// Object *A = m->A
// Object *B = m->B
// // Vector from A to B
// Vec2 n = B->pos - A->pos
// // Closest point on A to center of B
// Vec2 closest = n
// // Calculate half extents along each axis
// float x_extent = (A->aabb.max.x - A->aabb.min.x) / 2
// float y_extent = (A->aabb.max.y - A->aabb.min.y) / 2
// // Clamp point to edges of the AABB
// closest.x = Clamp( -x_extent, x_extent, closest.x )
// closest.y = Clamp( -y_extent, y_extent, closest.y )
// bool inside = false
// // Circle is inside the AABB, so we need to clamp the circle's center
// // to the closest edge
// if(n == closest)
// {
// inside = true
// // Find closest axis
// if(abs( n.x ) > abs( n.y ))
// {
// // Clamp to closest extent
// if(closest.x > 0)
// closest.x = x_extent
// else
// closest.x = -x_extent
// }
// // y axis is shorter
// else
// {
// // Clamp to closest extent
// if(closest.y > 0)
// closest.y = y_extent
// else
// closest.y = -y_extent
// }
// }
// Vec2 normal = n - closest
// real d = normal.LengthSquared( )
// real r = B->radius
// // Early out of the radius is shorter than distance to closest point and
// // Circle not inside the AABB
// if(d > r * r && !inside)
// return false
// // Avoided sqrt until we needed
// d = sqrt( d )
// // Collision normal needs to be flipped to point outside if circle was
// // inside the AABB
// if(inside)
// {
// m->normal = -n
// m->penetration = r - d
// }
// else
// {
// m->normal = n
// m->penetration = r - d
// }
// return true
// }

#[derive(Debug)]
pub struct Collision {
    point: [f32; 2],
    normal: [f32; 2],
    penetration: f32,
}

pub fn collide_circle_to_circle(p1: [f32; 2], p2: [f32; 2], r1: f32, r2: f32) -> Option<Collision> {
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

pub fn collide_circle_to_segment(
    center: [f32; 2],
    r: f32,
    seg_a: [f32; 2],
    seg_b: [f32; 2],
    seg_r: f32,
) -> Option<Collision> {
    let seg_delta = seg_b.sub(seg_a);
    let closest_t = seg_delta.dot(center.sub(seg_a)).clamp(0.0, 1.0) / seg_delta.length_squared();
    let closest = seg_a.add(seg_delta.mul(closest_t));
    collide_circle_to_circle(center, closest, r, seg_r)
}

pub fn collide_circle_to_rect(
    center: [f32; 2],
    radius: f32,
    rect_center: [f32; 2],
    rect_bounds: [f32; 2],
) -> Option<[f32; 2]> {
    let [cx, cy] = center;
    let [rx, ry] = rect_center.sub(rect_bounds.mul(0.5));
    let [rw, rh] = rect_bounds;
    let test_x = if cx < rx {
        rx
    } else if cx > rx + rw {
        rx + rw
    } else {
        cx
    };

    let test_y = if cy < ry {
        ry
    } else if cy > ry + rh {
        ry + rh
    } else {
        cy
    };

    let dist_x = cx - test_x;
    let dist_y = cy - test_y;
    let distance = ((dist_x * dist_x) + (dist_y * dist_y)).sqrt();

    if distance <= radius {
        println!("TEST {}x{}", test_x, test_y);
        Some([test_x, test_y])
    } else {
        None
    }
}

// boolean circleRect(float cx, float cy, float radius, float rx, float ry, float rw, float rh) {
//
// // temporary variables to set edges for testing
// float testX = cx;
// float testY = cy;
//
// // which edge is closest?
// if (cx < rx)         testX = rx;      // test left edge
// else if (cx > rx+rw) testX = rx+rw;   // right edge
// if (cy < ry)         testY = ry;      // top edge
// else if (cy > ry+rh) testY = ry+rh;   // bottom edge
//
// // get distance from closest edges
// float distX = cx-testX;
// float distY = cy-testY;
// float distance = sqrt( (distX*distX) + (distY*distY) );
//
// // if the distance is less than the radius, collision!
// if (distance <= radius) {
// return true;
// }
// return false;
// }

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

pub fn move_with_collisions(
    body: &impl Collider,
    destination: [f32; 2],
    barriers: &Vec<impl Collider>,
) -> Option<[f32; 2]> {
    let mut blocked = false;
    let mut offsets = vec![];
    for barrier in barriers.iter() {
        // let collision = collide_circle_to_segment(
        //     destination,
        //     body.bounds()[0],
        //     barrier.position().add([-0.5, -0.5]),
        //     barrier.position().add([-0.5, 0.5]),
        //     0.0,
        // );
        // if let Some(collision) = collision {
        //     println!("COLLISION {:?}", collision);
        //     blocked = true;
        //     // return Some(
        //     //     collision
        //     //         .point
        //     //         .add(collision.normal.mul(-collision.penetration)),
        //     // );
        //     break;
        // }
        // if let Some(contact) = collide_circle_to_rect(
        //     destination,
        //     body.bounds()[0],
        //     barrier.position(),
        //     barrier.bounds(),
        // ) {
        //     blocked = true;
        //     let offset = contact.direction_to(body.position()).mul(body.bounds()[0]);
        //     //break;
        //     return Some(contact.add(offset));
        // }
        // if test_rect_collision(
        //     destination,
        //     body.bounds(), // body radius
        //     barrier.position(),
        //     barrier.bounds(),
        // ) {
        //     blocked = true;
        //     break;
        // }
        if let Some(contact) = collide_aabb_circle(
            barrier.position(),
            barrier.bounds(),
            destination,
            body.bounds()[0],
        ) {
            let movement = destination.sub(body.position());
            let offset = contact.normal.normalize().mul(contact.penetration);
            // println!(
            //     "CONTACT {:?} offset {:?} movement {:?} dir {:?} L {} NEW {:?} R {}",
            //     contact,
            //     offset,
            //     movement,
            //     movement.normalize(),
            //     movement.length(),
            //     movement
            //         .normalize()
            //         .mul(movement.length() - contact.penetration),
            //     body.bounds()[0],
            // );
            blocked = true;
            offsets.push(offset);
        }
    }

    if !blocked {
        Some(destination)
    } else {
        if offsets.len() == 1 {
            Some(destination.add(offsets[0]))
        } else {
            None
        }
    }
}
