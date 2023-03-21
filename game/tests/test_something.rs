use std::collections::HashMap;
use std::ops::Add;
use std::time::Instant;

#[test]
fn test_hash() {
    let mut keys = vec![];
    let mut string_keys = vec![];
    let mut t1 = Instant::now();
    for behaviour in 0..10 {
        for target in 0..20 {
            for decision in 0..7 {
                for consideration in 0..5 {
                    let key = [
                        behaviour as u8,
                        target as u8,
                        decision as u8,
                        consideration as u8,
                    ];
                    keys.push(key);
                    let key = format!("{key:?}");
                    string_keys.push(key);
                }
            }
        }
    }
    let t = t1.elapsed().as_secs_f32();
    println!("keys: {}, geration time: {}", keys.len(), t);

    let keys2 = keys.clone();

    t1 = Instant::now();
    let mut key_hash = HashMap::new();
    for (i, key) in keys.into_iter().enumerate() {
        key_hash.insert(key, i as f32 * 0.1);
    }
    let t = t1.elapsed().as_secs_f32();
    println!("A: {t}");

    t1 = Instant::now();
    let mut key_hash = HashMap::new();
    for (i, key) in string_keys.into_iter().enumerate() {
        key_hash.insert(key, i as f32 * 0.1);
    }
    let t = t1.elapsed().as_secs_f32();
    println!("B: {t}");

    t1 = Instant::now();
    let mut table = vec![vec![vec![vec![0.0; 5]; 7]; 20]; 10];
    for (i, key) in keys2.into_iter().enumerate() {
        let [b, t, d, c] = key;
        table[b as usize][t as usize][d as usize][c as usize] = i as f32 * 0.1;
    }
    let t = t1.elapsed().as_secs_f32();
    println!("B: {t}")
}

#[test]
fn test_drop_item_with_empty_hands() {
    let mut map = vec![vec![0; 21]; 21];
    let center = [9, 9];
    let radius = 8;
    let output: isize = (1..=radius).map(|wave| wave * 4).sum();
    println!("OUTPUT: {output}");
    let t1 = Instant::now();
    let mut frontier = vec![vec![center]];
    let mut wave = 1;
    let in_bounds =
        |point: [usize; 2]| point[0] >= 0 && point[0] < 21 && point[1] >= 0 && point[1] < 21;
    while let Some(wave_cells) = frontier.pop() {
        let mut new_wave = vec![];
        for current in wave_cells {
            let [cx, cy] = current;
            map[cy][cx] = wave;
            let t = [current[0], current[1] - 1];
            let l = [current[0] - 1, current[1]];
            let r = [current[0] + 1, current[1]];
            let b = [current[0], current[1] + 1];
            let directions = [t, l, r, b];
            for next in directions {
                if in_bounds(next) && map[next[1]][next[0]] == 0 {
                    map[next[1]][next[0]] = 9;
                    new_wave.push(next);
                }
            }
        }
        let l = new_wave.len();
        // println!("{wave}: L{l}");
        frontier.push(new_wave);
        wave += 1;
        if wave == radius + 2 {
            break;
        }
    }
    let elapsed = t1.elapsed().as_secs_f32();
    println!("Elapsed: {elapsed}");
    let mut repr = String::new();
    for row in map {
        let row: Vec<String> = row.iter().map(|value| value.to_string()).collect();
        repr += &row.join(" ");
        repr += "\n";
    }
    println!("{repr}")
}
