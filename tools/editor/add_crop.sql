begin transaction;

insert into Plant
values (null,
        (select id from PlantKind where name = (select plant from CropKind where name = :kind_name)),
        (select soil from Farmland where id = :farmland),
        0.0, -- impact
        0.0, -- thirst
        0.0, -- hunger
        1.0, -- health
        :growth,
        (select max_fruits from PlantKind where name = (select plant from CropKind where name = :kind_name)));

insert into Barrier
values (null,
        (select id from BarrierKind where name = (select barrier from CropKind where name = :kind_name)),
        (select space from Farmland where id = :farmland),
        :position,
        true);

insert into Sensor
values (null,
        (select id from SensorKind where name = (select sensor from CropKind where name = :kind_name)),
        (select space from Farmland where id = :farmland),
        :position,
        '[]');

insert into Crop
values (null,
        (select id from CropKind where name = :kind_name),
        (select max(id) from Plant),
        (select max(id) from Barrier),
        (select max(id) from Sensor));
commit;