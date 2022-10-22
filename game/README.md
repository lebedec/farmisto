# Game

Game is heart of the project. Module handles mechanics, 
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

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {
  'primaryColor': '#428cd5',
  'primaryBorderColor': '#3d7ec0',
  'secondaryColor': 'white',
  'tertiaryColor': 'white',
  'lineColor': '#707070',
  'primaryTextColor': '#707070',
  'tertiaryTextColor': 'black',
  'textColor': '#707070',
  'background': 'white',
  'fontSize': '8px',
  'fontFamily': 'ui-monospace,SFMono-Regular,SF Mono,Menlo,Consolas,Liberation Mono,monospace'
  }}
}%%
flowchart LR

	player(("<h2>&nbspPlayer&nbsp</h2>[Person]")):::person
	server(("<h2>&nbspServer&nbsp</h2>[Process]")):::person
	action("<h2>Plant Apple Tree</h2>[Action]"):::component
	event1("<h2>Farmer Moved</h2>[Event]"):::external
	event2("<h2>Tree Appeared</h2>[Event]"):::component
	event3("<h2>Event ...</h2>[Event]"):::external
	universe("<h2>handle</h2>[function]"):::component
	
	tree("<h2>&nbspTree&nbsp</h2>BarrierId, ObstacleId, PlantId"):::component
	farmer("<h2>Farmer</h2>BodyId, NavigatorId"):::external
	animal("<h2>Animal</h2>BodyId, NavigatorId"):::external
	othermodel("<h2>Model ...</h2>ItemId, BodyId"):::external
	
	barrier("<h2>Barrier</h2>BarrierId"):::component
	body1("<h2>Body</h2>BodyId"):::external
	body2("<h2>Body</h2>BodyId"):::external
	
	obstacle("<h2>Obstacle</h2>ObstacleId"):::component
	navigator1("<h2>Mesh</h2>MeshId"):::external
	navigator2("<h2>Mesh</h2>MeshId"):::external
	
	plant("<h2>Farmland</h2>FarmlandId"):::component
	plant1("<h2>Plant</h2>PlantId"):::component
	plant2("<h2>Plant</h2>PlantId"):::external
	
	item("<h2>Container</h2>ContainerId"):::external
	item1("<h2>Item</h2>ItemId"):::external
	item2("<h2>Item</h2>ItemId"):::external
	
	update("<h2>update</h2>[function]"):::component
	
	player --- action & event2
	
	action & event2 --- universe
	
	universe --- tree
	
	tree --- physics & navigation & farming
	
	subgraph api[API]
		action
		event1
		event2
		event3
	end
	
	subgraph model[Model]
	
		farmer
		tree
		animal
		othermodel
	end
	
	subgraph physics[Physics]
		barrier
		body1
		body2
	end
	
	subgraph navigation[Navigation]
		obstacle
		navigator1
		navigator2
	end
	
	subgraph planting[Planting]
		plant
		plant1
		plant2
	end
	
	subgraph inventory[Domain ...]
		item
		item1
		item2
	end
	
	subgraph domains[Domains]
	        physics
		navigation
		planting 
		inventory
	end
	physics & navigation & planting & inventory --- update --- server

	classDef person fill:#07427b,stroke:#073b6f,color:white
	classDef component color:white
	classDef external fill:#999999,stroke:#707070,color:white

```

## Domains

It is the bounds within which certain game processes are implemented
and certain rules/actions are applied. In fact, each domain is unique game mechanic.
Domains are independent Rust modules.

*   [Physics](src/domains/physics.rs)

    Provides an real-time simulation of rigid body dynamics including collision detection.
    This domain is the foundation to organize game world in bounded areas for performance optimization
    and implementation of related algorithms.

*   Navigation

    Navigation domain addresses the problem of finding a path from the starting point to the destination,
    avoiding obstacles and optimizing costs (time, distance, equipment, etc).
    By using both pathfinding and movement algorithms of physics domain
    we're trying to achieve best result of task to move game entity to destination.

*   [Planting](src/domains/planting.rs)

    Planting domain associated with planting, growing, and harvesting plants on the farm.
    It's one of the main income sources for the game, and provides most of the ingredients
    for character life support process.

*   Inventory

    While playing game, player will acquire a good many items. Some will be looted from fallen enemies.
    Some will be purchased from a merchant or crafted.
    Inventory domain represents these items, solves management and storage problems.
