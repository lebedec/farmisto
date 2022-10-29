# Farmisto

Farmisto is a game about farming within are different planets of
the solar system. Use your imagination to design farm,
harvest crops, tend to animals, manage productions and finally 
take on planet environment challenges.

## Project Overview

![](.readme/diagrams/client.png)
![](.readme/diagrams/client2.png)

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
	developer(("<h2>Developer</h2>[Person]")):::person
	assets[("<h2>&nbspassets&nbsp</h2>[Files]")]:::component
	database[("<h2> database </h2>[SQL]")]:::component
	game("<h2>&nbspgame&nbsp</h2>[lib]"):::component
    network("<h2>network</h2>[lib]"):::component
	server("<h2>server</h2>[lib]"):::component
	client("<h2>client</h2>[bin]"):::component
	tools("<h2>tools</h2>[Add-ons]"):::external
	
	developer --- tools --- assets & database;
	database .- game;
	assets --- app;
	game --- server .- network --- client --> player;
	subgraph thread[Game Server Thread]
        game
        server
	end
    subgraph app[Farmisto.exe]
        thread
        network
        client
    end

	classDef person fill:#07427b,stroke:#073b6f,color:white
	classDef component color:white
	classDef external fill:#999999,stroke:#707070,color:white

    click developer "https://store.steampowered.com/search/?developer=farmisto"
    click player "https://steamcommunity.com/app/farmisto"
    click assets "https://github.com/lebedec/farmisto/tree/main/assets"
    click database "https://github.com/lebedec/farmisto/tree/main/database"
    click game "https://github.com/lebedec/farmisto/tree/main/game"
    click network "https://github.com/lebedec/farmisto/tree/main/network"
    click server "https://github.com/lebedec/farmisto/tree/main/server"
    click client "https://github.com/lebedec/farmisto/tree/main/client"
    click tools "https://github.com/lebedec/farmisto/tree/main/tools"
```

- [game](game) 

  Module handles mechanics and gameplay in general.

- [client](client)
  
  Is a game engine primarily designed 
  for game development with "hot reload" experience for editing game scenes.
  Includes a rendering for 2D or 3D graphics, sound, animation,
  AI configuration tools, networking, localization support.

- [server](server)
- [network](network)
- [database](database)
- [assets](assets)
- [tools](tools)


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

## Development

### Windows

1. Download SDL development libraries from http://www.libsdl.org
2. Download FMOD development libraries from https://www.fmod.com
3. Install a Vulkan ready driver and SDK from https://www.lunarg.com/vulkan-sdk
4. Copy all development libraries to folder of your choosing:

   Current target build:
   > ./target/debug/deps

5. Run `client/src/main.rs`, specify env variables:
   - `RUST_LOG=info` for logging
   - `FARMISTO_EDITOR=1` for dev mode
6. When you're shipping `client.exe` make sure to copy development libraries 
to the same directory that your compiled exe is in.