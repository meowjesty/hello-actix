select *
from Task
where
    Task.title like $1 and
    Task.user_id = $2;