begin transaction;
insert into Container
values (null,
        (select id from ContainerKind where name = '<drop>'));
insert into Barrier
values (null,
        (select id from BarrierKind where name = '<drop>'),
        (select space from Farmland where id = :farmland),
        :position);
insert into Stack
values (null,
        (select max(id) from Container),
        (select max(id) from Barrier));
with recursive seq(x) as (select 1 union all select x + 1 from seq limit :count)
insert
into Item
select null, :itemKind, (select max(id) from Container), :functions, :quantity
from seq;
commit;

-- itemKind: 1
-- functions: '[{"Material": {"keyword": 0}}]'
-- quantity: 1
-- farmland: 1
-- position: '[2.5, 3.5]'