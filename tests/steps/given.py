from behave import given, register_type

from steps.parsing import parse_position, parse_index
from testing import Context, FarmerTestContext, BuildingSurveyingTestContext, Material

register_type(Position=parse_position)
register_type(Index=parse_index)
register_type(int=int)
register_type(Material=lambda label: Material[label])


@given('{kind} farmland')
def add_farmland(context: Context, kind: str):
    name = 'test-farmland'
    farmland = context.game.add_farmland(kind)
    context.farmlands[name] = farmland
    context.farmland = farmland


@given("{kind} farmer {name} at {point}")
def step_impl(context: Context, kind: str, name: str, point: str):
    farmland = context.farmland
    position = context.points[point]
    farmer = context.game.add_farmer(name, kind, farmland, position)
    context.farmers[name] = FarmerTestContext(
        player=name,
        entity=farmer,
        position=position,
        actions=[]
    )


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
    position = context.points.get(point, context.points_identified.get(point))
    theodolite = context.game.add_theodolite(kind, context.farmland, position)
    context.theodolites[point] = theodolite


@given("building surveying as {legend} using {theodolite}")
def step_impl(context: Context, legend: str, theodolite: str):
    if context.surveying is None:
        context.surveying = BuildingSurveyingTestContext()
    wall, window, door = legend.split(' ')
    markers = {
        wall: {'Construction': 'Wall'},
        window: {'Construction': 'Window'},
        door: {'Construction': 'Door'},
    }
    surveyor = context.theodolites[theodolite].surveyor
    grid = context.farmland.grid
    for key in [wall, window, door]:
        for index, position in enumerate(context.points_array[key]):
            marker = markers[key]
            construction = context.game.add_construction(surveyor, marker, grid, position)
            context.surveying.append(position, construction)
            point_id = context.points_array_id[key][index]
            if point_id is not None:
                context.containers[point_id] = construction.container


@given("building deconstruction at {points} using {theodolite}")
def step_impl(context: Context, points: str, theodolite: str):
    context.surveying = BuildingSurveyingTestContext()
    marker = 'Deconstruction'
    surveyor = context.theodolites[theodolite].surveyor
    grid = context.farmland.grid
    for point in points.split(' '):
        position = context.points_identified.get(point)
        construction = context.game.add_construction(surveyor, marker, grid, position)
        context.surveying.append(position, construction)


@given("{kind} laid out for construction")
def step_impl(context: Context, kind: str):
    for construction in context.surveying.constructions:
        context.game.add_items(kind, 1, construction.container)


@given("{kind} x {count:int} in {farmer} hands")
def step_impl(context: Context, kind: str, count: int, farmer: str):
    farmer = context.farmers[farmer]
    context.game.add_items(kind, count, farmer.entity.hands)
    context.game.set_farmer_activity(farmer.entity, 'Usage')


@given("{kind} in {farmer} hands")
def step_impl(context: Context, kind: str, farmer: str):
    farmer = context.farmers[farmer]
    context.game.add_items(kind, 1, farmer.entity.hands)
    context.game.set_farmer_activity(farmer.entity, 'Usage')


@given("{kind} planted as {point}")
def step_impl(context: Context, kind: str, point: str):
    for position in context.points_array[point]:
        context.game.add_crop(kind, context.farmland, position)


@given("building as {legend} made of {material:Material}")
def step_impl(context: Context, legend: str, material: Material):
    wall, window, door = legend.split(' ')
    structures = {
        wall: 'Wall',
        window: 'Window',
        door: 'Door',
    }
    for key in [wall, window, door]:
        for point in context.points_array[key]:
            context.game.add_building(context.farmland, point, material, structures[key])
    context.game.rebuild_grid(context.farmland)


@given("stack of {count:int} {kind} as {point}")
def step_impl(context: Context, count: int, kind: str, point: str):
    position = context.points[point]
    stack = context.game.add_stack(context.farmland, position, count, kind)
    context.stacks[point] = stack
