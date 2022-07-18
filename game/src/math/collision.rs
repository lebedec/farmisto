use crate::math::VectorMath;

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

pub fn detect_collision(
    body: &impl Collider,
    position: [f32; 2],
    barriers: &Vec<impl Collider>,
) -> Option<[f32; 2]> {
    let mut blocked = false;
    for barrier in barriers.iter() {
        if test_rect_collision(
            position,
            body.bounds(), // body radius
            barrier.position(),
            barrier.bounds(),
        ) {
            blocked = true;
            break;
        }
    }

    if !blocked {
        Some(position)
    } else {
        None
    }
}
