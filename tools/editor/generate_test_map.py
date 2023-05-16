import json
import os
import shutil
import sqlite3
import struct
from io import BytesIO
from sqlite3 import Connection
from typing import List, Dict, Tuple, Callable, Union

from PIL import Image


class Editor:

    def __init__(self, connection: Connection):
        self.connection = connection
        self.connection.execute("PRAGMA foreign_keys = ON")

    def add_farmer(self, kind_name: str, player: str, space: str, tile: Tuple[int, int]):
        execute_script(
            self.connection,
            './add_farmer.sql',
            kind_name=f"'{kind_name}'",
            space=space,
            position=as_sql_position(tile),
            player=f"'{player}'"
        )

    def add_creature(self, kind_name: str, space: str, tile: Tuple[int, int]):
        execute_script(
            self.connection,
            './add_creature.sql',
            kind_name=f"'{kind_name}'",
            space=space,
            position=as_sql_position(tile),
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

    def add_door(self, farmland: str, tile: [int, int], kind_name: str):
        execute_script(
            self.connection,
            './add_door.sql',
            kind_name=f"'{kind_name}'",
            position=as_sql_position(tile),
            pivot=as_sql_tile(tile),
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

    def add_cementer(self, farmland: str, tile: [int, int], kind_name: str):
        execute_script(
            self.connection,
            './add_cementer.sql',
            kind_name=f"'{kind_name}'",
            position=as_sql_position(tile),
            pivot=as_sql_tile(tile),
            farmland=farmland
        )

    def add_composter(self, farmland: str, tile: [int, int], kind_name: str):
        execute_script(
            self.connection,
            './add_composter.sql',
            kind_name=f"'{kind_name}'",
            position=as_sql_position(tile),
            pivot=as_sql_tile(tile),
            farmland=farmland
        )

    def add_rest(self, farmland: str, tile: [int, int], kind_name: str):
        execute_script(
            self.connection,
            './add_rest.sql',
            kind_name=f"'{kind_name}'",
            position=as_sql_position(tile),
            pivot=as_sql_tile(tile),
            farmland=farmland
        )

    def add_crop(self, farmland: str, tile: [int, int],
                 kind_name: str, growth: float, health: float = 1.0, hunger: float = 0.0, thirst: float = 0.0):
        execute_script(
            self.connection,
            './add_crop.sql',
            kind_name=f"'{kind_name}'",
            position=as_sql_position(tile),
            farmland=farmland,
            growth=str(growth),
            health=str(health),
            hunger=str(hunger),
            thirst=str(thirst)
        )

    def create_farmland(
            self,
            kind: str,
            holes: bytes,
            moisture: bytes,
            moisture_capacity: bytes,
            surface: bytes,
            fertility: bytes,
            grid: bytes
    ) -> str:
        connection = self.connection
        print(
            'Create farmland', kind,
            'holes:', len(holes),
            'moisture:', len(moisture),
            'moisture_capacity:', len(moisture_capacity),
            'surface:', len(surface),
            'fertility:', len(fertility),
            'grid:', len(grid)
        )

        kind = connection.execute(
            'select id, space, soil, grid, land, calendar from FarmlandKind where name = ?',
            [kind]
        ).fetchone()
        kind, space_name, soil_name, grid_name, land_name, calendar_name = kind

        space_kind = '(select id from SpaceKind where name = ?)'
        connection.execute(
            f'insert into Space values (null, {space_kind}, ?)',
            [space_name, holes]
        )
        space_id = '(select max(id) from Space)'

        soil_kind = '(select id from SoilKind where name = ?)'
        connection.execute(
            f'insert into Soil values (null, {soil_kind}, ?)',
            [soil_name, fertility]
        )
        soil_id = '(select max(id) from Soil)'

        grid_kind = '(select id from GridKind where name = ?)'
        connection.execute(
            f'insert into Grid values (null, {grid_kind}, ?)',
            [grid_name, grid]
        )
        grid_id = '(select max(id) from Grid)'

        land_kind = '(select id from LandKind where name = ?)'
        connection.execute(
            f'insert into Land values (null, {land_kind}, ?, ?, ?)',
            [land_name, moisture, moisture_capacity, surface]
        )
        land_id = '(select max(id) from Land)'

        calendar_kind = '(select id from CalendarKind where name = ?)'
        connection.execute(
            f'insert into Calendar values (null, {calendar_kind}, 0, 0.0, 0.25)',
            [calendar_name]
        )
        calendar_id = '(select max(id) from Calendar)'

        connection.execute(
            f'insert into Farmland values (null, ?, {space_id}, {soil_id}, {grid_id}, {land_id}, {calendar_id})',
            [kind])
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


def pack_bincode_length():
    return struct.pack('BBB', *[251, 0, 64])


def generate_farmland(
        editor: Editor,
        farmland_kind: str,
        moisture_data: bytes,
        moisture_capacity_data: bytes,
        fertility_data: bytes,
        objects: Dict[str, Callable[[Tuple[int, int], str], None]],
        buildings: Dict[str, Tuple[int, int, int, int]],
        surfaces: Dict[str, int],
        user_define_map: str
):
    user_define = []
    for line in user_define_map.splitlines(keepends=False):
        line = line.strip().replace(' ', '')
        if line:
            user_define.append(line)
    surface_data = BytesIO()
    grid_data = BytesIO()
    holes_data = BytesIO()
    size_y = 128
    size_x = 128
    grid_data.write(struct.pack('B', size_y))
    holes_data.write(struct.pack('B', size_y))
    surface_data.write(pack_bincode_length())
    edits = []
    for y in range(size_y):
        grid_data.write(struct.pack('B', size_x))
        holes_data.write(struct.pack('B', size_x))
        for x in range(size_x):
            surface = 0
            wall, door, window, material = 0, 0, 0, 0
            is_hole = 0
            if y < len(user_define) and x < len(user_define[y]):
                code = user_define[y][x]
                if code in objects:
                    tile = (x, y)
                    edits.append((tile, objects[code]))

                if code in surfaces:
                    surface = surfaces[code]
                    is_hole = 1
                elif code in buildings:
                    wall, door, window, material = buildings[code]
                    is_hole = 1 if wall and not door else 0

            grid_data.write(struct.pack('BBBB', *[wall, door, window, material]))
            holes_data.write(struct.pack('B', is_hole))
            surface_data.write(struct.pack('B', surface))

    farmland_id = editor.create_farmland(
        farmland_kind,
        holes_data.getvalue(),
        moisture_data,
        moisture_capacity_data,
        surface_data.getvalue(),
        fertility_data,
        grid_data.getvalue()
    )
    for tile, edit in edits:
        edit(tile, farmland_id)


def generate_feature_map(feature: Callable[[Tuple[int, int]], Union[int, float]]) -> bytes:
    data = BytesIO()
    size_y = 128
    size_x = 128
    data.write(pack_bincode_length())
    for y in range(size_y):
        for x in range(size_x):
            value = feature((x, y))
            if isinstance(value, int):
                data.write(struct.pack('B', value))
            else:
                data.write(struct.pack('f', value))
    return data.getvalue()


def moisture_capacity_from_image(path: str, max_value=0.7) -> bytes:
    image = Image.open(path).convert('L')
    data = BytesIO()
    size_y = 128
    size_x = 128
    data.write(pack_bincode_length())
    for y in range(size_y):
        for x in range(size_x):
            capacity = max_value - (image.getpixel((x, y)) / 255) * max_value
            data.write(struct.pack('f', capacity))
    return data.getvalue()



def execute_script(connection: Connection, script_path: str, **params):
    script = open(script_path, 'r').read()
    for key, value in params.items():
        script = script.replace(f':{key}', value)
    connection.executescript(script)


def copy_database(src_path: str, dst_path: str):
    if os.path.exists(dst_path):
        os.remove(dst_path)

    shutil.copy(src_path, dst_path)


def create_new_database(src_path: str, dst_path: str):
    if os.path.exists(dst_path):
        os.remove(dst_path)

    src = sqlite3.connect(src_path)
    dst = sqlite3.connect(dst_path)

    def move_table(table: str):
        rows = src.execute(f'select * from {table}')
        statement = None
        for row in rows:
            if statement is None:
                params = ', '.join(['?'] * len(row))
                statement = f'insert into {table} values ({params})'
            dst.execute(statement, row)
        dst.commit()

    migrations = open('../../database/database.sql').read()
    dst.executescript(migrations)

    rows = src.execute("select tbl_name from main.sqlite_master where type = 'table'")
    tables = [name for columns in rows for name in columns if name.endswith('Kind')]
    order = {
        'FarmlandKind': 1,
        'CropKind': 1,

        'DoorKind': 1,
        'RestKind': 1,
        'EquipmentKind': 1,
        'CementerKind': 1,
        'AssemblyKind': 2,

        'CorpseKind': 1,
        'CreatureKind': 2,
    }
    tables = sorted(tables, key=lambda name: order.get(name, 0))

    for table in tables:
        move_table(table)


def as_sql_tile(tile: Tuple[int, int]) -> str:
    x, y = tile
    return f"'[{x}, {y}]'"


def as_sql_position(tile: Tuple[int, int]) -> str:
    x, y = tile
    return f"'[{x}.5, {y}.5]'"


def prototype_raising(destination_path: str, template_path: str):
    create_new_database(template_path, destination_path)
    editor = Editor(sqlite3.connect(destination_path))
    generate_farmland(
        editor,
        farmland_kind='test',
        moisture_data=generate_feature_map(lambda _: 0.0),
        moisture_capacity_data=moisture_capacity_from_image(
            "../bin/data/noise.png",
            max_value=0.7
        ),
        fertility_data=generate_feature_map(lambda tile: 0.0),
        objects={
            'L': lambda tile, farmland: editor.add_creature('lama', farmland, tile),
            's': lambda tile, farmland: editor.add_stack(farmland, tile, ['shovel'], 1),
            'b': lambda tile, farmland: editor.add_rest(farmland, tile, 'bed'),
            'e': lambda tile, farmland: editor.add_stack(farmland, tile, ['seeds'], 10),
            'w': lambda tile, farmland: editor.add_stack(farmland, tile, ['watering-can'], 1),
            'f': lambda tile, farmland: editor.add_stack(farmland, tile, ['fertilizer'], 10),
            '1': lambda tile, farmland: editor.add_crop(farmland, tile, 'corn', 1.0),
            '2': lambda tile, farmland: editor.add_crop(farmland, tile, 'corn', 2.0),
            '3': lambda tile, farmland: editor.add_crop(farmland, tile, 'corn', 3.0),
            '4': lambda tile, farmland: editor.add_crop(farmland, tile, 'corn', 4.0),
            '8': lambda tile, f: editor.add_crop(f, tile, 'corn', 1.3, health=0.75, hunger=1.0, thirst=0.5),
            '9': lambda tile, f: editor.add_crop(f, tile, 'corn', 1.9, health=0.5, hunger=1.0, thirst=0.5),
            'k': lambda tile, farmland: editor.add_composter(farmland, tile, 'composter'),
            'r': lambda tile, farmland: editor.add_stack(farmland, tile, ['residue'], 10),
            'd': lambda tile, farmland: editor.add_door(farmland, tile, 'door-x3-planks'),
            't': lambda tile, farmland: editor.add_stack(farmland, tile, ['tether'], 1),
            'p': lambda tile, farmland: editor.add_stack(farmland, tile, ['peg'], 1),
        },
        buildings={
            #?': (w, d, window, material)
            '.': (0, 0, 0, Material.UNKNOWN),
            '#': (1, 0, 0, Material.CONCRETE),
            '|': (1, 1, 0, Material.CONCRETE),
            '-': (1, 0, 1, Material.CONCRETE),
            '+': (1, 0, 0, Material.PLANKS),
            '=': (1, 1, 0, Material.PLANKS),
            'd': (1, 1, 0, Material.PLANKS),
        },
        surfaces={
            '~': 1
        },
        # . . . . . . . . 1 . . . . . . . . . 2 . . . . . . . . . 3 . . . . . . . . . 4
        user_define_map="""
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . r . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . r r . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . p p t t . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . p t . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . + + + = d = + + + . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . + . . . . . . . + . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . b . . . + . . . L . . . + . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . + . . . . . . . + . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . + . . . . . . . + . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . + + + + + + + + + . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        """
    )

def prototype_planting(destination_path: str, template_path: str):
    create_new_database(template_path, destination_path)
    editor = Editor(sqlite3.connect(destination_path))
    generate_farmland(
        editor,
        farmland_kind='test',
        moisture_data=generate_feature_map(lambda _: 0.0),
        moisture_capacity_data=moisture_capacity_from_image(
            "../bin/data/prototype-planting-moisture-capacity.png",
            max_value=1.0
        ),
        fertility_data=generate_feature_map(lambda tile: 1.0 if 24 <= tile[0] < 32 and 5 < tile[1] < 14 else 0.0),
        objects={
            's': lambda tile, farmland: editor.add_stack(farmland, tile, ['shovel'], 1),
            'r': lambda tile, farmland: editor.add_rest(farmland, tile, 'bed'),
            'e': lambda tile, farmland: editor.add_stack(farmland, tile, ['seeds'], 10),
            'w': lambda tile, farmland: editor.add_stack(farmland, tile, ['watering-can'], 1),
            'f': lambda tile, farmland: editor.add_stack(farmland, tile, ['fertilizer'], 10),
            '1': lambda tile, farmland: editor.add_crop(farmland, tile, 'corn', 1.0),
            '2': lambda tile, farmland: editor.add_crop(farmland, tile, 'corn', 2.0),
            '3': lambda tile, farmland: editor.add_crop(farmland, tile, 'corn', 3.0),
            '4': lambda tile, farmland: editor.add_crop(farmland, tile, 'corn', 4.0),
            '8': lambda tile, f: editor.add_crop(f, tile, 'corn', 1.3, health=0.75, hunger=1.0, thirst=0.5),
            '9': lambda tile, f: editor.add_crop(f, tile, 'corn', 1.9, health=0.5, hunger=1.0, thirst=0.5),
            'k': lambda tile, farmland: editor.add_composter(farmland, tile, 'composter'),
        },
        buildings={
            # (wall, door, window, material)
            '.': (0, 0, 0, Material.UNKNOWN),
            '#': (1, 0, 0, Material.CONCRETE),
            '|': (1, 1, 0, Material.CONCRETE),
            '-': (1, 0, 1, Material.CONCRETE),
            '+': (1, 0, 0, Material.PLANKS)
        },
        surfaces={
            '~': 1
        },
        # . . . . . . . . 1 . . . . . . . . . 2 . . . . . . . . . 3 . . . . . . . . . 4
        user_define_map="""
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # # | # - # # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . . . r . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . w e . . . . . . . . . # . . . . . - . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . w e . . . . . . . . . | . . . r . # . . . . . . . . . . . . . . . . . . . s . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . w e . . . . . . . . . # . . . . . - . . . . . . . . . . . . . . . . . . . s s s . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . w e . . . . . . . . . # . . . r . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 9 . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # # | # - # # . . . . . . . . . . . . . # # - # | # # . . . . . . . . 8 . . 8 . . 9 . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . r . . . # . . . . . . . . . 9 . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . - . . . . . # . . . . . . . . . 8 . ~ ~ ~ ~ . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . r . . . | . . . . . . . . . . . ~ ~ ~ ~ . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . - . . . . . # . . . . ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . r . . . # . . . . ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ ~ . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # # - # | # # . . . . . . . . . . . ~ ~ ~ ~ ~ ~ . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . k . . . . . . . . . . . . . . . ~ ~ ~ ~ ~ ~ . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . f f . e e . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . e e . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . f . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . f f . . . # # # # # # # # | # - # # w w . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . k . # e . e e e # . . . r . # w w . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . k . # e e e e e # . . . . . - . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . k . # . e . . . | . . . r . # . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . k . # e . . . e # . . . . . - . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . # e . . . e # . . . r . # s s . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . # # | | | # # # | # - # # s s . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 9 . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 8 . . . . . . 9 . . 9 . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 8 . . . . . 8 . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 8 9 . . . . 9 . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . 8 . . . . 8 8 . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        """
    )


def prototype_assembling(destination_path: str, template_path: str):
    create_new_database(template_path, destination_path)
    editor = Editor(sqlite3.connect(destination_path))
    generate_farmland(
        editor,
        farmland_kind='test',
        moisture_data=generate_feature_map(lambda _: 0.0),
        moisture_capacity_data=moisture_capacity_from_image("../bin/data/noise.png"),
        fertility_data=generate_feature_map(lambda _: 0.0),
        objects={
            'c': lambda tile, farmland: editor.add_stack(farmland, tile, ['concrete-material'] * 5, 1),
            'w': lambda tile, farmland: editor.add_stack(farmland, tile, ['wood-material'] * 5, 1),
            'p': lambda tile, farmland: editor.add_stack(farmland, tile, ['planks-material'] * 5, 1),
            'g': lambda tile, farmland: editor.add_stack(farmland, tile, ['glass-material'] * 5, 1),
            'd': lambda tile, farmland: editor.add_stack(farmland, tile, ['door-x1'] * 5, 1),
            'b': lambda tile, farmland: editor.add_stack(farmland, tile, ['door-x3'] * 5, 1),
            'k': lambda tile, farmland: editor.add_stack(farmland, tile, ['cementer-kit'] * 5, 1),
            'z': lambda tile, farmland: editor.add_cementer(farmland, tile, 'cementer'),
            'r': lambda tile, farmland: editor.add_stack(farmland, tile, ['stones'] * 5, 1),
            's': lambda tile, farmland: editor.add_stack(farmland, tile, ['shovel'], 1),
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
        surfaces={},
        # . . . . . . . . 1 . . . . . . . . . 2 . . . . . . . . . 3 . . . . . . . . . 4
        user_define_map="""
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
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . # | # - # | # . . . . . . . . . . . . . . . . . . . . # # # # # # | | | # # # # . . . . . . . . . . . . . . . . .
        . . . . . . # . . . . . # . . . . . . . . . . . . . . . . . . . . # . s h s h . . . . . . # . . . . . . . . . . . . . . . . .
        . . . . . . | . . . . . | . . . . . . . . . . . . . . . . . . . . | . . . . . . . . . . . | . . . . . . . . . . . . . . . . .
        . . . . . . # . . . . . # . . . . . . . . . . . . . . . . . . . . # . k . . . . . . . . . | . . . . . . . . . . . . . . . . .
        . . . . . . # | # - # | # . . . . . . . . . . . . . . . . . . . . # # # # | | # . . . . . | . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # . z . . . # . . . . . # . . . . . . . . . . . . . . . . .
        . . . . . . . . . . d d . . . . . . . . . . . . . . . . . . . . . # . . . . . # . . . . k # . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . | . . . . . | . . . . . # . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . r # . . . . . # . . . . . # . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . # # - - - # # # | | | # # . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . d b . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
    
        """
    )


def prototype_building(destination_path: str, template_path: str):
    create_new_database(template_path, destination_path)
    editor = Editor(sqlite3.connect(destination_path))
    generate_farmland(
        editor,
        farmland_kind='test',
        moisture_data=generate_feature_map(lambda _: 0.5),
        moisture_capacity_data=moisture_capacity_from_image("../bin/data/noise.png"),
        fertility_data=generate_feature_map(lambda _: 0.0),
        objects={
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
        surfaces={},
        # . . . . . . . . 1 . . . . . . . . . 2 . . . . . . . . . 3 . . . . . . . . . 4
        user_define_map="""
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . c . . . . . . . . . . . . . c . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . c c . . . . . c c . . . . . . c . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . c c . . . . c c c . . . . c c c . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
        . . . . . . . . . . . . . . n h . . . . . n h . . . . . n h . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . . .
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
    template = '../../assets/database.sqlite'
    prototype_raising('../../assets/saves/prototype-raising.sqlite', template)
    prototype_planting('../../assets/saves/prototype-planting.sqlite', template)
    prototype_assembling('../../assets/saves/prototype-assembling.sqlite', template)
    prototype_building('../../assets/saves/prototype-building.sqlite', template)

    copy_database('../../assets/saves/prototype-raising.sqlite', template)
