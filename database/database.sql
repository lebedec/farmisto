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
    id   integer primary key,
    name text not null
);

create table Space
(
    id   integer primary key,
    kind integer not null references SpaceKind
);

create table BodyKind
(
    id    integer primary key,
    name  text not null,
    speed real not null
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
    name   text not null,
    bounds json not null
);

create table Barrier
(
    id       integer primary key,
    kind     integer not null references BarrierKind,
    space    integer not null references Space,
    position json    not null
);

-- Planting

create table LandKind
(
    id   integer primary key,
    name text not null
);

create table Land
(
    id   integer primary key,
    kind integer             not null references LandKind,
    map  blob collate binary not null
);

create table PlantKind
(
    id     integer primary key,
    name   text not null,
    growth real not null
);

create table Plant
(
    id   integer primary key,
    kind integer not null references PlantKind,
    land integer not null references Land
);

-- Building

create table GridKind
(
    id   integer primary key,
    name text not null
);

create table Grid
(
    id   integer primary key,
    kind integer             not null references GridKind,
    map  blob collate binary not null
);

-- Inventory

create table ContainerKind
(
    id       integer primary key,
    name     text    not null,
    capacity integer not null
);

create table Container
(
    id   integer primary key,
    kind integer not null references ContainerKind
);

create table ItemKind
(
    id        integer primary key,
    name      text not null,
    functions json not null
);

create table Item
(
    id        integer primary key,
    kind      integer not null references ItemKind,
    container integer not null references Container
);

-- Universe

create table TreeKind
(
    id      integer primary key,
    name    text    not null,
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
    name  text    not null,
    space integer not null references SpaceKind,
    land  integer not null references LandKind,
    grid  integer not null references GridKind
);

create table Farmland
(
    id    integer primary key,
    kind  integer not null references FarmlandKind,
    space integer not null references Space,
    land  integer not null references Land,
    grid  integer not null references Grid
);

create table FarmerKind
(
    id   integer primary key,
    name text    not null,
    body integer not null references BodyKind
);

create table Farmer
(
    id     integer primary key,
    kind   integer not null references FarmerKind,
    player integer not null references Player,
    body   integer not null references Body
);

create table "Drop"
(
    id        integer primary key,
    container integer not null references Container,
    barrier   integer not null references Barrier
);

