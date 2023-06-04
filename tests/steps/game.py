from behave import given, when, register_type

from steps.parsers import parse_position, Position
from testing import Context, FarmerTestContext, TestBuildingSurveying

register_type(Position=parse_position)


@given('{kind} farmland')
def add_farmland(context: Context, kind: str):
    name = 'test-farmland'
    farmland = context.game.add_farmland(kind)
    context.farmlands[name] = farmland
    context.farmland = farmland


@given("{kind} farmer {name}")
def step_impl(context: Context, kind: str, name: str):
    farmland = context.farmland
    position = [0.0, 0.0]
    farmer = context.game.add_farmer(name, kind, farmland, position)
    context.farmers[name] = FarmerTestContext(
        player=name,
        entity=farmer,
        position=position,
        actions=[]
    )


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


@given("{kind} theodolite as {point}")
def step_impl(context: Context, kind: str, point: str):
    position = context.points[point]
    theodolite = context.game.add_theodolite(kind, context.farmland, position)
    context.theodolites[point] = theodolite


@given("building surveying as {legend} using {theodolite}")
def step_impl(context: Context, legend: str, theodolite: str):
    context.surveying = TestBuildingSurveying()
    wall, window, door = legend.split(' ')
    scene = context.scenario.description
    surveyor = context.theodolites[theodolite].surveyor
    grid = context.farmland.grid
    for y in range(len(scene)):
        line = scene[y].split(' ')
        for x in range(len(line)):
            code = line[x]
            position = [x + 0.5, y + 0.5]
            marker = None

            if code == wall:
                marker = {'Construction': 'Wall'}
            if code == window:
                marker = {'Construction': 'Window'}
            if code == door:
                marker = {'Construction': 'Door'}

            if marker is not None:
                construction = context.game.add_construction(surveyor, marker, grid, position)
                context.surveying.append(position, construction)


@given("{kind} laid out for construction")
def step_impl(context: Context, kind: str):
    for construction in context.surveying.constructions:
        context.game.add_item(kind, construction.container)


@given("{kind} in {farmer} hands")
def step_impl(context: Context, kind: str, farmer: str):
    farmer = context.farmers[farmer]
    context.game.add_item(kind, farmer.entity.hands)


@when("{farmer} builds everything around")
def step_impl(context: Context, farmer: str):
    farmer = context.farmers[farmer]

    def build_everything_around():
        for construction in context.surveying.around(farmer.position, 2.0):
            print(construction.as_json())
            action = {'Build': {'construction': construction}}
            context.game.perform_action(farmer.player, {'Farmer': {'action': action}})

    farmer.actions.append(build_everything_around)


@when("{farmer} repeats actions in points {points}")
def step_impl(context: Context, farmer: str, points: str):
    farmer = context.farmers[farmer]
    for point in points.split(' '):
        position = context.points[point]
        context.game.set_body_position(farmer.entity.body, position)
        farmer.position = position
        for action in farmer.actions:
            action()
