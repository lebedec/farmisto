import sqlite3
import struct
from io import BytesIO
from typing import List


def generate_land(land_id: int, user_define: List[str]):
    data = BytesIO()
    size_y = 128
    size_x = 128
    data.write(struct.pack('B', size_y))
    for y in range(size_y):
        data.write(struct.pack('B', size_x))
        for x in range(size_x):
            capacity = 0.71
            moisture = 0.0
            if y < len(user_define) and x < len(user_define[y]):
                code = user_define[y][x]
                capacity = float(code) / 10.0
                moisture = float(code) / 10.0
            data.write(struct.pack('=Bff', *[2, capacity, moisture]))
    data = data.getvalue()
    print('data length', len(data))
    connection = sqlite3.connect('../../assets/database.sqlite')
    connection.execute('update Land set map = ? where id = ?', [data, land_id])
    connection.commit()


def generate_grid(land_id: int, user_define_map: str):
    user_define = []
    for line in user_define_map.splitlines(keepends=False):
        line = line.strip().replace(' ', '')
        if line:
            user_define.append(line)
    data = BytesIO()
    size_y = 128
    size_x = 128
    data.write(struct.pack('B', size_y))
    for y in range(size_y):
        data.write(struct.pack('B', size_x))
        for x in range(size_x):
            wall = 0
            inner = 0
            door = 0
            window = 0
            marker = 0
            material = 0
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
                if code == '+':
                    wall = 1
                    inner = 0
                    marker = 1
            data.write(struct.pack('BBBBBB', *[wall, inner, door, window, marker, material]))
    data = data.getvalue()
    print('data length', len(data))
    connection = sqlite3.connect('../../assets/database.sqlite')
    connection.execute('update Grid set map = ? where id = ?', [data, land_id])
    connection.commit()


if __name__ == '__main__':
    generate_land(1, [
        '7777777777777777777777777',
        '0123456777777777777777012',
        '9999999977777777777777012',
        '1111111177777777777777012',
        '9876543277777777777777012'
    ])

    # value = bytes([2, 0, 0, 0, 0, 0, 0, 0, 2, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1, 0, 1, 0, 0, 2, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 3, 0, 0, 0, 1, 4])
    value = bytes([2, 2, 2, 205, 204, 204, 61, 205, 204, 76, 62, 2, 154, 153, 153, 62, 205, 204, 204, 62, 2, 2, 205, 204, 204, 61, 205, 204, 76, 62, 2, 154, 153, 153, 62, 205, 204, 204, 62])

    data = BytesIO()
    data.write(struct.pack('B', 2))
    # data.write(struct.pack('B', 2))
    # data.write(struct.pack('BBBBB', *[True, False, False, False, 1]))
    # data.write(struct.pack('BBBBB', *[False, True, False, False, 2]))
    # data.write(struct.pack('B', 2))
    # data.write(struct.pack('BBBBB', *[False, False, True, False, 3]))
    # data.write(struct.pack('BBBBB', *[False, False, False, True, 4]))
    data.write(struct.pack('B', 2))
    data.write(struct.pack('=Bff', *[2, 0.1, 0.2]))
    data.write(struct.pack('=Bff', *[2, 0.3, 0.4]))
    data.write(struct.pack('B', 2))
    data.write(struct.pack('B', 2))
    data.write(struct.pack('ff', *[0.1, 0.2]))
    data.write(struct.pack('B', 2))
    data.write(struct.pack('ff', *[0.3, 0.4]))
    result = data.getvalue()

    print(len(value), value)
    print(len(result), result)
    generate_grid(
        1,
        """
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # # # # 0 # # . . . . . . # # # # # # . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . . . . . # . . . . . . # . . . . # # # # # . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . # # # . # . # # # . . # # # . . # # . . # . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 0 . # . # . 0 . . . # . . . . # . . # # . . # . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . # . # . # . . . # . . . . # . . # # # # # . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . . . . . # . # . # . . . . # . . # # . . . . . . . . . .
        . . . . . . . + + + + + + . . . . . . . . . . . . . . . . . . . . # # 0 # o # # . # # # . . . . # . . # # . . . . . . . . . .
        . . . . . . . + . . . . + . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . . # # # # # # # . . . . .
        . . . . . . . + . . . . + . . . . . . . . . . . . . . . . . . . # o # . # # 0 # o # . . . . . . # . . # . . # . . # . . . . .
        . . . . . . . + + + + + + . . . . . . . . . . . . . . . . . . . # . # . # . . . . # . . . . . . # . . # . . # . . # . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . o . # 0 # . . # # # . . . . . . # . . # # # # # # # . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . . . . . . # . . . . . . . . # . . # . . . . . # . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # # . # # # . # . # # # # # # . # . . # . . . . . # . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . # . # . # . . . . . . # . # . . # # # # # # # . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . # # # # # . # # # # . # . # . . # . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . . . . . # . # . . # . # . # . . # . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # # # # # # # . # # # # # # . # . . # # # # # # # # # . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . . . . . . . . . . # . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # # # # # # # # # # # . . . . . . . . . . # . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . . . . . . . . . . . . . # # # # # . . # . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . . . . . . . . . . . . . # . . . # . . # . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . # # # # # # # . . . . . # . . . # . . # . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . # . . . . . # . . . . . # . . . # . . # . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . # . # # # . # . . . . . # # # # # . . # . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . # . # . # . # . . . . . . . . . . . . # . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . # . # # # . # . . . . . . . . . . . . # . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . # . . . . . # . . # # # # # # # # # # # . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . # # # # # # # . . # . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . . . . . . . . . . # . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # # # # # # # # # # # # . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        """
    )
