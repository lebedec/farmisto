import ctypes
import json
from dataclasses import dataclass
from typing import List, Dict, Callable

from .types import Material, Position

Array2Float = ctypes.c_float * 2
Array2Int = ctypes.c_ulonglong * 2


def define(structure_type):
    fields = []
    for name, python_type in structure_type.__annotations__.items():
        c_type = ctypes.c_void_p

        if python_type == int:
            c_type = ctypes.c_ulonglong

        if python_type == List[int]:
            c_type = Array2Int

        fields.append((name, c_type))

    structure_type._fields_ = fields

    def as_json(self):
        data = {}
        for field in fields:
            name, *_ = field
            data[name] = getattr(self, name)
        return data

    structure_type.as_json = as_json

    return structure_type


class Entity(ctypes.Structure):
    def as_json(self) -> Dict:
        pass


@define
class Creature(Entity):
    id: int
    key: int
    body: int
    animal: int


@define
class Farmland(Entity):
    id: int
    kind: int
    space: int
    soil: int
    grid: int
    land: int
    calendar: int


@define
class Farmer(Entity):
    id: int
    kind: int
    player: int
    body: int
    hands: int
    backpack: int
    tether: int


@define
class Equipment(Entity):
    id: int
    key: int


@define
class Theodolite(Entity):
    id: int
    key: int
    surveyor: int
    barrier: int


@define
class Construction(Entity):
    id: int
    container: int
    grid: int
    surveyor: int
    stake: int


@define
class Stack(Entity):
    id: int
    container: int
    barrier: int


@define
class Crop(Entity):
    id: int
    key: int
    plant: int
    barrier: int
    sensor: int


@dataclass
class FarmerTestContext:
    player: str
    entity: Farmer
    position: Position
    actions: List[Callable]


class GameTestScenario:

    def __init__(self, lib):
        self._lib = lib
        self._scenario = None

    def create(self, database: str):
        self._scenario = self._lib.create(database.encode('utf-8'))

    def dispose(self):
        self._lib.dispose(self._scenario)

    def update(self, time: float):
        self._lib.update(self._scenario, ctypes.c_float(time))

    # game

    def add_farmland(self, kind: str) -> Farmland:
        self._lib.add_farmland.restype = Farmland
        return self._lib.add_farmland(self._scenario, kind.encode('utf-8'))

    def add_farmer(self, name: str, kind: str, farmland: Farmland, position: List[float]) -> Farmer:
        self._lib.add_farmer.restype = Farmer
        return self._lib.add_farmer(
            self._scenario,
            name.encode('utf-8'),
            kind.encode('utf-8'),
            farmland,
            Array2Float(*position)
        )

    def set_farmer_activity(self, farmer: Farmer, activity):
        return self._lib.set_farmer_activity(self._scenario, farmer, json.dumps(activity).encode('utf-8'))

    def add_theodolite(self, kind: str, farmland: Farmland, position: List[float]) -> Theodolite:
        self._lib.add_theodolite.restype = Theodolite
        return self._lib.add_theodolite(
            self._scenario,
            kind.encode('utf-8'),
            farmland,
            Array2Float(*position)
        )

    def add_crop(self, kind: str, farmland: Farmland, position: List[float]) -> Crop:
        self._lib.add_crop.restype = Crop
        return self._lib.add_crop(
            self._scenario,
            kind.encode('utf-8'),
            farmland,
            Array2Float(*position)
        )

    def add_construction(self, surveyor: int, grid: int, position: List[float]) -> Construction:
        self._lib.add_construction.restype = Construction
        return self._lib.add_construction(
            self._scenario,
            surveyor,
            grid,
            Array2Float(*position)
        )

    def add_deconstruction(self, surveyor: int, grid: int, position: List[float]) -> Construction:
        self._lib.add_construction.restype = Construction
        return self._lib.add_construction(
            self._scenario,
            surveyor,
            grid,
            Array2Float(*position)
        )

    def add_items(self, kind: str, count: int, container: int):
        self._lib.add_items(
            self._scenario,
            kind.encode('utf-8'),
            count,
            container
        )

    def add_stack(self, farmland: Farmland, position: List[float], count: int, item_kind: str) -> Stack:
        self._lib.add_stack.restype = Stack
        return self._lib.add_stack(
            self._scenario,
            farmland,
            Array2Float(*position),
            count,
            item_kind.encode('utf-8'),
        )

    def perform_farmer_action(self, farmer: FarmerTestContext, action: Dict):
        self.perform_action(farmer.player, {'Farmer': {'action': action}})

    def perform_action(self, player: str, action: Dict):
        action = json.dumps(action)
        self._lib.perform_action(self._scenario, player.encode('utf-8'), action.encode('utf-8'))

    def take_events(self) -> Dict:
        self._lib.take_events.restype = ctypes.c_void_p
        ptr = self._lib.take_events(self._scenario)
        data = ctypes.cast(ptr, ctypes.c_char_p).value.decode('utf-8')
        return json.loads(data)

    def look_around(self) -> Dict:
        self._lib.look_around.restype = ctypes.c_void_p
        ptr = self._lib.look_around(self._scenario)
        data = ctypes.cast(ptr, ctypes.c_char_p).value.decode('utf-8')
        return json.loads(data)

    def take_errors(self) -> List:
        self._lib.take_errors.restype = ctypes.c_void_p
        ptr = self._lib.take_errors(self._scenario)
        data = ctypes.cast(ptr, ctypes.c_char_p).value.decode('utf-8')
        return json.loads(data)

    def get_grid(self, grid: int) -> Dict:
        self._lib.get_grid.restype = ctypes.c_void_p
        ptr = self._lib.get_grid(self._scenario, grid)
        data = ctypes.cast(ptr, ctypes.c_char_p).value.decode('utf-8')
        return json.loads(data)

    def get_constructions(self, farmland: Farmland) -> List[Dict]:
        self._lib.get_constructions.restype = ctypes.c_void_p
        ptr = self._lib.get_constructions(self._scenario, farmland)
        data = ctypes.cast(ptr, ctypes.c_char_p).value.decode('utf-8')
        return json.loads(data)

    def get_stacks(self, farmland: Farmland) -> List[Dict]:
        self._lib.get_stacks.restype = ctypes.c_void_p
        ptr = self._lib.get_stacks(self._scenario, farmland)
        data = ctypes.cast(ptr, ctypes.c_char_p).value.decode('utf-8')
        return json.loads(data)

    def get_items(self, container: int) -> List[Dict]:
        self._lib.get_items.restype = ctypes.c_void_p
        ptr = self._lib.get_items(self._scenario, container)
        data = ctypes.cast(ptr, ctypes.c_char_p).value.decode('utf-8')
        return json.loads(data)

    def add_building(self, farmland: Farmland, position: List[float], material: Material, structure: str):
        self._lib.add_building(
            self._scenario,
            farmland,
            Array2Float(*position),
            material.value,
            f'"{structure}"'.encode('utf-8')
        )

    def rebuild_grid(self, farmland: Farmland):
        self._lib.rebuild_grid(
            self._scenario,
            farmland,
        )

    # physics

    def add_space(self, kind: str) -> int:
        return self._lib.add_space(self._scenario, kind.encode('utf-8'))

    def add_barrier(self, kind: str, space: int, position: List[float], active: bool) -> int:
        return self._lib.add_barrier(
            self._scenario,
            kind.encode('utf-8'),
            space,
            Array2Float(*position),
            int(active),
        )

    def set_body_position(self, body: int, position: List[float]):
        return self._lib.set_body_position(
            self._scenario,
            body,
            Array2Float(*position),
        )

    def change_barrier(self, id: int, active: bool) -> int:
        return self._lib.change_barrier(
            self._scenario,
            id,
            int(active),
        )

    def get_barrier(self, barrier: int) -> Dict:
        self._lib.get_barrier.restype = ctypes.c_void_p
        ptr = self._lib.get_barrier(self._scenario, barrier)
        data = ctypes.cast(ptr, ctypes.c_char_p).value.decode('utf-8')
        return json.loads(data)
