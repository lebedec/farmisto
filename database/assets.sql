create table PropsAssetData
(
    id        integer not null
        constraint PropsAssetData_pk
            primary key autoincrement,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    name      text    not null unique,
    texture   text    not null,
    mesh      text    not null
);
drop trigger if exists sql_tracking_PropsAssetData;
create trigger sql_tracking_PropsAssetData
    after update
    on PropsAssetData
begin
    update PropsAssetData set timestamp = (cast(strftime('%s') as integer)) where id = new.id;
end;


create table TreeAssetData
(
    id        integer not null
        constraint TreeAssetData_pk
            primary key autoincrement,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    name      text    not null unique,
    texture   text    not null,
    mesh      text    not null
);
drop trigger if exists sql_tracking_TreeAssetData;
create trigger sql_tracking_TreeAssetData
    after update
    on TreeAssetData
begin
    update TreeAssetData set timestamp = (cast(strftime('%s') as integer)) where id = new.id;
end;


create table FarmlandAssetData
(
    id        integer not null
        constraint FarmlandAssetData_pk
            primary key autoincrement,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    name      text    not null unique
);
drop trigger if exists sql_tracking_FarmlandAssetData;
create trigger sql_tracking_FarmlandAssetData
    after update
    on FarmlandAssetData
begin
    update FarmlandAssetData set timestamp = (cast(strftime('%s') as integer)) where id = new.id;
end;
create table FarmlandAssetPropItem
(
    id       integer not null
        constraint FarmlandAssetPropData_pk
            primary key autoincrement,
    farmland integer not null
        constraint FarmlandAssetPropData_FarmlandAssetData_id_fk
            references FarmlandAssetData,
    position json    not null,
    rotation json    not null,
    scale    json    not null,
    asset    text    not null
        constraint FarmlandAssetPropItem_PropsAssetData_name_fk
            references PropsAssetData (name)
);
drop trigger if exists sql_tracking_FarmlandAssetPropDataUpdate;
create trigger sql_tracking_FarmlandAssetPropDataUpdate
    after update
    on FarmlandAssetPropItem
begin
    update FarmlandAssetData set timestamp = (cast(strftime('%s') as integer)) where id = new.farmland;
end;
drop trigger if exists sql_tracking_FarmlandAssetPropDataDelete;
create trigger sql_tracking_FarmlandAssetPropDataDelete
    after delete
    on FarmlandAssetPropItem
begin
    update FarmlandAssetData set timestamp = (cast(strftime('%s') as integer)) where id = old.farmland;
end;
drop trigger if exists sql_tracking_FarmlandAssetPropDataInsert;
create trigger sql_tracking_FarmlandAssetPropDataInsert
    after insert
    on FarmlandAssetPropItem
begin
    update FarmlandAssetData set timestamp = (cast(strftime('%s') as integer)) where id = new.farmland;
end;

create table FarmerAssetData
(
    id        integer not null
        constraint FarmerAssetData_pk
            primary key autoincrement,
    timestamp integer not null default (cast(strftime('%s') as integer)),
    deleted   bool    not null default false,
    name      text    not null unique,
    texture   text    not null,
    mesh      text    not null
);
drop trigger if exists sql_tracking_FarmerAssetData;
create trigger sql_tracking_FarmerAssetData
    after update
    on FarmerAssetData
begin
    update FarmerAssetData set timestamp = (cast(strftime('%s') as integer)) where id = new.id;
end;