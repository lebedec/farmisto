import json
import os
import sqlite3
import struct
from io import BytesIO
from sqlite3 import Connection
from typing import List, Dict, Tuple, Callable


class Editor:

    def __init__(self, connection: Connection):
        self.connection = connection

    def add_farmer(self, kind_name: str, player: str, space: str, tile: Tuple[int, int]):
        execute_script(
            self.connection,
            './add_farmer.sql',
            kind_name=f"'{kind_name}'",
            space=space,
            position=as_sql_position(tile),
            player=f"'{player}'"
        )

    def create_farmland(self, kind: str, holes: bytes, soil: bytes, grid: bytes) -> str:
        connection = self.connection
        print('Create farmland', kind, 'holes:', len(holes), 'soil:', len(soil), 'grid:', len(grid))

        kind = connection.execute('select id, space, soil, grid from FarmlandKind where name = ?', [kind]).fetchone()
        kind, space_kind, soil_kind, grid_kind = kind
        connection.execute('insert into Space values (null, ?, ?)', [space_kind, holes])
        space_id = '(select max(id) from Space)'
        connection.execute('insert into Soil values (null, ?, ?)', [soil_kind, soil])
        soil_id = '(select max(id) from Soil)'
        connection.execute('insert into Grid values (null, ?, ?)', [grid_kind, grid])
        grid_id = '(select max(id) from Grid)'
        connection.execute(f'insert into Farmland values (null, ?, {space_id}, {soil_id}, {grid_id})', [kind])
        connection.commit()

class Material:
    UNKNOWN = 0
    METAL = 10
    MESH = 15
    CONCRETE = 20
    WOOD = 30
    PLANKS = 35
    GLASS = 40
    TARPAULIN = 50

def generate_farmland(
        editor: Editor,
        farmland_kind: str,
        soil_define_map: List[str],
        objects: Dict[str, Callable[[Tuple[int, int], str], None]],
        buildings: Dict[str, Tuple[int, int, int, int]],
        user_define_map: str
):
    user_define = []
    for line in user_define_map.splitlines(keepends=False):
        line = line.strip().replace(' ', '')
        if line:
            user_define.append(line)
    grid_data = BytesIO()
    holes_data = BytesIO()
    size_y = 128
    size_x = 128
    grid_data.write(struct.pack('B', size_y))
    holes_data.write(struct.pack('B', size_y))
    edits = []
    for y in range(size_y):
        grid_data.write(struct.pack('B', size_x))
        holes_data.write(struct.pack('B', size_x))
        for x in range(size_x):
            if y < len(user_define) and x < len(user_define[y]):
                code = user_define[y][x]
                if code in objects:
                    tile = (x, y)
                    edits.append((tile, objects[code]))
                    wall, door, window, material = 0, 0, 0, 0
                    is_hole = 0
                else:
                    wall, door, window, material = buildings[code]
                    is_hole = 1 if wall and not door else 0
            else:
                wall, door, window, material = 0, 0, 0, 0
                is_hole = 0
            grid_data.write(struct.pack('BBBB', *[wall, door, window, material]))
            holes_data.write(struct.pack('B', is_hole))

    grid_data = grid_data.getvalue()
    holes_data = holes_data.getvalue()
    soil_data = generate_soil(soil_define_map)
    editor.create_farmland(farmland_kind, holes_data, soil_data, grid_data)
    farmland_id = '1' # TODO: select real id
    for tile, edit in edits:
        edit(tile, farmland_id)


def generate_soil(user_define: List[str]) -> bytes:
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
    return data.getvalue()


def execute_script(connection: Connection, script_path: str, **params):
    script = open(script_path, 'r').read()
    for key, value in params.items():
        script = script.replace(f':{key}', value)
    connection.executescript(script)


def create_new_database(dst_path: str, tmp_path: str):
    if os.path.exists(tmp_path):
        os.remove(tmp_path)

    os.rename(dst_path, tmp_path)

    src = sqlite3.connect(tmp_path)
    dst = sqlite3.connect(dst_path)

    def move_table(table: str):
        rows = src.execute(f'select * from {table}')
        statement = None
        for row in rows:
            if statement is None:
                params = ', '.join(['?'] * len(row))
                statement = f'insert into {table} values ({params})'
                print('MOVE', statement)
            dst.execute(statement, row)
        dst.commit()

    migrations = open('../../database/database.sql').read()
    dst.executescript(migrations)

    rows = src.execute("select tbl_name from main.sqlite_master where type = 'table'")
    tables = [name for columns in rows for name in columns if name.endswith('Kind')]
    root_tables = []
    for table in tables:
        try:
            move_table(table)
        except sqlite3.IntegrityError:
            # HACK: move aggregates after domain kinds
            # TODO: determine aggregates and handle properly
            print('FAILED try again later')
            root_tables.append(table)

    for table in root_tables:
        move_table(table)



def as_sql_position(tile: Tuple[int, int]) -> str:
    x, y = tile
    return f"'[{x}.5, {y}.5]'"


if __name__ == '__main__':
    create_new_database('../../assets/database.sqlite', '../../assets/database_tmp.sqlite')
    editor = Editor(sqlite3.connect('../../assets/database.sqlite'))
    generate_farmland(
        editor,
        farmland_kind='test',
        soil_define_map=[
            '7777777777777777777777777',
            '0123456777777777777777012',
            '9999999977777777777777012',
            '1111111177777777777777012',
            '9876543277777777777777012'
        ],
        objects={
            'A': lambda tile, space: editor.add_farmer('farmer', 'Alice', space, tile),
            'B': lambda tile, space: editor.add_farmer('farmer', 'Boris', space, tile),
            'C': lambda tile, space: editor.add_farmer('farmer', 'Carol', space, tile),
            'D': lambda tile, space: editor.add_farmer('farmer', 'David', space, tile),
        },
        buildings={
            # (wall, door, window, material)
            '.': (0, 0, 0, Material.UNKNOWN),
            '#': (1, 0, 0, Material.CONCRETE),
            '|': (1, 1, 0, Material.CONCRETE),
            '-': (1, 0, 1, Material.CONCRETE),
            '+': (1, 0, 0, Material.PLANKS)
        },
        user_define_map="""
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . A . . . . . # # # # | # - - - # # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . B C D . . # . # . . . . . . . - . . . . . . . . . . . . . . # # | | | # # . . . . . . # # # # # # . . . . . . . . . . .
        . . . . . . . . # . # . . # # # # . # . . . . . . . . . . . . . . # . . . . . # . . . . . . # . . . . # # # # # . . . . . . .
        . . . . . . . . - . . . . # . . # # # . . . . . . . . . . . . . . | . # # # . | . # # # . . # # # . . # # . . # . . . . . . .
        . . . . . . . . # . . . . # . . . . . . # # # . . . . . . . . . . | . # . # . | . . . # . . . . # . . # # . . # . . . . . . .
        . . . . . . . . # - # | # # . . . # | # # . # # . . . . . . . . . # . # . # . | . . . # . . . . # . . # # # # # . . . . . . .
        . . . . . . . . . . . . . . . . . # . . . . . # . . . . . . . . . # . . . . . # . # . # . . . . # . . # # . . . . . . . . . .
        . . . . . . . + + + + + + . . . . # . # # # . # # . . . . . . . . # | | # - - # . # # # . . . . # . . # # . . . . . . . . . .
        . . . . . . . + . . . . + . . # - # . # . # . . # . . . . . . . . . . . . . . . . . . . . . . . # . . # # # # # # # . . . . .
        . . . . . . . + . . . . + . . # . . . # # # . # # . . . . . . . # - # . # # | # - # . . . . . . # . . # . . # . . # . . . . .
        . . . . . . . + + + + + + . . # - # . . . . . # . . . . . . . . # . # . # . . . . # . . . . . . # . . # . . # . . # . . . . .
        . . . . . . . . . . . . . . . . . # . # | # # # . . . . . . . . - . # | # . . # # # . . . . . . # . . # # # # # # # . . . . .
        . . . . . . . . . . . . . . . . . # # # . . . . . . . . . . . . # . . . . . . # . . . . . . . . # . . # . . . . . # . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # # . # # # . # . # # # # # # . # . . # . . . . . # . . . . .
        . . . . . . . . . . . . . . . . . . . . # . . . . . . . . . . . . # . # . # . # . . . . . . # . # . . # # # # # # # . . . . .
        . . . . . . . . . . . . . . . . . . . . # . . . . . . . . . . . . # . # # # # # . # # # # . # . # . . # . . . . . . . . . . .
        # # # # # # # # # # # # # # # # # # # # # . . . . . . . . . . . . # . . . . . # . # . . # . # . # . . # . . . . . . . . . . .
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
