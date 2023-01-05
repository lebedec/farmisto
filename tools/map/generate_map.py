import sqlite3
import struct
from io import BytesIO
from typing import List


def generate_map(land_id: int, user_define: List[str]):
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


if __name__ == '__main__':
    generate_map(1, [
        '0123456777777777777777012',
        '9999999977777777777777012',
        '1111111177777777777777012',
        '9876543277777777777777012'
    ])