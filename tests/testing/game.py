import ctypes
import json
from ctypes import c_float
from typing import List, Dict

Array2Float = c_float * 2


def define(structure_type):
    fields = []
    for name, python_type in structure_type.__annotations__.items():
        c_type = ctypes.c_void_p

        if python_type == int:
            c_type = ctypes.c_ulonglong

        fields.append((name, c_type))

    structure_type._fields_ = fields
    return structure_type


@define
class Creature(ctypes.Structure):
    id: int
    key: int
    body: int
    animal: int


class GameTestScenario:

    def __init__(self, lib):
        self._lib = lib
        self._scenario = None

    def create(self, database: str):
        self._scenario = self._lib.create(database.encode('utf-8'))

    def dispose(self):
        self._lib.dispose(self._scenario)

    def test_entity(self) -> Creature:
        self._lib.test_entity.restype = Creature
        return self._lib.test_entity()

    def test_entity2(self, value: Creature):
        return self._lib.test_entity2(value)

    # game

    def perform_action(self, action: Dict):
        action = json.dumps(action)
        self._lib.perform_action(self._scenario, action.encode('utf-8'))

    def take_events(self) -> Dict:
        self._lib.take_events.restype = ctypes.c_void_p
        ptr = self._lib.take_events(self._scenario)
        data = ctypes.cast(ptr, ctypes.c_char_p).value.decode('utf-8')
        return json.loads(data)

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
