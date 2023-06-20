begin transaction;
insert into Surveyor
values (null,
        (select id from SurveyorKind where name = (select surveyor from TheodoliteKind where name = :kind_name)),
        (select grid from Farmland where id = :farmland));
insert into Barrier
values (null,
        (select id from BarrierKind where name = (select barrier from TheodoliteKind where name = :kind_name)),
        (select space from Farmland where id = :farmland),
        :position,
        true);
insert into Theodolite
values (null,
        (select id from TheodoliteKind where name = :kind_name),
        (select max(id) from Barrier),
        (select max(id) from Surveyor));
commit;