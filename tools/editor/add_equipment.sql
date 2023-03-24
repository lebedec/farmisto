begin transaction;
insert into Surveyor
values (null,
        (select id from SurveyorKind where name = (select p_surveyor from EquipmentKind where name = :kind_name)),
        (select grid from Farmland where id = :farmland));
insert into Barrier
values (null,
        (select id from BarrierKind where name = (select barrier from EquipmentKind where name = :kind_name)),
        (select space from Farmland where id = :farmland),
        :position);
insert into Equipment
values (null,
        (select max(id) from Barrier),
        (select id from EquipmentKind where name = :kind_name),
        (select max(id) from Surveyor));
commit;