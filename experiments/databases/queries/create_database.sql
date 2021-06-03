drop view if exists OngoingTodo;
drop table if exists Todo;
drop table if exists Done;
create table if not exists Todo (
    id integer primary key,
    task text not null,
    details text
);
create table if not exists Done (
    todo_id int not null,
    foreign key (todo_id) references Todo(id) on delete cascade
);
create view if not exists OngoingTodo as
select Todo.id,
    Todo.task,
    Todo.details
from Todo
where Todo.id not in (
        select todo_id
        from Done
    );