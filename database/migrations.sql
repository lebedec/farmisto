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
