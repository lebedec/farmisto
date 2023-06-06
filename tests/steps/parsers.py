from testing import Position, Index


def parse_position(text: str) -> Position:
    parts = text.split(',')
    x, y = parts
    return [float(x), float(y)]


def parse_index(text: str) -> Index:
    if text == 'first':
        return 0
    if text == 'second':
        return 1
    if text.endswith('-th'):
        return int(text[:-3])
    return int(text)
