use crate::Nature;
use game::math::ArrayIndex;
use game::physics::SpaceId;

impl Nature {
    pub(crate) fn get_tiles_around(
        &self,
        center: [usize; 2],
        radius: usize,
        game_tiles: &Vec<u8>,
    ) -> Vec<[usize; 2]> {
        let mut tiles = vec![];
        let mut map = vec![vec![0; 128]; 128];
        let mut frontier = vec![center];
        let mut wave = 1;
        loop {
            let mut new_wave = vec![];
            for current in frontier {
                let [cx, cy] = current;
                map[cy][cx] = wave;
                tiles.push(current);
                let cx = cx as isize;
                let cy = cy as isize;
                let steps: [[isize; 2]; 4] =
                    [[cx, cy - 1], [cx - 1, cy], [cx + 1, cy], [cx, cy + 1]];
                for next in steps {
                    let [nx, ny] = next;
                    if nx >= 0 && nx < 128 && ny >= 0 && ny < 128 {
                        let nx = nx as usize;
                        let ny = ny as usize;
                        let nindex = [nx, ny].fit(128);
                        let tile = game_tiles[nindex];
                        let not_empty = tile > 0;
                        if not_empty {
                            // mark blocked tiles
                            map[ny][nx] = 1;
                        } else if map[ny][nx] == 0 {
                            map[ny][nx] = wave;
                            new_wave.push([nx as usize, ny as usize]);
                        }
                    }
                }
            }
            wave += 1;
            if wave == radius + 2 || new_wave.len() == 0 {
                break;
            }
            frontier = new_wave;
        }
        tiles
    }
}
