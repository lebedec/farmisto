Feature: Строительство - Растениевод

  Background:
    Given regular farmland
    Given regular farmer Alice
    Given corn planted as $

  Scenario: Строит теплицы для растений
  . . . . . . . .
  . + + + + + . .
  . + A $ B = . T
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

  Scenario: Планирует забор для ограждения поля
  . . . . . . . .
  . + + + + + . .
  . + $ . . = . A
  . + $ $ $ + . T
  . + + + + + . .
    Given regular theodolite as T
    When Alice moves to point A
    When Alice use theodolite T
    When Alice survey building as + - =
    Then there should be 14 building markers
