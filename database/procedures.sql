drop table if exists sql_sp_clean;
create table sql_sp_clean
(
    id       integer not null
);
create trigger sql_sp_clean_trigger
    after insert
    on sql_sp_clean
begin
    delete from Tree where deleted = true;
    delete from Farmland where deleted = true;

    delete from Barrier where deleted = true;
    delete from Body where deleted = true;
    delete from Space where deleted = true;

    delete from Plant where deleted = true;
    delete from Land where deleted = true;
end;

drop table if exists sql_sp_create_tree;
create table sql_sp_create_tree
(
    id       integer not null,
    kind     integer not null,
    position json    not null
);
create trigger sql_sp_create_tree_trigger
    after insert
    on sql_sp_create_tree
begin
    insert into Barrier
    (id, deleted, kind, space, position)
    values (new.id, false, (select barrier from TreeKind where id = new.kind), 1, new.position);

    insert into Plant
    (id, deleted, kind, land)
    values (new.id, false, (select plant from TreeKind where id = new.kind), 1);

    insert into Tree
    (id, deleted, kind)
    values (new.id, false, new.kind);
end;

drop table if exists sql_sp_delete_tree;
create table sql_sp_delete_tree
(
    id       integer not null
);
create trigger sql_sp_delete_tree_trigger
    after insert
    on sql_sp_delete_tree
begin
    update Tree set deleted = true where id = new.id;
    update Barrier set deleted = true where id = new.id;
    update Plant set deleted = true where id = new.id;
end;

