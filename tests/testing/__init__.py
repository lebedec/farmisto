import math
from collections import defaultdict
from dataclasses import dataclass
from typing import Protocol, Dict, Iterable

from behave.model import Scenario

from .ffi import load_testing_library
from .game import GameTestScenario, Farmer, Farmland, Construction, Theodolite, Stack, FarmerTestContext
from .types import *


class Planting(Protocol):
    soils: Dict[str, int]


class Physics(Protocol):
    spaces: Dict[str, int]
    barriers: Dict[str, int]
    space: int
    barrier: int


@dataclass
class RoomAssert:
    id: int
    x: int
    y: int
    w: int
    h: int
    area: List[int]
    material: Material


class BuildingSurveyingTestContext:

    def __init__(self):
        self._positions = []
        self._constructions = []

    def append(self, position: Position, construction: Construction):
        self._positions.append(position)
        self._constructions.append(construction)

    def get_by_position(self, position: Position) -> Construction:
        index = self._positions.index(position)
        return self._constructions[index]

    @property
    def constructions(self) -> List[Construction]:
        return self._constructions

    def around(self, center: Position, radius: float) -> Iterable[Construction]:
        for index in range(len(self._positions)):
            position = self._positions[index]
            construction = self._constructions[index]
            if distance(center, position) <= radius:
                yield construction


def distance(a: Position, b: Position):
    ax, ay = a
    bx, by = b
    x = bx - ax
    y = by - ay
    return math.sqrt(x * x + y * y)


class Inventory(Protocol):
    containers: Dict[str, int]


class Universe(Protocol):
    farmlands: Dict[str, Farmland]
    farmers: Dict[str, FarmerTestContext]
    theodolites: Dict[str, Theodolite]
    stacks: Dict[str, Stack]

    farmland: Farmland


class Context(Universe, Physics, Planting, Inventory):
    game: GameTestScenario
    scenario: Scenario

    room: RoomAssert

    surveying: BuildingSurveyingTestContext
    points: Dict[str, Position]
    points_identified: Dict[str, Position]
    points_array: Dict[str, List[Position]]
    points_array_id: Dict[str, List[str]]


def setup_environment(context: Context):
    context.game = load_testing_library()


def setup_scenario(context: Context):
    context.game.create('../assets/database.sqlite')

    context.actions = []

    context.spaces = {}
    context.barriers = {}

    context.containers = {}

    context.farmlands = {}
    context.farmers = {}
    context.theodolites = {}
    context.stacks = {}

    context.surveying = None
    context.points = {}
    context.points_array = defaultdict(list)
    context.points_array_id = defaultdict(list)
    context.points_identified = {}
    if context.scenario.description:
        scene = context.scenario.description
        for y in range(len(scene)):
            for i, code in enumerate(scene[y]):
                x = i // 2
                if i % 2 == 0:
                    position = [x + 0.5, y + 0.5]
                    context.points[code] = position
                    context.points_array[code].append(position)
                    context.points_array_id[code].append(None)
                elif code != ' ':
                    prev = scene[y][i - 1]
                    position = [x + 0.5, y + 0.5]
                    key = f'{prev}{code}'
                    context.points_identified[key] = position
                    context.points_array_id[prev][-1] = key


def teardown_scenario(context: Context):
    context.game.dispose()
