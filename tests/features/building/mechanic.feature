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

  Scenario: Проектирует здание полностью перед началом строительства
  . . . . . . . . . .
  . A T . + + = + . .
  . . . . - . . + + +
  . . . . + . . . . -
  . . . . + + + + = +
    Given regular theodolite as T
    When Alice use theodolite T
    When Alice survey building as + - =
    Then there should be 16 building markers

  Scenario: Использует несколько нивелиров для разных проектов
  . . . . . . . . . . .
  . + . . T₁. . . . ■ ■
  . + . . . A . T₂. . ▪
  . + + + . . . . . ■ ■
  . . . . . . . . . . .
    Given regular theodolite as T₁
    Given regular theodolite as T₂
    When Alice use theodolite T₁
    When Alice survey building as + - =
    When Alice use theodolite T₂
    When Alice survey building as ■ ▪ □
    Then there should be 10 building markers

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

  Scenario: Перетаскивает строительный материал на место разметки перед строительством
  . . . . . . . . .
  . +₁+ + + + . . .
  . +₂B . . + . A T
  . +₃+ = + + . . M
  . . . . . . . . .
    Given regular theodolite as T
    Given building surveying as + - = using T
    Given stack of 3 concrete as M
    When Alice takes 3 items from stack M
    When Alice moves to point B
    When Alice put items into constructions +₁ +₂ +₃
    When server updates game
    Then there should be no stacks
    Then concrete items should be in +₁ +₂ +₃

  Scenario: Использует молоток для строительства здания
  . . . . . . . .
  . + + + + + . .
  . - A H B - . T
  . + + = + + . .
  . . . . . . . .
    Given regular theodolite as T
    Given building surveying as + - = using T
    Given concrete laid out for construction
    Given stack of 1 hammer as H
    When Alice takes item from stack H
    When Alice builds everything around
    When Alice repeats actions in points B
    Then second room should exist
    And room bounds is 5 x 3
    And room is mostly made of CONCRETE
    And room has roof and floor

  Scenario: Может строить сооружения только в радиусе нескольких метров
  . . . . . . . .
  . + +₁. . . A T
  . + . . . . . .
    Given regular theodolite as T
    Given building surveying as + - = using T
    Given planks laid out for construction
    Given hammer in Alice hands
    When Alice builds constructions +₁
    Then error "TargetUnreachable" should occur

  Scenario: Использует молоток для разборки строений
  . ■ ■ ■ ■ ■ . .
  . ■ . . . ▪₁. T
  . ■ ■ □ ■ ■₂A .
    Given building as ■ ▪ □ made of CONCRETE
    Given regular theodolite as T
    Given building deconstruction at ▪₁ ■₂ using T
    Given hammer in Alice hands
    When Alice builds constructions ▪₁ ■₂
    Then there should be no rooms

  Scenario: Комбинирует несколько материалов для строительства зданий
  . . . . . . . . .
  . ■ ■ ■ +₁+₂. . .
  . ▪ B □ A =₃. T .
  . ■ ■ ■ ■ ■ . . .
  . . . . . . . . .
    Given regular theodolite as T
    Given building surveying as ■ ▪ □ using T
    Given concrete laid out for construction
    Given building surveying as + - = using T
    Given wood x 3 in Alice hands
    When Alice put items into constructions +₁ +₂ =₃
    When Alice builds everything around
    When Alice repeats actions in points B
    Then second room should exist
    And room bounds is 5 x 3
    And room is mostly made of CONCRETE
    And room has roof and floor
