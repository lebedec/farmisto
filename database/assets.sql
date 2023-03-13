create table FarmerAssetData
(
    id      text primary key unique,
    texture text not null
);

create table FarmlandAssetData
(
    id                          text primary key unique,
    texture                     text not null,
    sampler                     text not null references SamplerAssetData,
    building_templates          json not null,
    building_template_surveying text not null references TilesetAssetData
);

create table PropsAssetData
(
    id      text primary key unique,
    texture text not null
);

create table FarmlandAssetPropItem
(
    id       text not null references FarmlandAssetData,
    position json not null,
    rotation json not null,
    scale    json not null,
    asset    text not null references PropsAssetData
);

create table TreeAssetData
(
    id      text primary key unique,
    texture text not null
);

create table PipelineAssetData
(
    id       text primary key unique,
    fragment text not null,
    vertex   text not null
);

create table SpriteAssetData
(
    id       text primary key unique,
    texture  text not null,
    position json not null,
    size     json not null,
    sampler  text not null references SamplerAssetData,
    pivot    json not null default '[
      0.5,
      0.5
    ]'
);

create table SamplerAssetData
(
    id                       text primary key unique,
    mag_filter               text not null default 'LINEAR'
        check ( mag_filter in ('LINEAR', 'NEAREST') ),
    min_filter               text not null default 'LINEAR'
        check ( min_filter in ('LINEAR', 'NEAREST') ),
    mipmap_mode              text not null default 'LINEAR'
        check ( mipmap_mode in ('LINEAR', 'NEAREST') ),
    address_mode_u           text not null default 'REPEAT'
        check ( address_mode_u in
                ('REPEAT', 'MIRRORED_REPEAT', 'CLAMP_TO_EDGE',
                 'CLAMP_TO_BORDER') ),
    address_mode_v           text not null default 'REPEAT'
        check ( address_mode_v in
                ('REPEAT', 'MIRRORED_REPEAT', 'CLAMP_TO_EDGE',
                 'CLAMP_TO_BORDER') ),
    address_mode_w           text not null default 'REPEAT'
        check ( address_mode_w in
                ('REPEAT', 'MIRRORED_REPEAT', 'CLAMP_TO_EDGE',
                 'CLAMP_TO_BORDER') ),
    mip_lod_bias             real not null default 0.0,
    anisotropy_enable        bool not null default true,
    max_anisotropy           real not null default 16.0,
    compare_enable           bool not null default false,
    compare_op               text not null default 'ALWAYS'
        check ( compare_op in
                ('NEVER', 'LESS', 'EQUAL', 'LESS_OR_EQUAL',
                 'GREATER', 'NOT_EQUAL', 'GREATER_OR_EQUAL',
                 'ALWAYS') ),
    min_lod                  real not null default 0.0,
    max_lod                  real not null default 0.0,
    border_color             text not null default 'INT_OPAQUE_BLACK'
        check ( border_color in ('FLOAT_TRANSPARENT_BLACK',
                                 'INT_TRANSPARENT_BLACK',
                                 'FLOAT_OPAQUE_BLACK',
                                 'INT_OPAQUE_BLACK',
                                 'FLOAT_OPAQUE_WHITE',
                                 'INT_OPAQUE_WHITE') ),
    unnormalized_coordinates bool not null default false
);

create table TilesetAssetData
(
    id      text primary key unique,
    texture text not null,
    sampler text not null references SamplerAssetData,
    tiles   json not null
);

create table ItemAssetData
(
    id     text primary key unique,
    sprite text not null references SpriteAssetData
);

create table CropAssetData
(
    id    text primary key unique,
    spine text not null
);

create table CreatureAssetData
(
    id    text primary key unique,
    spine text not null
);