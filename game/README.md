# Game

Game crate is heart of the project, handles mechanics, 
provides game model and API for interaction.
Designed as server side of client-server architecture
to serve local and remote players on host computer.

## Architecture

The fundamental idea is that instead of processing just the current state of the game,
use push-only sequence of events to describe all changes to the game state.

The event sequence acts as log and can be used to materialize views of the game objects.
This leads to a number of features:

*   Flexibility. The game server publish these events so that player client can be notified and
    can handle the game state changes if needed. Client known about the type of event and
    event data, but decoupled from server runtime. In addition, multiple consumer can handle each event.
    This enables easy development and integration with other sub-systems of the game: AI, UI, animation, graphics, etc.

*   Performance. The action that initiated an event can continue,
    and clients that handle the events can run in the background.
    This can vastly improve performance and respond time of the game server.

*   Responsive AI. The continual evaluating of transition conditions
    of AI decision making engine can be expensive and evaluating all game state
    can be challenging. Rather than evaluating everything,
    it makes more sense to provide game events, so that the AI
    can make complex decisions when these events occur.

![](../.readme/diagrams/game.png)

## Modules

### Domains

It is the bounds within which certain game processes are implemented
and certain rules/actions are applied. In fact, each domain is unique game mechanic.
Domains are independent Rust modules.

*   [Universe](src/model.rs) 

    A game world is an artificial universe. In the context of the domain driven development,
    this is root domain which contains aggregates — clusters of domain objects that can be treated as a single 
    game object. 

*   [Physics](src/domains/physics)

    Provides an real-time simulation of rigid body dynamics including collision detection.
    This domain is the foundation to organize game world in bounded areas for performance optimization
    and implementation of related algorithms.

*   Navigation

    Navigation domain addresses the problem of finding a path from the starting point to the destination,
    avoiding obstacles and optimizing costs (time, distance, equipment, etc).
    By using both pathfinding and movement algorithms of physics domain
    we're trying to achieve best result of task to move game entity to destination.

*   [Planting](src/domains/planting)

    Planting domain associated with planting, growing, and harvesting plants on the farm.
    It's one of the main income sources for the game, and provides most of the ingredients
    for character life support process.

*   [Building](src/domains/building)

    Building ...

*   [Inventory](src/domains/inventory)

    While playing game, player will acquire a good many items. Some will be purchased from a merchant or crafted.
    Inventory domain represents these items, solves management and storage problems.

### Math

Probably the most common aspect of game development is vector math. This module handles this 
and provides any math calculations which can be shared between domains, server and client sides.
For example collision detection need not only for physics domain update,
but for client side prediction to compensate network lag and make possible smooth animation.

### API

When the player performs any action, the API is the single place where this action should be defined.

### Collections

This module implements specialized collection data types providing alternatives to Rust general purpose built-in 
collections.

### Data

Defines process of game save and load, separates game domains logic and concrete implementation of
data access layer.

### Model

The universe domain implements game model by composition of multiple domain objects. Any references from API
should only go to the universe objects.
