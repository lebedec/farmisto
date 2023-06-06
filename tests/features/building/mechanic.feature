Feature: Строительство - Механизатор

  Background:
    Given regular farmland
    Given regular farmer Alice

  Scenario: Размечает на готовом здании участки под снос
  . . . . . . . .
  . ■ ■ ■ ■ ■ . A
  . ■ . . . ▪₁. T
  . ■ ■ □₄■₃■₂. .
  . . . . . . . .
    Given building as ■ ▪ □ made of CONCRETE
    Given regular theodolite as T
    When Alice moves to point A
    When Alice use theodolite T
    When Alice survey for deconstruction ▪₁ ■₂ ■₃ □₄