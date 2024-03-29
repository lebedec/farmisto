use std::collections::HashMap;

use log::error;

use crate::building::{Cell, Grid, Material, Room};

impl Grid {
    pub fn calculate_rooms(map: &Vec<Vec<Cell>>) -> Vec<Room> {
        // TODO: array on stack increases speed to ~2 times!
        // let mut map = [[Cell::default(); Grid::COLUMNS]; Grid::ROWS];
        // for y in 0..Grid::ROWS {
        //     for x in 0..Grid::COLUMNS {
        //         map[y][x] = input_map[y][x];
        //     }
        // }

        let exterior = Room {
            id: Room::EXTERIOR_ID,
            contour: false,
            area_y: 0,
            area: vec![u128::MAX],
            aabb: [0, 0, Grid::COLUMNS, Grid::ROWS],
            active: true,
            material: Material(Material::UNKNOWN),
        };
        let mut unique_id = 1;
        let mut rooms: Vec<Room> = vec![exterior];
        for y in 1..Grid::ROWS {
            let mut row = 0;
            for x in 0..Grid::COLUMNS {
                if !map[y][x].wall {
                    row = row | 1 << (Grid::COLUMNS - x - 1);
                }
            }
            let rooms_above_row: Vec<u128> = rooms
                .iter()
                .map(|room| match room.active {
                    true => *room.area.last().unwrap(),
                    false => 0,
                })
                .collect();
            let expansions = Self::expand_rooms_by_row(row, rooms_above_row);
            let merges = Self::apply_expansion(y, &mut rooms, &mut unique_id, expansions);
            rooms = Self::merge_rooms(merges, rooms);
        }
        Self::include_walls_to_rooms(&mut rooms);
        let mut rooms = Self::merge_rooms_into_buildings(rooms);
        Self::determine_room_bounds(&mut rooms);
        Self::determine_room_dominant_material(map, &mut rooms);
        rooms
    }

    fn determine_room_bounds(rooms: &mut Vec<Room>) {
        for room in rooms {
            if room.id == Room::EXTERIOR_ID {
                continue;
            }
            let min_y = room.area_y;
            let max_y = room.area_y + room.area.len() - 1;
            let mut min_x = Grid::COLUMNS;
            let mut max_x = 0;
            for row in &room.area {
                let left = row.leading_zeros() as usize;
                let right = row.trailing_zeros() as usize;
                let right = Grid::COLUMNS - right - 1;
                if left < min_x {
                    min_x = left;
                }
                if right > max_x {
                    max_x = right;
                }
            }
            room.aabb = [min_x, min_y, max_x, max_y];
        }
    }

    fn determine_room_dominant_material(cells: &Vec<Vec<Cell>>, rooms: &mut Vec<Room>) {
        for room in rooms {
            if room.id == Room::EXTERIOR_ID {
                continue;
            }
            let mut materials = HashMap::new();
            for (i, row) in room.area.iter().enumerate() {
                let y = room.area_y + i;
                let left = row.leading_zeros() as usize;
                let right = row.trailing_zeros() as usize;
                let right = Grid::COLUMNS - right - 1;
                let left = cells[y][left].material;
                let right = cells[y][right].material;
                let counter = materials.entry(left).or_insert(0);
                *counter += 1;
                let counter = materials.entry(right).or_insert(0);
                *counter += 1;
            }
            match materials.iter().max_by(|one, other| one.1.cmp(&other.1)) {
                Some((material, _)) => {
                    room.material = *material;
                }
                None => {
                    error!("Unable to determine room dominant material, zero bounds")
                }
            }
        }
    }

    fn include_walls_to_rooms(rooms: &mut Vec<Room>) {
        for room in rooms {
            if room.id == Room::EXTERIOR_ID {
                continue;
            }
            // grow vertically
            let mut area = vec![0; room.area.len() + 2];
            for i in 1..=room.area.len() {
                area[i - 1] |= room.area[i - 1];
                area[i] |= room.area[i - 1];
                area[i + 1] |= room.area[i - 1];
            }
            // grow horizontally by segments
            for row in area.iter_mut() {
                let mut value: u128 = *row;
                let mut i = 0;
                let mut grow_row = 0;
                while value != 0 {
                    let skip_zeros = value.leading_zeros();
                    i += skip_zeros;
                    value = value << skip_zeros;

                    let width = value.leading_ones() + 2;
                    i -= 1;
                    let val = u128::MAX >> (128 - width);
                    let segment = val << (128 - i - width);
                    grow_row = grow_row | segment;

                    i += width - 1;
                    if width == 128 {
                        break;
                    }
                    value = value << (width - 2);
                }
                *row = grow_row;
            }
            room.area_y = room.area_y - 1;
            room.area = area;
        }
    }

    fn merge_rooms(merges: Vec<[usize; 2]>, mut rooms: Vec<Room>) -> Vec<Room> {
        if !merges.is_empty() {
            let mut to_delete = vec![];
            for [source, destination] in merges {
                to_delete.push(source);
                let source = &mut rooms[source];
                source.active = false;
                let source_y = source.area_y;
                let source_rows = source.area.clone();
                let room = &mut rooms[destination];

                let offset = source_y as isize - room.area_y as isize;
                if offset < 0 {
                    room.area_y = source_y;
                    let mut rows = vec![0; offset.abs() as usize];
                    rows.extend(&room.area);
                    room.area = rows;
                }
                for (index, row) in source_rows.into_iter().enumerate() {
                    let room_index = (index as isize + offset) as usize;
                    room.area[room_index] = room.area[room_index] | row;
                }
            }

            let mut new_rooms = vec![];
            for (index, room) in rooms.into_iter().enumerate() {
                if !to_delete.contains(&index) {
                    new_rooms.push(room);
                }
            }
            new_rooms
        } else {
            rooms
        }
    }

    fn apply_expansion(
        y: usize,
        rooms: &mut Vec<Room>,
        room_id: &mut usize,
        expansions: Vec<u128>,
    ) -> Vec<[usize; 2]> {
        let rooms_before = rooms.len();
        let mut merges = vec![];
        let mut trunk = HashMap::new();
        for room in 0..expansions.len() {
            if room >= rooms_before {
                rooms.push(Room {
                    id: *room_id,
                    contour: false,
                    area_y: y,
                    area: vec![expansions[room]],
                    aabb: [0; 4],
                    active: true,
                    material: Material(Material::UNKNOWN),
                });
                *room_id += 1;
            } else {
                let expansion = expansions[room];
                if expansion != 0 {
                    match trunk.get(&expansion) {
                        None => {
                            rooms[room].area.push(expansion);
                            trunk.insert(expansion, room);
                        }
                        Some(trunk) => {
                            merges.push([room, *trunk]);
                        }
                    }
                } else {
                    rooms[room].active = false;
                }
            }
        }
        merges
    }

    pub fn expand_rooms_by_row(row: u128, rooms: Vec<u128>) -> Vec<u128> {
        let mut appends = vec![0u128; rooms.len()];
        let mut value: u128 = row;
        let mut i = 0;
        while value != 0 {
            let skip_zeros = value.leading_zeros();
            i += skip_zeros;
            value = value << skip_zeros;
            // println!("skip to {}", i);
            let width = value.leading_ones();
            let val = u128::MAX >> (128 - width);
            let segment = val << (128 - i - width);
            // println!("segment {}..{} {:#010b}", i, i + width - 1, segment);
            let mut any = false;
            for (index, append) in appends.iter_mut().enumerate() {
                if index < rooms.len() && rooms[index] & segment != 0 {
                    *append = *append | segment;
                    any = true;
                    continue;
                }
            }
            if !any {
                appends.push(segment);
            }
            i += width;
            if width == 128 {
                break;
            }
            value = value << width;
        }
        appends
    }

    pub fn merge_rooms_into_buildings(mut rooms: Vec<Room>) -> Vec<Room> {
        loop {
            let mut merge = None;
            'collision_detection: for source_index in 1..rooms.len() {
                for destination_index in 1..rooms.len() {
                    if source_index == destination_index {
                        continue;
                    }
                    let source = &rooms[source_index];
                    let destination = &rooms[destination_index];
                    let offset = source.area_y as isize - destination.area_y as isize;
                    if offset < 0 || offset >= destination.area.len() as isize {
                        continue;
                    }
                    let offset = offset as usize;
                    let overlaps = source.area.len().min(destination.area.len() - offset);
                    for i in 0..overlaps {
                        if destination.area[i + offset] & source.area[i] != 0 {
                            merge = Some([source_index, destination_index]);
                            break 'collision_detection;
                        }
                    }
                }
            }
            if let Some(merge) = merge {
                rooms = Self::merge_rooms(vec![merge], rooms);
            } else {
                break;
            }
        }
        rooms
    }
}
