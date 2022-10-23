create table FarmerAssetData
(
    id      text primary key unique,
    texture text not null,
    mesh    text not null
);

create table FarmlandAssetData
(
    id text primary key unique
);

create table PropsAssetData
(
    id      text primary key unique,
    texture text not null,
    mesh    text not null
);

create table FarmlandAssetPropItem
(
    id      text primary key unique,
    farmland text not null references FarmlandAssetData,
    position json not null,
    rotation json not null,
    scale    json not null,
    asset    text not null references PropsAssetData
);

create table TreeAssetData
(
    id      text primary key unique,
    texture text not null,
    mesh    text not null
);



