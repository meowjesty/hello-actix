drop view if exists OngoingTodo;
drop table if exists Todo;
create table if not exists Todo (
    id integer primary key,
    task text not null,
    details text
);
create table if not exists Done (
    todo_id int not null,
    foreign key (todo_id) references Todo(id)
);
create view if not exists OngoingTodo as
select Todo.id,
    Todo.task,
    Todo.details
from Todo
    inner join Done on Todo.id != Done.todo_id;