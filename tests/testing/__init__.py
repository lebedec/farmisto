from typing import Protocol, Dict

from behave.model import Scenario

from .ffi import load_testing_library
from .game import GameTestScenario, Farmer, Farmland


class Planting(Protocol):
    soils: Dict[str, int]


class Physics(Protocol):
    spaces: Dict[str, int]
    barriers: Dict[str, int]
    space: int
    barrier: int


class Universe(Protocol):
    farmlands: Dict[str, Farmland]
    farmers: Dict[str, Farmer]

    farmland: Farmland


class Context(Universe, Physics, Planting):
    game: GameTestScenario
    scenario: Scenario


def setup_environment(context: Context):
    context.game = load_testing_library()


def setup_scenario(context: Context):
    context.game.create('../assets/database.sqlite')

    context.spaces = {}
    context.barriers = {}

    context.farmlands = {}
    context.farmers = {}


def teardown_scenario(context: Context):
    context.game.dispose()
