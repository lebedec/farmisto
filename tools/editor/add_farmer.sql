
begin transaction;
insert into Container
values (null,
        (select id from ContainerKind where name = '<hands>'));
insert into Container
values (null,
        (select id from ContainerKind where name = '<backpack>'));
insert into Body
values (null,
        (select body from FarmerKind where name = :kind_name),
        :space,
        :position,
        :position);
insert into Player
values (null, :player);
insert into Farmer
values (null,
        (select id from FarmerKind where name = :kind_name),
        (select max(id) from Player),
        (select max(id) from Body),
        (select max(id) - 1 from Container),
        (select max(id) - 0 from Container));
commit;