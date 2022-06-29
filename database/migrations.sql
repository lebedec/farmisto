select sqlite_version();

create table SpaceKind
(
    id        integer not null
        constraint SpaceKind_pk
            primary key autoincrement,
    timestamp integer not null default -1,
    deleted   bool    not null default false,
    name      text    not null
)

create table Space
(
    id        integer not null
        constraint Space_pk
            primary key autoincrement,
    timestamp integer not null default -1,
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
    timestamp integer not null default -1,
    deleted   bool    not null default false,
    name      text    not null,
    speed     real    not null
);

create table Body
(
    id        integer not null
        constraint BodyKind_pk
            primary key autoincrement,
    timestamp integer not null default -1,
    deleted   bool    not null default false,
    kind      integer not null
        constraint Body_BodyKind_id_fk
            references BodyKind,
    space     integer not null
        constraint Body_Space_id_fk
            references Space,
    position  json    not null
);

create table BarrierKind
(
    id        integer not null
        constraint BarrierKind_pk
            primary key autoincrement,
    timestamp integer not null default -1,
    deleted   bool    not null default false,
    name      text    not null,
    bounds    json    not null
);

create table Barrier
(
    id        integer not null
        constraint BarrierKind_pk
            primary key autoincrement,
    timestamp integer not null default -1,
    deleted   bool    not null default false,
    kind      integer not null
        constraint Barrier_BarrierKind_id_fk
            references BarrierKind,
    space     integer not null
        constraint Barrier_Space_id_fk
            references Space,
    position  json    not null
);