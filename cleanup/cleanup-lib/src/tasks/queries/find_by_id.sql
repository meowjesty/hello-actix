select *
from Task
where
    Task.id = $1 and
    Task.user_id = $2;