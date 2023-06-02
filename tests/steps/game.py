from behave import given, when, then, register_type

from steps.parsers import parse_position, Position
from testing import Context
from testing.game import Creature

register_type(Position=parse_position)


@given('{kind} farmland')
def add_farmland(context: Context, kind: str):
    pass


@when("I move {position:Position}")
def step_impl(context: Context, position: Position):
    context.game.perform_action({'Farmer': {'action': {
        'Move': {'destination': position}
    }}})


@given("farmer Alice on empty farm")
def step_impl(context: Context):
    pass


@given("creature Teddy near Alice")
def step_impl(context):
    """
    :type context: behave.runner.Context
    """
    raise NotImplementedError(u'STEP: Given creature Teddy near Alice')


@when("Alice feeds Teddy")
def step_impl(context):
    """
    :type context: behave.runner.Context
    """
    raise NotImplementedError(u'STEP: When Alice feeds Teddy')


@then("Teddy is not hungry")
def step_impl(context):
    """
    :type context: behave.runner.Context
    """
    raise NotImplementedError(u'STEP: Then Teddy is not hungry')


@given("test something")
def step_impl(context: Context):
    result = context.game.test_entity()
    print(type(result), result)
    print(result.id, result.key, result.body, result.animal, type(result.key))


@given("test something 2")
def step_impl(context: Context):
    creature = Creature(43, 2, 4, 5)
    context.game.test_entity2(creature)
