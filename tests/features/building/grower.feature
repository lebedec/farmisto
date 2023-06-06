Feature: Строительство - Растениевод

  Background:
    Given regular farmland
    Given regular farmer Alice

  Scenario: Строит теплицы для растений
  . . . . . . . .
  . + + + + + . .
  . + A . B + . T
  . + + = + + . .
  . . . . . . . .
    Given regular theodolite as T
    Given building surveying as + - = using T
    Given fiberglass laid out for construction
    Given hammer in Alice hands
    When Alice builds everything around
    When Alice repeats actions in points A B
    Then second room should exist
    And room bounds is 5 x 3
    And room is mostly made of GLASS
    And room has roof, but no floor

  Scenario: Планирует ограду вокруг полей
  . . . . . . . . . .
  . + = + - o + . . .
  . ■ ◪ ■ ◪ ■ . . . .
  . ■ . . . ■ ◆ ◆ ◆ .
  . ■ A . B ■ . T ◆ .
  . ■ ■ □ ■ ■ ◆ ◇ ◆ .
  . ▪ ▫ ◆ ◇ ◌ ◦ . . .
  . ◾ ◽ ◕ ◊ ● . . . .

    Given regular theodolite as T
