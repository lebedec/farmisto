Feature: Строительство - Растениевод

  Background:
    Given regular farmland
    Given regular farmer Alice

  Scenario: Строит теплицы для растений
  . . . . . . . .
  . ■ ■ ■ ■ ■ . .
  . ■ A . B ■ . T
  . ■ ■ □ ■ ■ . .
  . . . . . . . .
    Given regular theodolite as T
    Given building surveying as ■ ◪ □ using T
    Given fiberglass laid out for construction
    Given hammer in Alice hands
    When Alice builds everything around
    When Alice repeats actions in points A B
    Then should appear room X
    And room area is 15 tiles
    And room have roof
    But room have no floor

  Scenario: Планирует ограду вокруг полей
  . . . . . . . . . .
  . ■ ◪ ■ ◪ ■ . . . .
  . ■ . . . ■ ◆ ◆ ◆ .
  . ■ A . B ■ . T ◆ .
  . ■ ■ □ ■ ■ ◆ ◇ ◆ .
  . ▪ ▫ ◆ ◇ ◌ ◦ . . .
  . ◾ ◽ ◕ ◊ ● . . . .

    Given regular theodolite as T
