select *
from OngoingTask
where Task.user_id = $1;