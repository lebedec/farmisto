begin transaction;
pragma temp_store = 2;
drop table if exists target;
create temp table target
(
    plant   integer,
    barrier integer,
    sensor integer
);
insert into target
select plant, barrier, sensor
from Crop
where id = :id;
delete
from Crop
where id = :id;
delete
from Plant
where id = (select plant from target);
delete
from Barrier
where id = (select barrier from target);
delete
from Sensor
where id = (select sensor from target);
commit;