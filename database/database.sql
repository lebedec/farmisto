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

-- Physics

create table SpaceKind
(
    id     integer primary key,
    name   text not null unique,
    bounds json not null
);

create table Space
(
    id   integer primary key,
    kind integer not null references SpaceKind
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
    id        integer primary key,
    kind      integer not null references BodyKind,
    space     integer not null references Space,
    position  json    not null,
    direction json    not null
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
    kind     integer not null references BarrierKind,
    space    integer not null references Space,
    position json    not null
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
    kind     integer not null references SensorKind,
    space    integer not null references Space,
    position json    not null,
    signals  json    not null
);

-- Planting

create table SoilKind
(
    id   integer primary key,
    name text not null unique
);

create table Soil
(
    id   integer primary key,
    kind integer             not null references SoilKind,
    map  blob collate binary not null
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
    kind   integer not null references PlantKind,
    soil   integer not null references Soil,
    impact real    not null,
    thirst real    not null,
    hunger real    not null,
    health real    not null,
    growth real    not null,
    fruits integer not null
);

-- Raising

create table AnimalKind
(
    id  integer primary key,
    name text not null unique
);

create table Animal
(
    id     integer primary key,
    kind   integer not null references AnimalKind,
    age    real    not null,
    thirst real    not null,
    hunger real    not null,
    health real    not null,
    stress real    not null
);

-- Building

create table GridKind
(
    id        integer primary key,
    name      text not null unique,
    materials json not null
);

create table Grid
(
    id   integer primary key,
    kind integer             not null references GridKind,
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
    kind integer not null references SurveyorKind,
    grid integer not null references Grid
);

-- Inventory

create table ContainerKind
(
    id       integer primary key,
    name     text    not null unique,
    capacity integer not null
);

create table Container
(
    id   integer primary key,
    kind integer not null references ContainerKind
);

create table ItemKind
(
    id           integer primary key,
    name         text not null unique,
    stackable    integer,
    max_quantity integer
);

create table Item
(
    id        integer primary key,
    kind      integer not null references ItemKind,
    container integer not null references Container,
    functions json    not null,
    quantity  integer not null
);

-- Universe

create table TreeKind
(
    id      integer primary key,
    name    text    not null unique,
    barrier integer not null references BarrierKind,
    plant   integer not null references PlantKind
);

create table Tree
(
    id      integer primary key,
    kind    integer not null references TreeKind,
    barrier integer not null references Barrier,
    plant   integer not null references Plant
);

create table FarmlandKind
(
    id    integer primary key,
    name  text    not null unique,
    space integer not null references SpaceKind,
    soil  integer not null references SoilKind,
    grid  integer not null references GridKind
);

create table Farmland
(
    id    integer primary key,
    kind  integer not null references FarmlandKind,
    space integer not null references Space,
    soil  integer not null references Soil,
    grid  integer not null references Grid
);

create table FarmerKind
(
    id   integer primary key,
    name text    not null unique,
    body integer not null references BodyKind
);

create table Farmer
(
    id       integer primary key,
    kind     integer not null references FarmerKind,
    player   integer not null references Player,
    body     integer not null references Body,
    hands    integer not null references Container,
    backpack integer not null references Container
);

create table "Drop"
(
    id        integer primary key,
    container integer not null references Container,
    barrier   integer not null references Barrier
);

create table Construction
(
    id        integer primary key,
    container integer not null references Container,
    grid      integer not null references Grid,
    surveyor  integer not null references Surveyor,
    cell      json    not null
);

create table Theodolite
(
    id   integer primary key,
    cell json not null
);

create table EquipmentKind
(
    id         integer primary key,
    name       text not null unique,
    barrier    text not null references BarrierKind (name),
    p_surveyor text null references SurveyorKind (name)
);

create table Equipment
(
    id         integer primary key,
    barrier    integer not null references Barrier,
    kind       integer not null references EquipmentKind,
    p_surveyor integer null references Surveyor
);

create table CropKind
(
    id      integer primary key,
    name    text not null unique,
    plant   text not null references PlantKind (name),
    barrier text not null references BarrierKind (name),
    sensor  text not null references SensorKind (name)
);

create table Crop
(
    id      integer primary key,
    kind    integer not null references CropKind,
    plant   integer not null references Plant,
    barrier integer not null references Barrier,
    sensor  integer not null references Sensor
);

create table CreatureKind
(
    id    integer primary key,
    name   text not null unique,
    animal text not null references AnimalKind (name),
    body   text not null references BodyKind (name)
);

create table Creature
(
    id     integer primary key,
    kind   integer not null references CreatureKind,
    animal integer not null references Animal,
    body   integer not null references Body
);