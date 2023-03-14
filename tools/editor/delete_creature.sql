begin transaction;
pragma temp_store = 2;
drop table if exists target;
create temp table target
(
    body   integer,
    animal integer
);
insert into target
select body, animal
from Creature
where id = :id;
delete
from Creature
where id = :id;
delete
from Body
where id = (select body from target);
delete
from Animal
where id = (select animal from target);
commit;