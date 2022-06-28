select sqlite_version();

create table TriangleKind
(
    entry   integer not null
        constraint TriangleKind_pk
            primary key autoincrement,
    deleted bool    not null,
    id      integer not null,
    name    text    not null
);

create table Triangle
(
    entry    integer not null
        constraint Triangle_pk
            primary key autoincrement,
    deleted  bool    not null,
    id       integer not null,
    kind     integer not null
        constraint Triangle_TriangleKind_entry_fk
            references TriangleKind,
    position json    not null
);

create table QuadKind
(
    entry   integer not null
        constraint QuadKind_pk
            primary key autoincrement,
    deleted bool    not null,
    id      integer not null,
    name    text    not null
);

create table Quad
(
    entry    integer not null
        constraint Quad_pk
            primary key autoincrement,
    deleted  bool    not null,
    id       integer not null,
    kind     integer not null
        constraint Quad_QuadKind_entry_fk
            references QuadKind,
    position json    not null
);

create table EntityKind
(
    entry    integer not null
        constraint EntityKind_pk
            primary key autoincrement,
    deleted  bool    not null,
    id       integer not null,
    name     text    not null,
    triangle integer not null
        constraint EntityKind_TriangleKind_entry_fk
            references TriangleKind,
    quad     integer not null
        constraint EntityKind_QuadKind_entry_fk
            references QuadKind,
);

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