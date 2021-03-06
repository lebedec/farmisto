select sqlite_version();

PRAGMA
foreign_keys = true;
PRAGMA
recursive_triggers = false;

create table SpaceKind
(
    id        integer not null
        constraint SpaceKind_pk
            primary key autoincrement,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    name      text    not null
);

create table Space
(
    id        integer not null
        constraint Space_pk
            primary key,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    kind      integer not null
        constraint Space_SpaceKind_id_fk
            references SpaceKind
);

create table BodyKind
(
    id        integer not null
        constraint BodyKind_pk
            primary key autoincrement,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    name      text    not null,
    speed     real    not null
);

create table Body
(
    id        integer not null
        constraint BodyKind_pk
            primary key,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    kind      integer not null
        constraint Body_BodyKind_id_fk
            references BodyKind,
    space     integer not null
        constraint Body_Space_id_fk
            references Space,
    position  json    not null,
    direction  json    not null
);

create table BarrierKind
(
    id        integer not null
        constraint BarrierKind_pk
            primary key autoincrement,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    name      text    not null,
    bounds    json    not null
);

create table Barrier
(
    id        integer not null
        constraint BarrierKind_pk
            primary key,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    kind      integer not null
        constraint Barrier_BarrierKind_id_fk
            references BarrierKind,
    space     integer not null
        constraint Barrier_Space_id_fk
            references Space,
    position  json    not null
);

create table LandKind
(
    id        integer not null
        constraint LandKind_pk
            primary key autoincrement,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    name      text    not null
);

create table Land
(
    id        integer not null
        constraint Land_pk
            primary key,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    kind      integer not null
        constraint Land_LandKind_id_fk
            references LandKind
);

create table PlantKind
(
    id        integer not null
        constraint PlantKind_pk
            primary key autoincrement,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    name      text    not null,
    growth    real    not null
);

create table Plant
(
    id        integer not null
        constraint PlantKind_pk
            primary key,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    kind      integer not null
        constraint Plant_PlantKind_id_fk
            references PlantKind,
    land      integer not null
        constraint Plant_Land_id_fk
            references Land
);

create table TreeKind
(
    id        integer not null
        constraint TreeKind_pk
            primary key autoincrement,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    name      text    not null,
    barrier   integer not null
        constraint Tree_BarrierKind_id_fk
            references BarrierKind,
    plant     integer not null
        constraint Tree_PlantKind_id_fk
            references PlantKind
);

create table Tree
(
    id        integer not null
        constraint TreeKind_pk
            primary key
        constraint Tree_Barrier_id_fk
            references Barrier
        constraint Tree_Plant_id_fk
            references Plant,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    kind      integer not null
        constraint Tree_TreeKind_id_fk
            references TreeKind
);

create table FarmlandKind
(
    id        integer not null
        constraint FarmlandKind_pk
            primary key autoincrement,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    name      text    not null,
    space     integer not null
        constraint Farmland_SpaceKind_id_fk
            references SpaceKind,
    land      integer not null
        constraint Farmland_LandKind_id_fk
            references LandKind
);

create table Farmland
(
    id        integer not null
        constraint FarmlandKind_pk
            primary key
        constraint Farmland_Space_id_fk
            references Space
        constraint Farmland_Land_id_fk
            references Land,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    kind      integer not null
        constraint Farmland_FarmlandKind_id_fk
            references FarmlandKind
);

create table FarmerKind
(
    id        integer not null
        constraint FarmerKind_pk
            primary key autoincrement,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    name      text    not null,
    body      integer not null
        constraint Farmer_BodyKind_id_fk
            references BodyKind
);

create table Farmer
(
    id        integer not null
        constraint FarmerKind_pk
            primary key
        constraint Farmer_Body_id_fk
            references Body,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    kind      integer not null
        constraint Farmer_FarmerKind_id_fk
            references FarmerKind,
    player    text    not null
);