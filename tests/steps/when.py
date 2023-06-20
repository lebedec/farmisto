from behave import when, register_type

from steps.parsing import parse_position, parse_index
from testing import Context, Position, Material, \
    tile, Index

register_type(Position=parse_position)
register_type(Index=parse_index)
register_type(int=int)
register_type(Material=lambda label: Material[label])


@when("I move {position:Position}")
def step_impl(context: Context, position: Position):
    context.game.perform_action({'Farmer': {'action': {
        'Move': {'destination': position}
    }}})


@when("{farmer} builds everything around")
def step_impl(context: Context, farmer: str):
    farmer = context.farmers[farmer]

    def build_everything_around():
        for construction in context.surveying.around(farmer.position, 2.0):
            action = {'Build': {'construction': construction.as_json()}}
            context.game.perform_farmer_action(farmer, action)

    farmer.actions.append(build_everything_around)
    build_everything_around()


@when("{farmer} builds constructions {points}")
def step_impl(context: Context, farmer: str, points: str):
    farmer = context.farmers[farmer]
    for point in points.split(' '):
        position = context.points_identified[point]
        construction = context.surveying.get_by_position(position)
        action = {'Build': {'construction': construction.as_json()}}
        context.game.perform_farmer_action(farmer, action)


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


@when("{farmer} use theodolite {theodolite}")
def step_impl(context: Context, farmer: str, theodolite: str):
    farmer = context.farmers[farmer]
    theodolite = context.theodolites[theodolite]
    action = {'UseTheodolite': {'theodolite': theodolite.as_json()}}
    context.game.perform_farmer_action(farmer, action)
    # TODO: remove
    context.theodolite = theodolite


@when("{farmer} set {option:Index} mode of theodolite")
def step_impl(context: Context, farmer: str, option: Index):
    farmer = context.farmers[farmer]
    action = {'ToggleSurveyingOption': {'option': option}}
    context.game.perform_farmer_action(farmer, action)


@when("{farmer} uninstall theodolite {theodolite}")
def step_impl(context: Context, farmer: str, theodolite: str):
    farmer = context.farmers[farmer]
    theodolite = context.theodolites[theodolite]
    action = {'UninstallTheodolite': {'theodolite': theodolite.as_json()}}
    context.game.perform_farmer_action(farmer, action)


@when("{farmer} survey building as {legend}")
def step_impl(context: Context, farmer: str, legend: str):
    farmer = context.farmers[farmer]
    wall, window, door = legend.split(' ')
    for mode, key in enumerate([wall, door, window]):
        action = {'ToggleSurveyingOption': {'option': mode}}
        context.game.perform_farmer_action(farmer, action)
        for position in context.points_array[key]:
            action = {'Construct': {
                'surveyor': context.theodolite.surveyor,
                'tile': tile(position),
            }}
            context.game.perform_farmer_action(farmer, action)


@when("{farmer} survey point {point}")
def step_impl(context: Context, farmer: str, point: str):
    farmer = context.farmers[farmer]
    position = context.points_identified[point]
    action = {'Construct': {
        'surveyor': context.theodolite.surveyor,
        'tile': tile(position),
    }}
    context.game.perform_farmer_action(farmer, action)


@when("{farmer} survey for reconstruction {points} to {structure}")
def step_impl(context: Context, farmer: str, points: str, structure: str):
    farmer = context.farmers[farmer]
    points = points.split(' ')
    for point in points:
        position = context.points_identified[point]
        action = {'Reconstruct': {
            'surveyor': context.theodolite.surveyor,
            'tile': tile(position),
        }}
        context.game.perform_farmer_action(farmer, action)


@when("{farmer} survey for deconstruction {points}")
def step_impl(context: Context, farmer: str, points: str):
    farmer = context.farmers[farmer]
    points = points.split(' ')
    for point in points:
        position = context.points_identified[point]
        action = {'Deconstruct': {
            'surveyor': context.theodolite.surveyor,
            'tile': tile(position),
        }}
        context.game.perform_farmer_action(farmer, action)


@when("{farmer} install item to {point}")
def step_impl(context: Context, farmer: str, point: str):
    farmer = context.farmers[farmer]
    position = context.points[point]
    action = {'Install': {'tile': tile(position)}}
    context.game.perform_action(farmer.player, {'Farmer': {'action': action}})


@when("{farmer} takes item from stack {stack}")
def step_impl(context: Context, farmer: str, stack: str):
    farmer = context.farmers[farmer]
    stack = context.stacks[stack]
    action = {'TakeItemFromStack': {'stack': stack.as_json()}}
    context.game.perform_farmer_action(farmer, action)


@when("{farmer} takes {count:int} items from stack {stack}")
def step_impl(context: Context, farmer: str, count: int, stack: str):
    farmer = context.farmers[farmer]
    stack = context.stacks[stack]
    action = {'TakeItemFromStack': {'stack': stack.as_json()}}
    for _ in range(count):
        context.game.perform_farmer_action(farmer, action)


@when("{farmer} put items into constructions {points}")
def step_impl(context: Context, farmer: str, points: str):
    farmer = context.farmers[farmer]
    points = points.split(' ')
    for point in points:
        position = context.points_identified[point]
        construction = context.surveying.get_by_position(position)
        action = {'PutItemIntoConstruction': {'construction': construction.as_json()}}
        context.game.perform_farmer_action(farmer, action)


@when("server updates game")
def step_impl(context: Context):
    context.game.update(0.02)
