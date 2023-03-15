from typing import Callable

import numpy as np

from ai.definition import Curve, simplify


def linear(function: Callable[[float], float], accuracy: float = 0.1) -> Curve:
    step = 0.001
    fx = np.arange(0.0, 1.0 + step, step)
    fy = list(map(function, fx))
    return simplify(fy, step, accuracy)


def log(accuracy: float = 0.1) -> Curve:
    step = 0.001
    fx = np.arange(0.0, 1.0 + step, step)
    fy = list(map(lambda x: np.log(1.0 + x), fx))
    return simplify(fy, step, accuracy)