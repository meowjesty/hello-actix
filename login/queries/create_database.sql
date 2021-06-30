drop view if exists OngoingTask;
drop table if exists Task;
drop table if exists Done;

create table if not exists Task (
    id integer primary key,
    title text not null,
    details text
);

create table if not exists Done (
    task_id int not null,
    foreign key (task_id) references Task(id) on delete cascade
);

create view if not exists OngoingTask as
select Task.id,
    Task.title,
    Task.details
from Task
where
    Task.id not in (
        select task_id
        from Done
    );