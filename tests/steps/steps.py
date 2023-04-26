from behave import given, when, then

from testing import Context


@given('something')
def step_impl(context: Context):
    print(type(context))
    print('GIVEN something')


@when('something "{data}"')
def step_impl(context: Context, data: str):
    print('WHEN something')
    context.game.change_data(data)


@then("something")
def step_impl(context: Context):
    print('THEN something')
