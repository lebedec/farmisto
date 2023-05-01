Feature: Fight or flight

  Scenario: Physics only
    Given test space
    Given tree barrier at 0.0, 1.5
    # When I move 0.0, 1.5
    When game deactivates barrier
    Then barrier position is 0.0, 1.5

  Scenario: Game test 2
    Given test farmland