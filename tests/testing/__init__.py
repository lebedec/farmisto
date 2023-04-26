from typing import Protocol

from .ffi import load_testing_library
from .game import GameTestScenario


class Context(Protocol):
    game: GameTestScenario


def setup_environment(context: Context):
    context.game = load_testing_library()


def setup_scenario(context: Context):
    context.game.create()


def teardown_scenario(context: Context):
    context.game.dispose()
