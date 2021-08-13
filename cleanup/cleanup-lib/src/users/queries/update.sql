update User
set username = $1,
    password = $2
where User.id = $3