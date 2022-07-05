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
	gamer(("<h2>&nbsp Gamer &nbsp</h2>[Person]")):::person
	developer(("<h2>Developer</h2>[Person]")):::person
	assets[(<h2>&nbsp assets  &nbsp</h2>&#91Files&#93)]:::component
	database[(<h2>database</h2>&#91SQLite&#93)]:::component
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
	

	client <-- "Views game progress,<br/>and makes actions using" ---> gamer;

    server---gamer;

	classDef person fill:#07427b,stroke:#073b6f,color:white
	classDef component color:white
	classDef external fill:#999999,stroke:#707070,color:white
	
	click database "https://github.com/lebedec/farmisto/tree/main/database"
```

## Design Principles

### Minimal Dependencies Count

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