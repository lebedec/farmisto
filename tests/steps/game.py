from behave import given, when, register_type, then
from hamcrest import assert_that, is_not, equal_to, is_in

from steps.parsers import parse_position, parse_index
from testing import Context, FarmerTestContext, BuildingSurveyingTestContext, Index, Position, RoomAssert, Material, \
    tile

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
        for position in context.points_array[key]:
            marker = markers[key]
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
            action = {'Build': {'construction': construction.as_json()}}
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
        context.game.update(0.02)


@when("{farmer} moves to point {point}")
def step_impl(context: Context, farmer: str, point: str):
    farmer = context.farmers[farmer]
    position = context.points[point]
    context.game.set_body_position(farmer.entity.body, position)


@then("{index:Index} room should exist")
def step_impl(context: Context, index: Index):
    rooms = context.game.get_grid(context.farmland.grid)
    assert_that(len(rooms) > index)
    room = rooms[index]
    min_x, min_y, max_x, max_y = room['aabb']
    context.room = RoomAssert(
        id=room['id'],
        x=0,
        y=room['area_y'],
        w=max_x - min_x + 1,
        h=max_y - min_y + 1,
        area=room['area'],
        material=Material(room['material'])
    )


@then("room bounds is {width:int} x {height:int}")
def step_impl(context: Context, width: int, height: int):
    room = context.room
    bounds = width, height
    assert_that((room.w, room.h), equal_to(bounds))


@then("room has roof, but no floor")
def step_impl(context: Context):
    assert_that(context.room.material, is_not(is_in([Material.PLANKS, Material.UNKNOWN])))
    assert_that(context.room.material, is_in([Material.GLASS, Material.PLANKS, Material.UNKNOWN]))


@then("room is mostly made of {material:Material}")
def step_impl(context: Context, material: Material):
    assert_that(context.room.material, equal_to(material))


@given("{kind} planted as {point}")
def step_impl(context: Context, kind: str, point: str):
    for position in context.points_array[point]:
        context.game.add_crop(kind, context.farmland, position)


@when("{farmer} use theodolite {theodolite}")
def step_impl(context: Context, farmer: str, theodolite: str):
    farmer = context.farmers[farmer]
    theodolite = context.theodolites[theodolite]
    action = {'UseTheodolite': {'theodolite': theodolite.as_json()}}
    context.game.perform_action(farmer.player, {'Farmer': {'action': action}})
    # TODO: remove
    context.theodolite = theodolite


@when("{farmer} survey building as {legend}")
def step_impl(context: Context, farmer: str, legend: str):
    farmer = context.farmers[farmer]
    wall, window, door = legend.split(' ')
    markers = {
        wall: {'Construction': 'Wall'},
        window: {'Construction': 'Window'},
        door: {'Construction': 'Door'},
    }
    for key in [wall, window, door]:
        for position in context.points_array[key]:
            action = {'Survey': {
                'surveyor': context.theodolite.surveyor,
                'tile': tile(position),
                'marker': markers[key]
            }}
            context.game.perform_action(farmer.player, {'Farmer': {'action': action}})


@then("there should be {count:int} constructions")
def step_impl(context: Context, count: int):
    constructions = context.game.get_constructions(context.farmland)
    assert_that(len(constructions), equal_to(count))


@given("building as {legend} made of {material:Material}")
def step_impl(context: Context, legend: str, material: Material):
    raise NotImplementedError()


@when("{farmer} survey for deconstruction {points}")
def step_impl(context: Context, farmer: str, points: str):
    raise NotImplementedError()
