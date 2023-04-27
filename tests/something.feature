Feature: Fight or flight

  Scenario: Weaker opponent
    Given something
    When something "Alice"
    Then something

  Scenario: Weaker opponent II
    Given something
    When something "Boris"
    Then something

  Scenario: Physics only
    Given test space
    Given tree barrier at 0.0, 1.5
    # When game deactivates barrier
    Then barrier position is 0.0, 1.5

  Scenario: Game test
    Given test farmland