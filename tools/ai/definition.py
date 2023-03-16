import json
from dataclasses import dataclass
from math import ceil
from typing import List, Tuple, Dict, Callable

import numpy as np
from IPython.display import display
from ipywidgets import interact, FloatSlider, Button
from matplotlib import pyplot as plt, patches
from numpy import sign


@dataclass
class Curve:
    x: List[float]
    y: List[float]

    @property
    def json(self) -> dict:
        return {
            'x': self.x,
            'y': self.y
        }


@dataclass
class Consideration:
    input: str
    weight: float
    curve: Curve

    @property
    def json(self) -> dict:
        return {
            'input': self.input,
            'weight': self.weight,
            'curve': self.curve.json
        }


@dataclass
class Decision:
    action: str
    considerations: List[Consideration]

    @property
    def json(self) -> dict:
        return {
            'action': self.action,
            'considerations': [consideration.json for consideration in self.considerations]
        }


@dataclass
class Behaviour:
    type: str
    name: str
    decisions: List[Decision]

    @property
    def json(self) -> dict:
        return {
            'name': self.name,
            'decisions': [decision.json for decision in self.decisions]
        }

    def update(self, **inputs):
        view_behaviour(self, inputs)
        return plt.show()

    def interact(self):
        def save_behaviour(_):
            data = open('../../assets/ai/nature.json', 'r')
            data = json.load(data)
            if self.type not in data:
                # introduce new behaviour type
                data[self.type] = []
            behaviour_type = data[self.type]
            for index, behaviour in enumerate(behaviour_type):
                if behaviour['name'] == self.name:
                    # overwrite existed behaviour definition
                    behaviour_type[index] = self.json
                    break
            else:
                # append new behaviour definition
                behaviour_type.append(self.json)
            output = open('../../assets/ai/nature.json', 'w')
            json.dump(data, output, indent=2)

        button = Button(description='Save')
        button.on_click(save_behaviour)
        display(button)

        inputs = {}
        for decision in self.decisions:
            for consideration in decision.considerations:
                if consideration.input not in inputs:
                    inputs[consideration.input] = FloatSlider(
                        value=0.5,
                        min=0.0,
                        max=1.0,
                        step=0.01,
                        continuous_update=False,
                        # layout={'width': '500px'}
                    )
        return interact(
            self.update,
            **inputs
        )


def evaluate(t: float, curve: Curve) -> Tuple[List[float], List[float]]:
    if t < 0.0:
        print(f'Incorrect value {t}, must be greater than 0')
        t = 0.0
    if t > 1.0:
        print(f'Incorrect value {t}, must be less or equal 1')
        t = 1.0
    for index, x in enumerate(curve.x):
        if x > t or x == 1.0:
            start = index - 1
            end = index
            segment = curve.x[end] - curve.x[start]
            progress = (t - curve.x[start]) / segment
            delta = curve.y[end] - curve.y[start]
            value = curve.y[start] + delta * progress
            points = (
                [curve.x[start], t, curve.x[end]],
                [curve.y[start], value, curve.y[end]]
            )
            return points


def simplify(curve: List[float], step: float, accuracy: float) -> Curve:
    x = []
    y = []
    current = 0
    last_appended = 0
    previous = curve[0]
    previous_direction = sign(curve[1] - curve[0])
    for index, value in enumerate(curve):
        delta = value - curve[last_appended]
        if abs(delta) >= accuracy or sign(value - previous) != previous_direction or index in [0, len(curve) - 1]:
            x.append(current)
            y.append(value)
            last_appended = index
        previous_direction = sign(value - previous)
        previous = value
        current += step
    return Curve(x, y)


def view_behaviour(behaviour: Behaviour, inputs: Dict[str, float]):
    rows = ceil((len(behaviour.decisions) / 3))
    figure, subplots = plt.subplots(rows, 3, figsize=(10, 3 * rows), layout='constrained')
    figure.suptitle(behaviour.name)
    palette = [
        '#fbb4aeaa',
        '#b3cde3aa',
        '#ccebc5aa',
        '#decbe4aa',
        '#fed9a6aa',
        '#ffffccaa',
        '#e5d8bdaa',
        '#fddaecaa',
        '#f2f2f2aa'
    ]
    legend = {}
    lmin = legend['Score'] = patches.Patch(edgecolor='#00000099', fill=False, label='Score', linestyle='dotted')
    # lmax = legend['Max'] = patches.Patch(edgecolor='#00000099', fill=False, label='Max', linestyle='dashed')
    scores = []
    for decision in behaviour.decisions:
        score = 1.0
        for consideration in decision.considerations:
            key = consideration.input
            if key not in legend:
                if len(palette) > 1:
                    color = palette.pop(0)
                else:
                    color = palette[0]
                legend[key] = patches.Patch(color=color, label=key)
            t = inputs[consideration.input]
            ex, ey = evaluate(t, consideration.curve)
            score = score * ey[1]
        scores.append(score)
    best = np.argmax(scores)

    for index, (decision, subplot) in enumerate(zip(behaviour.decisions, subplots.flat)):
        for consideration in decision.considerations:
            curve = consideration.curve
            color = legend[consideration.input].get_facecolor()
            subplot.plot(curve.x, curve.y, label=consideration.input, color=color)
            t = inputs[consideration.input]
            ex, ey = evaluate(t, curve)
            subplot.scatter(ex[1], ey[1], color=color)
        score = scores[index]
        subplot.plot([0.0, 1.0], [score, score], linestyle=lmin.get_linestyle(), color=lmin.get_edgecolor())
        subplot.set_title('{} {:.2f}'.format(decision.action, score), weight='bold' if index == best else 'normal')

    lines = []
    labels = []
    for label in legend:
        lines.append(legend[label])
        labels.append(label)
    figure.legend(
        lines,
        labels,
        bbox_to_anchor=(0.0, 1.02, 1.0, 0.102),
        loc='lower left',
        ncols=4,
        borderaxespad=0.0
    )


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
