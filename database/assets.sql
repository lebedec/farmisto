create table FarmlandAsset
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

create table FarmlandAssetProps
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
    position  json    not null
);