from typing import List

Position = List[float]


def parse_position(representation: str) -> Position:
    parts = representation.split(',')
    x, y = parts
    return [float(x), float(y)]
