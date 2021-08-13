select *
from User
where
    User.username = $1 and User.password = $2;