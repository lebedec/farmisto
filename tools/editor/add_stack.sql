begin transaction;
insert into Container
values (null,
        (select id from ContainerKind where name = '<drop>'));
insert into Barrier
values (null,
        (select id from BarrierKind where name = '<drop>'),
        (select space from Farmland where id = :farmland),
        :position,
        true);
insert into Stack
values (null,
        (select max(id) from Container),
        (select max(id) from Barrier));
with items(kind) as (select value from json_each(:items))
insert
into Item
select null, (select id from ItemKind where name = items.kind), (select max(id) from Container), :quantity
from items;
commit;

-- items: '["door-x1"]'
-- quantity: 1
-- farmland: 1
-- position: '[1.5, 2.5]'

