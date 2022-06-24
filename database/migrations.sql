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
