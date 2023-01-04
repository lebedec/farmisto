select sqlite_version();

PRAGMA
    foreign_keys = true;
PRAGMA
    recursive_triggers = false;

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

create table TreeKind
(
    id      integer primary key,
    name    text    not null,
    barrier integer not null references BarrierKind,
    plant   integer not null references PlantKind
);

create table Tree
(
    id   integer primary key references Barrier references Plant,
    kind integer not null references TreeKind
);

create table FarmlandKind
(
    id    integer primary key,
    name  text    not null,
    space integer not null references SpaceKind,
    land  integer not null references LandKind
);

create table Farmland
(
    id   integer primary key references Space references Land,
    kind integer not null references FarmlandKind
);

create table FarmerKind
(
    id   integer primary key,
    name text    not null,
    body integer not null references BodyKind
);

create table Farmer
(
    id     integer primary key references Body,
    kind   integer not null references FarmerKind,
    player text    not null
);
