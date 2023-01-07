import sqlite3
import struct
from io import BytesIO
from typing import List


def generate_land(land_id: int, user_define: List[str]):
    data = BytesIO()
    size_y = 120
    size_x = 120
    data.write(struct.pack('II', *[size_y, size_x]))
    print(struct.pack('II', *[size_y, size_x]))
    for y in range(size_y):
        for x in range(size_x):
            capacity = 0.71
            moisture = 0.0
            if y < len(user_define) and x < len(user_define[y]):
                code = user_define[y][x]
                capacity = float(code) / 10.0
                moisture = float(code) / 10.0
            data.write(struct.pack('ffff', *[capacity, moisture, 0.0, 0.0]))
    data = data.getvalue()
    print('data length', len(data))
    connection = sqlite3.connect('../../assets/database.sqlite')
    connection.execute('update Land set map = ? where id = ?', [data, land_id])
    connection.commit()


def generate_platform(land_id: int, user_define: List[str]):
    data = BytesIO()
    size_y = 120
    size_x = 120
    data.write(struct.pack('II', *[size_y, size_x]))
    print(struct.pack('II', *[size_y, size_x]))
    for y in range(size_y):
        for x in range(size_x):
            wall = 0
            inner = 0
            door = 0
            window = 0
            if y < len(user_define) and x < len(user_define[y]):
                code = user_define[y][x]
                if code == '=':
                    wall = 1
                    inner = 1
                if code == '#':
                    wall = 1
                    inner = 0
                if code == '0':
                    wall = 1
                    inner = 0
                    door = 1
                if code == 'o':
                    wall = 1
                    inner = 0
                    window = 1
            data.write(struct.pack('BBBB', *[wall, inner, door, window]))
    data = data.getvalue()
    print('data length', len(data))
    connection = sqlite3.connect('../../assets/database.sqlite')
    connection.execute('update Platform set map = ? where id = ?', [data, land_id])
    connection.commit()


if __name__ == '__main__':
    # generate_land(1, [
    #     '0123456777777777777777012',
    #     '9999999977777777777777012',
    #     '1111111177777777777777012',
    #     '9876543277777777777777012'
    # ])
    generate_platform(1, [
        '......................',
        '......................',
        '..............########',
        '..............#...0..#',
        '..#=o=#.......#...#..#',
        '..o...==0=#...##0o#0##',
        '..#.......0...#...0..#',
        '..##0#o####...##0#####.',
        '......................',
        '........#ooooooo0#....',
        '........o........#....',
        '........o........#....',
        '........o..####..#....',
        '........o..#..#..#....',
        '........0..#..#..#....',
        '........#..##0#..#....',
        '........#........#....',
        '........##########....',
        '......................',
    ])