# Farmisto

```mermaid
%%{init: {'theme': 'base', 'themeVariables': {
  'primaryColor': '#428cd5',
  'primaryBorderColor': '#3d7ec0',
  'secondaryColor': 'white',
  'tertiaryColor': 'yellow',
  'lineColor': '#707070',
  'primaryTextColor': '#707070',
  'tertiaryTextColor': 'red',
  'textColor': '#707070',
  'background': 'white',
  'fontSize': '8px',
  'fontFamily': 'ui-monospace,SFMono-Regular,SF Mono,Menlo,Consolas,Liberation Mono,monospace'
  }}
}%%
flowchart LR;
	player(("<h2>&nbsp Player &nbsp</h2>[Person]")):::person
	developer(("<h2>Developer</h2>[Person]")):::person
	assets[(<h2>&nbsp assets  &nbsp</h2>&#91Files&#93)]:::component
	database[(<h2>database</h2>&#91SQLite&#93)]:::component
	datamap[(<h2>datamap</h2>)]:::component
	game(<h2>game</h2>Module<br/>Provides all of the gameplay<br/> functionality to players.):::component
	network(<h2>network</h2>Module):::component
	server(<h2>server</h2>Module):::component
	client("<h2>client</h2>[Application]"):::component
	tools(<h2>tools</h2>Module):::external
	
	developer --> database;
	developer --> tools --> assets;
	
	database --> game;
	
	
	game-->network-->client;
		assets --> client;
	

	client <-- "Views game progress,<br/>and makes actions using" ---> player;

    server---player;

	classDef person fill:#07427b,stroke:#073b6f,color:white
	classDef component color:white
	classDef external fill:#999999,stroke:#707070,color:white
	
	click database "https://github.com/lebedec/farmisto/tree/main/database"
```

## Design Principles

### Database as Single Source of Data

One of the trickiest parts of developing games is managing data.
Using configuration files works well in many simple cases, 
populating game object properties or pre-fabricating assets.

But if you want to combine datasets from a whole range of domains, 
for example, to change a balance of game object properties 
or build a context for editor mode the separated file reads turn out to be far more painful.
The bigger the game model gets, the more data becomes fragmented,
and the harder these challenges become. 

Traditional SQL database help with this:

- Bringing together and manipulating datasets from different domains
- Guarantee that game data will be consistent
- Data format standardization, use of third-party tools

### Minimal Dependence

The code should only contain the necessary pure Rust dependencies.
Especially if game development aspect relies on well known 3rd party solution,
then the pre-compiled shared binaries are an effective technique
to reduce compile-time dependencies and improve maintainability.

Here is table, showing dependencies reduction (about):

| Aspect       | Popular Solution | Alternative | Dependencies |
|--------------|------------------|-------------|--------------|
| Windowing    | winint           | rust-sdl2   | -81          |
| 3D Rendering | vulkano          | ash         | -32          |
| File Changes | notify           | OS Shell    | -20          | 

### Marcos as Lesser Evil

It is better to use the amazing features of Rust: traits, generics, const expressions.
There are only a few "legal" reasons to use macros:

- Data structure introspection to eliminate mapping code 
- The variation of arguments to simplify non-game aspects, logging and stuff like that

