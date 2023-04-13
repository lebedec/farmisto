import json
import os
import sqlite3
import struct
from io import BytesIO
from sqlite3 import Connection
from typing import List, Dict, Tuple, Callable

from PIL import Image


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

    def add_stack(self, farmland: str, tile: [int, int], items: List[str], quantity: int):
        execute_script(
            self.connection,
            './add_stack.sql',
            position=as_sql_position(tile),
            items=f"'{json.dumps(items)}'",
            quantity=str(quantity),
            farmland=farmland
        )

    def add_equipment(self, farmland: str, tile: [int, int], kind_name: str):
        execute_script(
            self.connection,
            './add_equipment.sql',
            kind_name=f"'{kind_name}'",
            position=as_sql_position(tile),
            farmland=farmland
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
        return '1'  # TODO: select real id


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
        soil_data: bytes,
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
    farmland_id = editor.create_farmland(farmland_kind, holes_data, soil_data, grid_data)
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
            # data.write(struct.pack('=Bff', *[2, capacity, moisture]))
            data.write(struct.pack('=BB', *[int(capacity * 255), int(moisture * 255)]))
    return data.getvalue()


def generate_soil_from_image(path: str) -> bytes:
    image = Image.open(path).convert('L')
    data = BytesIO()
    size_y = 128
    size_x = 128
    data.write(struct.pack('B', size_y))
    for y in range(size_y):
        data.write(struct.pack('B', size_x))
        for x in range(size_x):
            capacity = 0.7 - (image.getpixel((x, y)) / 255) * 0.7
            moisture = 0.0
            # data.write(struct.pack('=Bff', *[2, capacity, moisture]))
            data.write(struct.pack('=BB', *[int(capacity * 255), int(moisture * 255)]))
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
    order = {
        'CementerKind': 1,
        'AssemblyKind': 2
    }
    tables = sorted(tables, key=lambda name: order.get(name, 0))

    for table in tables:
        move_table(table)


def as_sql_position(tile: Tuple[int, int]) -> str:
    x, y = tile
    return f"'[{x}.5, {y}.5]'"


def prototype_assembling():
    create_new_database('../../assets/database.sqlite', '../../assets/database_tmp.sqlite')
    editor = Editor(sqlite3.connect('../../assets/database.sqlite'))
    generate_farmland(
        editor,
        farmland_kind='test',
        soil_data=generate_soil_from_image("../bin/data/noise.png"),
        objects={
            'A': lambda tile, farmland: editor.add_farmer('farmer', 'Alice', farmland, tile),
            'B': lambda tile, farmland: editor.add_farmer('farmer', 'Boris', farmland, tile),
            'C': lambda tile, farmland: editor.add_farmer('farmer', 'Carol', farmland, tile),
            'D': lambda tile, farmland: editor.add_farmer('farmer', 'David', farmland, tile),
            'c': lambda tile, farmland: editor.add_stack(farmland, tile, ['concrete-material'] * 5, 1),
            'w': lambda tile, farmland: editor.add_stack(farmland, tile, ['wood-material'] * 5, 1),
            'p': lambda tile, farmland: editor.add_stack(farmland, tile, ['planks-material'] * 5, 1),
            'g': lambda tile, farmland: editor.add_stack(farmland, tile, ['glass-material'] * 5, 1),
            'd': lambda tile, farmland: editor.add_stack(farmland, tile, ['door-x1'] * 5, 1),
            'k': lambda tile, farmland: editor.add_stack(farmland, tile, ['cementer-kit'] * 5, 1),
            's': lambda tile, farmland: editor.add_stack(farmland, tile, ['stones'] * 5, 1),
            'h': lambda tile, farmland: editor.add_stack(farmland, tile, ['hammer'], 1),
            'n': lambda tile, farmland: editor.add_equipment(farmland, tile, 'theodolite'),
        },
        buildings={
            # (wall, door, window, material)
            '.': (0, 0, 0, Material.UNKNOWN),
            '#': (1, 0, 0, Material.CONCRETE),
            '|': (1, 1, 0, Material.CONCRETE),
            '-': (1, 0, 1, Material.CONCRETE),
            '+': (1, 0, 0, Material.PLANKS)
        },
        # . . . . . . . . 1 . . . . . . . . . 2 . . . . . . . . . 3 . . . . . . . . . 4
        user_define_map="""
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . c . . . . . . . . . . . . . c . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . c c . . . . . c c . . . . . . c . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . c c . . . . c c c . . . . c c c . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . k . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . s d . . . . . . . n h . . . . . n h . . . . . n h . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . # # # | # # # . . . . . . . B C D . . k . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . | . . . . . # | | # . . . . . A . . s d . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . # . . . . . . . . # . . . . . . . . s s . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . # | | | # # # . . | . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . | . . | . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . | . . | . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . # # # # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . p p c c . . . . c c p . . . . . c c . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . p . . . . . p p . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . n . n . n . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . . . . . n . . . . . . . . . . . . . . . . . . . . . . . . . w w w . w w w . . . . . . . . . . . . . . . .
        . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . . c c c . w w w . w w w . w w w . . . . . . . . . . . . . . . .
        . . . . # # # # # # # . . . . . . . . . . . . . . . . . . . . . c c c . w w w . w w w . w w w . . . . . . . . . . . . . . . .
        . . . . . . . . . . # . . . . . . . . . . . . . . . . . . . . . c c c . w w w . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . # . . . . . . . . . . . . . . . . . . . . p p p p . . c c c c . c c c c . c c c c c c . . . . . . . . . .
        . . . . . . . . . . # . . . . . . . . . . . . . . . . . . . . p p p p . . c c c c . c c c c . c c c c c c . . . . . . . . . .
        . . . . . . . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . c c c c . c c c c . c c c c . . . . . . . . . . . .
        . . . . . . . . . . # . . . . . . . . . . . . . . . . . . . . p p p p . . c c c c . c c c c . c c c c . . . . . . . . . . . .
        . . . . . . . . . . # . . . . . . . . . . . n . . . . . . . . p p p p . . . . . . . . . . . . c c c c . . . . . . . . . . . .
        . . . . # # # # # # # . . . . . . . . . . . . . . . . . . . . . . . . . . g g . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . p p p p p . g g g g g g . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . p p p p p . g g g g g g . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . p p p . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # # | # # # # # # # # # # # # # # # # # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . # . . . . . . # . . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . # . . . . . . # . . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . # . . . . . . # . . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # # - # - # # . . . . . . # # # # # # # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        """
    )
    
def prototype_building():
    create_new_database('../../assets/database.sqlite', '../../assets/database_tmp.sqlite')
    editor = Editor(sqlite3.connect('../../assets/database.sqlite'))
    generate_farmland(
        editor,
        farmland_kind='test',
        # soil_data=generate_soil([
        #     '7777777777777777777777777',
        #     '0123456777777777777777012',
        #     '9999999977777777777777012',
        #     '1111111177777777777777012',
        #     '9876543277777777777777012'
        # ]),
        soil_data=generate_soil_from_image("../bin/data/noise.png"),
        objects={
            'A': lambda tile, farmland: editor.add_farmer('farmer', 'Alice', farmland, tile),
            'B': lambda tile, farmland: editor.add_farmer('farmer', 'Boris', farmland, tile),
            'C': lambda tile, farmland: editor.add_farmer('farmer', 'Carol', farmland, tile),
            'D': lambda tile, farmland: editor.add_farmer('farmer', 'David', farmland, tile),
            'c': lambda tile, farmland: editor.add_stack(farmland, tile, ['concrete-material'] * 5, 1),
            'w': lambda tile, farmland: editor.add_stack(farmland, tile, ['wood-material'] * 5, 1),
            'p': lambda tile, farmland: editor.add_stack(farmland, tile, ['planks-material'] * 5, 1),
            'g': lambda tile, farmland: editor.add_stack(farmland, tile, ['glass-material'] * 5, 1),
            'd': lambda tile, farmland: editor.add_stack(farmland, tile, ['door-x1'] * 5, 1),
            'k': lambda tile, farmland: editor.add_stack(farmland, tile, ['cementer-kit'] * 5, 1),
            'h': lambda tile, farmland: editor.add_stack(farmland, tile, ['hammer'], 1),
            'n': lambda tile, farmland: editor.add_equipment(farmland, tile, 'theodolite'),
        },
        buildings={
            # (wall, door, window, material)
            '.': (0, 0, 0, Material.UNKNOWN),
            '#': (1, 0, 0, Material.CONCRETE),
            '|': (1, 1, 0, Material.CONCRETE),
            '-': (1, 0, 1, Material.CONCRETE),
            '+': (1, 0, 0, Material.PLANKS)
        },
        # . . . . . . . . 1 . . . . . . . . . 2 . . . . . . . . . 3 . . . . . . . . . 4
        user_define_map="""
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . c . . . . . . . . . . . . . c . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . c c . . . . . c c . . . . . . c . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . B C D . . . . . . . . . c c . . . . c c c . . . . c c c . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . A . . . . . . . . . . n h . . . . . n h . . . . . n h . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . p p c c . . . . c c p . . . . . c c . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . p . . . . . p p . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . n . n . n . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . . . . . n . . . . . . . . . . . . . . . . . . . . . . . . . w w w . w w w . . . . . . . . . . . . . . . .
        . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . . c c c . w w w . w w w . w w w . . . . . . . . . . . . . . . .
        . . . . # # # # # # # . . . . . . . . . . . . . . . . . . . . . c c c . w w w . w w w . w w w . . . . . . . . . . . . . . . .
        . . . . . . . . . . # . . . . . . . . . . . . . . . . . . . . . c c c . w w w . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . # . . . . . . . . . . . . . . . . . . . . p p p p . . c c c c . c c c c . c c c c c c . . . . . . . . . .
        . . . . . . . . . . # . . . . . . . . . . . . . . . . . . . . p p p p . . c c c c . c c c c . c c c c c c . . . . . . . . . .
        . . . . . . . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . c c c c . c c c c . c c c c . . . . . . . . . . . .
        . . . . . . . . . . # . . . . . . . . . . . . . . . . . . . . p p p p . . c c c c . c c c c . c c c c . . . . . . . . . . . .
        . . . . . . . . . . # . . . . . . . . . . . n . . . . . . . . p p p p . . . . . . . . . . . . c c c c . . . . . . . . . . . .
        . . . . # # # # # # # . . . . . . . . . . . . . . . . . . . . . . . . . . g g . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . p p p p p . g g g g g g . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . p p p p p . g g g g g g . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . p p p . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # # | # # # # # # # # # # # # # # # # # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . # . . . . . . # . . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . # . . . . . . # . . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # . . . . . # . . . . . . # . . . . . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . # # - # - # # . . . . . . # # # # # # # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        """
    )

if __name__ == '__main__':
    prototype_assembling()