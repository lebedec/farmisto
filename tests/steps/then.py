from behave import register_type, then
from hamcrest import assert_that, is_not, equal_to, is_in, greater_than, has_item, has_entry, empty

from steps.parsing import parse_position, parse_index
from testing import Context, Index, RoomAssert, Material

register_type(Position=parse_position)
register_type(Index=parse_index)
register_type(int=int)
register_type(Material=lambda label: Material[label])


@then("{index:Index} room should exist")
def step_impl(context: Context, index: Index):
    rooms = context.game.get_grid(context.farmland.grid)
    assert_that(len(rooms), greater_than(index))
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


@then("room has roof and floor")
def step_impl(context: Context):
    assert_that(context.room.material, is_not(is_in([Material.PLANKS, Material.UNKNOWN])))
    assert_that(context.room.material, is_not(is_in([Material.GLASS, Material.PLANKS, Material.UNKNOWN])))


@then("room is mostly made of {material:Material}")
def step_impl(context: Context, material: Material):
    assert_that(context.room.material, equal_to(material))


@then("there should be {count:int} building markers")
def step_impl(context: Context, count: int):
    constructions = context.game.get_constructions(context.farmland)
    assert_that(len(constructions), equal_to(count))


@then("theodolite should appear at {point}")
def step_impl(context: Context, point: str):
    position = context.points[point]
    events = context.game.take_events()
    for stream in events:
        if events := stream.get('UniverseStream'):
            expected_event = has_entry('TheodoliteAppeared', has_entry('position', position))
            assert_that(events, has_item(expected_event))
            break
    else:
        assert_that(False, 'Universe events not found in response')


@then("there should be no stacks")
def step_impl(context: Context):
    stacks = context.game.get_stacks(context.farmland)
    assert_that(stacks, empty())


@then("{kind} items should be in {containers}")
def step_impl(context: Context, kind: str, containers: str):

    for container in containers.split(' '):
        container = context.containers[container]
        items = context.game.get_items(container)
        expected_item = has_entry('kind', kind)
        assert_that(items, has_item(expected_item))



@then('error "{error}" should occur')
def step_impl(context: Context, error: str):
    errors = context.game.take_errors()
    assert_that(errors, equal_to(error))


@then("there should be no rooms")
def step_impl(context: Context):
    rooms = context.game.get_grid(context.farmland.grid)
    # skip first exterior "room"
    assert_that(rooms[1:], empty())
