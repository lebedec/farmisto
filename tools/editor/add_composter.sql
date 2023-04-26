begin transaction;
insert into Container
values (null,
        (select id from ContainerKind where name = (select input from ComposterKind where name = :kind_name)));
insert into Container
values (null,
        (select id from ContainerKind where name = (select output from ComposterKind where name = :kind_name)));
insert into Barrier
values (null,
        (select id from BarrierKind where name = (select barrier from ComposterKind where name = :kind_name)),
        (select space from Farmland where id = :farmland),
        :position,
        true);
insert into Device
values (null,
        (select id from DeviceKind where name = (select device from ComposterKind where name = :kind_name)),
        0.0, -- progress
        0.0, -- deprecation,
        true, -- enabled,
        false, -- broken,
        false, -- input
        false -- output
        );
insert into Placement
values (null,
        '"A000"',
        :pivot,
        true);

insert into Composter
values (null,
        (select id from ComposterKind where name = :kind_name),
        (select max(id) - 1 from Container),
        (select max(id)  from Device),
        (select max(id)  from Container),
        (select max(id)  from Barrier),
        (select max(id)  from main.Placement)
    );
commit;