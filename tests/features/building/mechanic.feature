Feature: Строительство - Механизатор

  Background:
    Given regular farmland
    Given regular farmer Alice at A

  Scenario: Устанавливает нивелир в месте строительства
    . . . . .
    . T . A .
    . . . . .
    Given theodolite in Alice hands
    When Alice install item to T
    Then theodolite should appear at T

  Scenario: Размечает на готовом здании участки под снос
  . . . . . . . .
  . ■ ■ ■ ■ ■ . A
  . ■ . . . ▪₁. T
  . ■ ■ □₄■₃■₂. .
  . . . . . . . .
    Given building as ■ ▪ □ made of CONCRETE
    Given regular theodolite as T
    When Alice use theodolite T
    When Alice survey for deconstruction ▪₁ ■₂ ■₃ □₄
    Then there should be 4 building markers

  Scenario: Размечает на готовом здании новые конструкции для перестройки
  . . . . . . . .
  . ■ ■ ■ ■ ■ . A
  . ■ . . . ▪₁. T
  . ■ ■ □₂■ ■ . .
  . . . . . . . .
    Given building as ■ ▪ □ made of CONCRETE
    Given regular theodolite as T
    When Alice use theodolite T
    When Alice survey for reconstruction ▪₁ □₂ to Wall
    Then there should be 2 building markers