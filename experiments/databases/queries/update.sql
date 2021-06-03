update Todo
set task = $1,
    details = $2
where Todo.id = $3