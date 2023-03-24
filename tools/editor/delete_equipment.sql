begin transaction;
pragma temp_store = 2;
drop table if exists target;

create temp table target
(
    barrier  integer,
    surveyor integer
);

insert into target
select barrier, p_surveyor
from Equipment
where id = :id;

delete
from Equipment
where id = :id;

delete
from Barrier
where id = (select barrier from target);

delete
from Construction
where surveyor = (select surveyor from target);

delete
from Surveyor
where id = (select surveyor from target);

commit;