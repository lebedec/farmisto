import enum
from typing import List

Tile = List[int]

Position = List[float]

Index = int


def tile(position: Position) -> Tile:
    x, y = position
    return [int(x), int(y)]


class Material(enum.IntEnum):
    UNKNOWN = 0
    METAL = 10
    MESH = 15
    CONCRETE = 20
    WOOD = 30
    PLANKS = 35
    GLASS = 40
    TARPAULIN = 50
