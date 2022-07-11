drop trigger if exists sql_tracking_Space;
create trigger sql_tracking_Space
    after update
    on Space
begin
    update Space set timestamp = (cast(strftime('%s') as integer)) where id = new.id;

    update Farmland set timestamp = (cast(strftime('%s') as integer)) where id = new.id;
end;

drop trigger if exists sql_tracking_Body;
create trigger sql_tracking_Body
    after update
    on Body
begin
    update Body set timestamp = (cast(strftime('%s') as integer)) where id = new.id;
end;

drop trigger if exists sql_tracking_Barrier;
create trigger sql_tracking_Barrier
    after update
    on Barrier
begin
    update Barrier set timestamp = (cast(strftime('%s') as integer)) where id = new.id;
    
    update Tree set timestamp = (cast(strftime('%s') as integer)) where id = new.id;
end;

drop trigger if exists sql_tracking_Land;
create trigger sql_tracking_Land
    after update
    on Land
begin
    update Land set timestamp = (cast(strftime('%s') as integer)) where id = new.id;

    update Farmland set timestamp = (cast(strftime('%s') as integer)) where id = new.id;
end;

drop trigger if exists sql_tracking_Plant;
create trigger sql_tracking_Plant
    after update
    on Plant
begin
    update Plant set timestamp = (cast(strftime('%s') as integer)) where id = new.id;

    update Tree set timestamp = (cast(strftime('%s') as integer)) where id = new.id;
end;


drop trigger if exists sql_tracking_Tree;
create trigger sql_tracking_Tree
    after update
    on Tree
begin
    update Tree set timestamp = (cast(strftime('%s') as integer)) where id = new.id;
end;