use crate::math::{ArrayIndex, TileMath, VectorMath};

pub fn cast_ray(
    start: [f32; 2],
    end: [f32; 2],
    holes: &Vec<Vec<u8>>,
    filter: &[u8],
) -> Vec<[f32; 2]> {
    let mut contacts = vec![];
    let line = rasterize_line(start.to_tile(), end.to_tile());
    for point in line {
        let [x, y] = point;
        let h = holes[y][x];
        if filter.contains(&h) && point.position() != end {
            contacts.push(point.position())
        }
    }
    contacts
}

pub fn cast_ray2(start: [f32; 2], end: [f32; 2], holes: &Vec<u8>, filter: &[u8]) -> Vec<[f32; 2]> {
    let mut contacts = vec![];
    let line = rasterize_line(start.to_tile(), end.to_tile());
    for point in line {
        let h = holes[point.fit(128)];
        if filter.contains(&h) && point.position() != end {
            contacts.push(point.position())
        }
    }
    contacts
}

pub fn rasterize_line(start: [usize; 2], end: [usize; 2]) -> Vec<[usize; 2]> {
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
