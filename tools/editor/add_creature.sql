begin transaction;
insert into Animal
values (null,
        (select id from AnimalKind where name = (select animal from CreatureKind where name = :kind_name)),
        :age,
        0.0, -- thirst
        0.0, -- hunger
        1.0, -- health
        0.0, -- stress
        0.0, -- voracity
        '"Idle"'
       );
insert into Body
values (null,
        (select id from BodyKind where name = (select body from CreatureKind where name = :kind_name)),
        :space,
        :position,
        :position);
insert into Creature
values (null,
        (select id from CreatureKind where name = :kind_name),
        (select max(id) from Animal),
        (select max(id) from Body));
commit;