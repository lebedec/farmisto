from behave import given, when, register_type

from steps.parsers import parse_position, Position
from testing import Context

register_type(Position=parse_position)


@given('{kind} farmland')
def add_farmland(context: Context, kind: str):
    pass


@when("I move {position:Position}")
def step_impl(context: Context, position: Position):
    context.game.perform_action({'Farmer': {'action': {
        'Move': {'destination': position}
    }}})
