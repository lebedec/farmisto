begin transaction;
insert into Container
values (null,
        (select id from ContainerKind where name = 'cementer-input'));
insert into Container
values (null,
        (select id from ContainerKind where name = 'cementer-output'));
insert into Barrier
values (null,
        (select id from BarrierKind where name = 'cementer'),
        (select space from Farmland where id = :farmland),
        :position,
        true);
insert into Device
values (null,
        (select id from DeviceKind where name = 'cementer'),
        0.0, -- progress
        1700.0, -- deprecation,
        false, -- enabled,
        false, -- broken,
        false, -- input
        false -- output
        );
insert into Placement
values (null,
        '"A000"',
        :pivot,
        true);

insert into Cementer
values (null,
        (select id from CementerKind where name = :kind_name),
        (select max(id) - 1 from Container),
        (select max(id)  from Device),
        (select max(id)  from Container),
        (select max(id)  from Barrier),
        (select max(id)  from main.Placement)
    );
commit;