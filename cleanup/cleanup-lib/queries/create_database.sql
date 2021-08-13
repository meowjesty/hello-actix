drop table if exists User;
drop view if exists OngoingTask;
drop table if exists Task;
drop table if exists Done;

create table if not exists Task (
    id integer primary key,
    created_by integer not null,
    title text not null,
    details text,
    foreign key (created_by) references User(id) on delete cascade,
    check (length(title) >= 3)
);

create table if not exists Done (
    task_id int not null,
    done_by int not null,
    foreign key (task_id) references Task(id) on delete cascade
    foreign key (done_by) references User(id) on delete cascade
);

create view if not exists OngoingTask as
    select
        Task.id,
        Task.created_by,
        Task.title,
        Task.details
    from Task
    join User on User.id = Task.created_by
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