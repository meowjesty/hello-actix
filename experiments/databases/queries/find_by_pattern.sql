select *
from Todo
where
    Todo.task like $1;