begin transaction;
pragma temp_store = 2;
drop table if exists target;

create temp table target
(
    barrier  integer,
    container integer
);

insert into target
select barrier, container
from Stack
where id = :id;

delete
from Stack
where id = :id;

delete
from Barrier
where id = (select barrier from target);

delete
from Item
where container = (select container from target);

delete
from Container
where id = (select container from target);

commit;