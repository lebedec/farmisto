select sqlite_version();

PRAGMA
    foreign_keys = true;
PRAGMA
    recursive_triggers = false;


-- Game

create table Player
(
    id   integer primary key,
    name text not null
);

-- Timing

create table CalendarKind
(
    id           integer primary key,
    name         text    not null unique,
    day_duration integer not null,
    seasons      json    not null
);

create table Calendar
(
    id           integer primary key,
    key          integer not null references CalendarKind (id),
    season       integer not null,
    season_day   real    not null,
    times_of_day real    not null
);

-- Physics

create table SpaceKind
(
    id     integer primary key,
    name   text not null unique,
    bounds json not null
);

create table Space
(
    id    integer primary key,
    kind  integer             not null references SpaceKind (id),
    holes blob collate binary not null
);

create table BodyKind
(
    id     integer primary key,
    name   text not null unique,
    speed  real not null,
    radius real not null
);

create table Body
(
    id          integer primary key,
    kind        integer not null references BodyKind (id),
    space       integer not null references Space (id),
    position    json    not null,
    destination json    not null
);

create table BarrierKind
(
    id     integer primary key,
    name   text not null unique,
    bounds json not null
);

create table Barrier
(
    id       integer primary key,
    kind     integer not null references BarrierKind (id),
    space    integer not null references Space (id),
    position json    not null,
    active   boolean not null
);

create table SensorKind
(
    id     integer primary key,
    name   text not null unique,
    radius real not null
);

create table Sensor
(
    id       integer primary key,
    kind     integer not null references SensorKind (id),
    space    integer not null references Space (id),
    position json    not null,
    signals  json    not null
);

-- Planting

create table SoilKind
(
    id     integer primary key,
    name   text    not null unique,
    width  integer not null,
    height integer not null
);

create table Soil
(
    id        integer primary key,
    kind      integer             not null references SoilKind (id),
    fertility blob collate binary not null
);

create table PlantKind
(
    id            integer primary key,
    name          text not null unique,
    growth        real not null,
    flexibility   real not null,
    transpiration real not null
);

create table Plant
(
    id     integer primary key,
    kind   integer not null references PlantKind (id),
    soil   integer not null references Soil (id),
    impact real    not null,
    thirst real    not null,
    hunger real    not null,
    health real    not null,
    growth real    not null,
    fruits integer not null
);

-- Landscaping

create table LandKind
(
    id     integer primary key,
    name   text    not null unique,
    width  integer not null,
    height integer not null
);

create table Land
(
    id                integer primary key,
    kind              integer             not null references LandKind (id),
    moisture          blob collate binary not null,
    moisture_capacity blob collate binary not null,
    surface           blob collate binary not null
);

-- Raising

create table AnimalKind
(
    id   integer primary key,
    name text not null unique
);

create table Animal
(
    id     integer primary key,
    kind   integer not null references AnimalKind (id),
    age    real    not null,
    thirst real    not null,
    hunger real    not null,
    health real    not null,
    stress real    not null
);

-- Building

create table GridKind
(
    id   integer primary key,
    name text not null unique
);

create table Grid
(
    id   integer primary key,
    kind integer             not null references GridKind (id),
    map  blob collate binary not null
);

create table SurveyorKind
(
    id   integer primary key,
    name text not null unique
);

create table Surveyor
(
    id   integer primary key,
    kind integer not null references SurveyorKind (id),
    grid integer not null references Grid (id)
);

-- Inventory

create table ContainerKind
(
    id       integer primary key,
    name     text    not null unique,
    capacity integer not null,
    filter   json    not null
);

create table Container
(
    id   integer primary key,
    kind integer not null references ContainerKind (id)
);

create table ItemKind
(
    id           integer primary key,
    name         text not null unique,
    max_quantity integer,
    functions    json not null,
    stackable    bool not null
);

create table Item
(
    id        integer primary key,
    kind      integer not null references ItemKind (id),
    container integer not null references Container (id),
    quantity  integer not null
);

-- Assembling

create table Placement
(
    id       integer primary key,
    rotation json not null,
    pivot    json not null,
    valid    bool not null
);

-- Working

create table DeviceKind
(
    id         integer primary key,
    name       text not null unique,
    duration   real not null,
    durability real not null
);

create table Device
(
    id          integer primary key,
    kind        integer not null references DeviceKind (id),
    progress    real    not null,
    deprecation real    not null,
    enabled     bool    not null,
    broken      bool    not null,
    input       bool    not null,
    output      bool    not null
);

-- Universe

create table TreeKind
(
    id      integer primary key,
    name    text    not null unique,
    barrier integer not null references BarrierKind (id),
    plant   integer not null references PlantKind (id)
);

create table Tree
(
    id      integer primary key,
    kind    integer not null references TreeKind (id),
    barrier integer not null references Barrier (id),
    plant   integer not null references Plant (id)
);

create table FarmlandKind
(
    id       integer primary key,
    name     text not null unique,
    space    text not null references SpaceKind (name),
    soil     text not null references SoilKind (name),
    grid     text not null references GridKind (name),
    land     text not null references LandKind (name),
    calendar text not null references CalendarKind (name)
);

create table Farmland
(
    id       integer primary key,
    kind     integer not null references FarmlandKind (id),
    space    integer not null references Space (id),
    soil     integer not null references Soil (id),
    grid     integer not null references Grid (id),
    land     integer not null references Land (id),
    calendar integer not null references Calendar (id)
);

create table FarmerKind
(
    id   integer primary key,
    name text    not null unique,
    body integer not null references BodyKind (id)
);

create table Farmer
(
    id       integer primary key,
    kind     integer not null references FarmerKind (id),
    player   integer not null references Player (id),
    body     integer not null references Body (id),
    hands    integer not null references Container (id),
    backpack integer not null references Container (id)
);

create table Stack
(
    id        integer primary key,
    container integer not null references Container (id),
    barrier   integer not null references Barrier (id)
);

create table Construction
(
    id        integer primary key,
    container integer not null references Container (id),
    grid      integer not null references Grid (id),
    surveyor  integer not null references Surveyor (id),
    marker    json    not null,
    cell      json    not null
);

create table EquipmentKind
(
    id         integer primary key,
    name       text not null unique,
    item       text not null references ItemKind (name),
    barrier    text not null references BarrierKind (name),
    p_surveyor text null references SurveyorKind (name)
);

create table Equipment
(
    id         integer primary key,
    barrier    integer not null references Barrier (id),
    kind       integer not null references EquipmentKind (id),
    p_surveyor integer null references Surveyor (id)
);

create table CropKind
(
    id      integer primary key,
    name    text not null unique,
    plant   text not null references PlantKind (name),
    barrier text not null references BarrierKind (name),
    sensor  text not null references SensorKind (name),
    fruits  text not null references ItemKind (name),
    residue text not null references ItemKind (name)
);

create table Crop
(
    id      integer primary key,
    kind    integer not null references CropKind (id),
    plant   integer not null references Plant (id),
    barrier integer not null references Barrier (id),
    sensor  integer not null references Sensor (id)
);

create table CreatureKind
(
    id     integer primary key,
    name   text not null unique,
    animal text not null references AnimalKind (name),
    body   text not null references BodyKind (name)
);

create table Creature
(
    id     integer primary key,
    kind   integer not null references CreatureKind (id),
    animal integer not null references Animal (id),
    body   integer not null references Body (id)
);

create table DoorKind
(
    id      integer primary key,
    name    text not null unique,
    barrier text not null references BarrierKind (name),
    kit     text not null references ItemKind (name)
);

create table Door
(
    id        integer primary key,
    key       integer not null references DoorKind (id),
    barrier   integer not null references Barrier (id),
    placement integer not null references Placement (id)
);

create table RestKind
(
    id      integer primary key,
    name    text    not null unique,
    comfort integer not null,
    barrier text    not null references BarrierKind (name),
    kit     text    not null references ItemKind (name)
);

create table Rest
(
    id        integer primary key,
    key       integer not null references RestKind (id),
    barrier   integer not null references Barrier (id),
    placement integer not null references Placement (id)
);

create table AssemblyKind
(
    id         integer primary key,
    name       text not null unique,
    t_door     text references DoorKind (name),
    t_cementer text references CementerKind (name),
    t_rest     text references RestKind (name)
);

create table Assembly
(
    id        integer primary key,
    key       integer not null references AssemblyKind (id),
    placement integer not null references Placement (id)
);

create table CementerKind
(
    id            integer primary key,
    name          text not null unique,
    kit           text references ItemKind (name),
    barrier       text references BarrierKind (name),
    device        text references DeviceKind (name),
    input_offset  json not null,
    input         text references ContainerKind (name),
    output_offset json not null,
    output        text references ContainerKind (name),
    cement        text references ItemKind (name)
);

create table Cementer
(
    id        integer primary key,
    kind      integer not null references CementerKind (id),
    input     integer not null references Container (id),
    device    integer not null references Device (id),
    output    integer not null references Container (id),
    barrier   integer not null references Barrier (id),
    placement integer not null references Placement (id)
);