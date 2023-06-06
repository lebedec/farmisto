from typing import List

Tile = List[int]

Position = List[float]

Index = int


def tile(position: Position) -> Tile:
    x, y = position
    return [int(x), int(y)]
