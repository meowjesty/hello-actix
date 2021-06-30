select *
from Task
where
    Task.title like $1;