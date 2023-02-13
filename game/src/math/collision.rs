use crate::math::VectorMath;

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
        if test_rect_collision(
            destination,
            body.bounds(), // body radius
            barrier.position(),
            barrier.bounds(),
        ) {
            blocked = true;
            break;
        }
    }

    if !blocked {
        Some(destination)
    } else {
        None
    }
}
