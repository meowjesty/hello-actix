drop table if exists User;
drop view if exists OngoingTask;
drop table if exists Task;
drop table if exists Done;

create table if not exists Task (
    id integer primary key,
    user_id integer not null,
    title text not null,
    details text,
    foreign key (user_id) references User(id) on delete cascade,
    check (length(title) >= 3)
);

create table if not exists Done (
    task_id int not null,
    foreign key (task_id) references Task(id) on delete cascade
);

create view if not exists OngoingTask as
    select
        Task.id,
        Task.user_id,
        Task.title,
        Task.details
    from Task
    inner join User on User.id = Task.user_id
    where
        Task.id not in (
            select task_id
            from Done
        );

create table if not exists User (
    id integer primary key,
    username text not null unique,
    password text not null,
    check (length(username) >= 4),
    check (username != password)
);