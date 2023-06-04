import math
from dataclasses import dataclass
from typing import Protocol, Dict, List, Callable, Iterable

from behave.model import Scenario

from steps.parsers import Position
from .ffi import load_testing_library
from .game import GameTestScenario, Farmer, Farmland, Construction, Theodolite


class Planting(Protocol):
    soils: Dict[str, int]


class Physics(Protocol):
    spaces: Dict[str, int]
    barriers: Dict[str, int]
    space: int
    barrier: int


@dataclass
class FarmerTestContext:
    player: str
    entity: Farmer
    position: Position
    actions: List[Callable]


class TestBuildingSurveying:

    def __init__(self):
        self._positions = []
        self._constructions = []

    def append(self, position: Position, construction: Construction):
        self._positions.append(position)
        self._constructions.append(construction)

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


class Universe(Protocol):
    farmlands: Dict[str, Farmland]
    farmers: Dict[str, FarmerTestContext]
    theodolites: Dict[str, Theodolite]

    farmland: Farmland


class Context(Universe, Physics, Planting):
    game: GameTestScenario
    scenario: Scenario

    surveying: TestBuildingSurveying
    points: Dict[str, Position]


def setup_environment(context: Context):
    context.game = load_testing_library()


def setup_scenario(context: Context):
    context.game.create('../assets/database.sqlite')

    context.actions = []

    context.spaces = {}
    context.barriers = {}

    context.farmlands = {}
    context.farmers = {}
    context.theodolites = {}

    context.points = {}
    if context.scenario.description:
        scene = context.scenario.description
        for y in range(len(scene)):
            line = scene[y].split(' ')
            for x in range(len(line)):
                code = line[x]
                position = [x + 0.5, y + 0.5]
                context.points[code] = position


def teardown_scenario(context: Context):
    context.game.dispose()
