from typing import  List

from behave import given, then, when, register_type
from hamcrest import equal_to, assert_that

from testing import Context

Position = List[float]


def parse_position(representation: str) -> Position:
    parts = representation.split(',')
    x, y = parts
    return [float(x), float(y)]


register_type(Position=parse_position)


@given('{kind} space')
def add_space(context: Context, kind: str):
    space = context.game.add_space(kind)
    context.space = space


@given('{kind} barrier at {position:Position}')
def add_barrier(context: Context, kind: str, position: Position):
    barrier = context.game.add_barrier(kind, context.space, position, True)
    context.barrier = barrier


@when("game deactivates barrier")
def step_impl(context: Context):
    context.game.change_barrier(context.barrier, False)


@then("barrier position is {position:Position}")
def step_impl(context: Context, position: Position):

    events = context.game.take_events()
    print(type(events), events)

    barrier = context.game.get_barrier(context.barrier)
    assert_that(barrier['position'], equal_to(position))

