from typing import Protocol, Dict

from .ffi import load_testing_library
from .game import GameTestScenario


class Planting(Protocol):
    soils: Dict[str, int]


class Physics(Protocol):
    spaces: Dict[str, int]
    barriers: Dict[str, int]
    space: int
    barrier: int


class Context(Physics, Planting):
    game: GameTestScenario


def setup_environment(context: Context):
    context.game = load_testing_library()


def setup_scenario(context: Context):
    context.game.create('../assets/database.sqlite')
    context.spaces = {}
    context.barriers = {}


def teardown_scenario(context: Context):
    context.game.dispose()
