#[inline]
fn test_rect_collision(a: [f32; 4], b: [f32; 4]) -> bool {
    let [ax, ay, aw, ah] = a;
    let [bx, by, bw, bh] = b;
    (ax + aw >= bx && bx + bw >= ax) && (ay + ah >= by && by + bh >= ay)
}
