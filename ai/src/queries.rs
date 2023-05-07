use crate::Nature;
use game::physics::SpaceId;

impl Nature {
    pub(crate) fn get_tiles_around(
        &self,
        space: SpaceId,
        center: [usize; 2],
        radius: usize,
    ) -> Vec<[usize; 2]> {
        let game_tiles = self.tiles.get(&space).expect("tiles");

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
                        let tile = &game_tiles[ny][nx];
                        let not_empty = tile.has_barrier || tile.has_hole;
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
