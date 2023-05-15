begin transaction;

insert into Barrier
values (null,
        (select id from BarrierKind where name = (select barrier from DoorKind where name = :kind_name)),
        (select space from Farmland where id = :farmland),
        :position,
        true);
insert into Placement
values (null,
        '"A000"',
        :pivot,
        true);

insert into Door
values (null,
        (select id from DoorKind where name = :kind_name),
        (select max(id)  from Barrier),
        (select max(id)  from Placement)
    );
commit;