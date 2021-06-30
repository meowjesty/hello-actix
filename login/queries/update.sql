update Task
set title = $1,
    details = $2
where Task.id = $3