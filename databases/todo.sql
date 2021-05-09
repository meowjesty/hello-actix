create table Todo (
    id int primary key not null,
    task text not null,
    details text,
);

create table Done (
    todo_id int not null,
    foreign key (todo_id) references Todo(id)
);

create view OngoingTodo
as
select
    Todo.id,
	Todo.task,
	Todo.details,
from
	Todo
inner join Done on Todo.id != Done.todo_id;

select * from OngoingTodo;