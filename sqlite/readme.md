# 3. SQLite

## 3.1 Revenge of the database

It's time to drop our sorry excuse of a database for the **real** thing. Introducing SQLite and the
[`sqlx`](https://github.com/launchbadge/sqlx) crate!

This project also introduces a [`build`](build.rs) script as a way of setting up some
[environment variables](https://doc.rust-lang.org/cargo/reference/environment-variables.html) to try
and keep it (and the following projects) self-contained, and require minimal fiddling to get things
compiling and running.

A more robust approach would be to have some `Config.toml` or `Config.env` (or `Config.*`) file and
set things up there, but I find that doing it with a build script is good enough, and doesn't
require adding any new dependencies.

Before we talk new features, let's take a look from afar and understand a bit of the project
structure.

### 3.1.1 Queries

We now have a [queries](queries/) folder. I don't like writing SQL as strings "inline", so each
query has its own `query.sql` file, and will be included with the `include_str!` macro, like so:

```rust
const FIND_BY_PATTERN: &'static str = include_str!("./../queries/find_by_pattern.sql");
const FIND_ONGOING: &'static str = include_str!("./../queries/find_ongoing.sql");
const FIND_ALL: &'static str = include_str!("./../queries/find_all.sql");
const FIND_BY_ID: &'static str = include_str!("./../queries/find_by_id.sql");
const INSERT: &'static str = include_str!("./../queries/insert.sql");
const UPDATE: &'static str = include_str!("./../queries/update.sql");
const DELETE: &'static str = include_str!("./../queries/delete.sql");

const COMPLETED: &'static str = include_str!("./../queries/done.sql");
const UNDO: &'static str = include_str!("./../queries/undo.sql");
```

### 3.1.2 Separate modules

From this project forwards, we'll be separating things into modules. No more "throw everything in
`main.rs`"!

- [`models`](src/models.rs) will contain our model types and `sqlx` functionality;
- [`errors`](src/errors.rs) has our `AppError` and its `ResponseError` implementation, plus
  a new friend, the `TaskError`;
- [`routes`](src/routes.rs) contains our services;

That's it for the project structure, time to dig in!

## 3.2 Digging in

Let's start with [main.rs](src/main.rs) this time, and do a very light intro to `sqlx`. We'll be
skipping the tests until after we've looked at every module.

### 3.2.1 The main changes

As I've mentioned before, [main.rs](src/main.rs) has been refactored, and we moved a bunch of code
into modules, keeping only `index` and `main` functions.

There is a new constant `CREATE_DATABASE` accompanying our old friend `WELCOME_MSG` now. It's used
to import the SQL code as a string (`&str` actually) to be passed as the `sqlx::query` parameter.
This SQL [file](queries/create_database.sql) contains our database migration query (a very brute
migration at that). It's one of the big things you should change when adapting whatever you learn
here to "production" (together with how you set up the server).

```rust
async fn create_database(db_pool: &SqlitePool) -> Result<String, AppError>
```

This new function is a bit of a hack that we'll be using to run our migration query. It could've
been part of the [build.rs](build.rs), but doing this would require adding `sqlx` as a
`dev-dependency` for very little benefit, so I chose to just let it be.

Looking at the function signature, it takes a reference to a
[`Pool`](https://docs.rs/sqlx/0.5.5/sqlx/struct.Pool.html) of the SQLite variety (`SqlitePool` is
just a type alias to `Pool<T>` where `T` is SQLite). We'll be interacting with the database by using
this `SqlitePool` to [`acquire`](https://docs.rs/sqlx/0.5.5/sqlx/struct.Pool.html#method.acquire) a
database connection.

With the [`PoolConnection`](https://docs.rs/sqlx/0.5.5/sqlx/pool/struct.PoolConnection.html)
acquired, we can now use the [`query`](https://docs.rs/sqlx/0.5.5/sqlx/fn.query.html) function to
execute our query string (`CREATE_DATABASE`), by calling
[`Query::execute`](https://docs.rs/sqlx/0.5.5/sqlx/query/struct.Query.html#method.execute).

I don't particularly care about the results of `create_database`, only if it succeeded or not. So we
take the [`SqliteQueryResult`](https://docs.rs/sqlx/0.5.5/sqlx/sqlite/struct.SqliteQueryResult.html)
and wrap the `rows_affected` function in `Ok`.

With this out of the way, we're back to `main`.

```rust
#[actix_web::main]
async fn main() -> std::io::Result<()>
```

Right out of the gate we greet another new function
[`env_logger::init`](https://docs.rs/env_logger/0.8.4/env_logger/fn.init.html), which will be used
to log stuff. We set the `log` level to `info` in [build.rs](build.rs).

[`SqliteConnectOptions`](https://docs.rs/sqlx/0.5.5/sqlx/sqlite/struct.SqliteConnectOptions.html) is
used to configure the database,
[`filename`](https://docs.rs/sqlx/0.5.5/sqlx/sqlite/struct.SqliteConnectOptions.html#method.filename)
specifies the database file that we want (we get it from our environment variable `DATABASE_FILE`),
and
[`create_if_missing`](https://docs.rs/sqlx/0.5.5/sqlx/sqlite/struct.SqliteConnectOptions.html#method.create_if_missing)
will create the file if it doesn't exist.

We use these options when creating the `Pool` via the builder
[`PoolOptions`](https://docs.rs/sqlx/0.5.5/sqlx/pool/struct.PoolOptions.html) (we're using the
aliased version `SqlitePoolOptions`). After the `Pool` is created, we have a small "hack" to detect
if we should execute the migration query.

And we're finally ready to wrap `Pool` in our pal `Data<T>`, to be used as the `App::app_data` root
level data.

But hey, what is this `App::wrap` thing at the end? Oh, nice of you to have noticed! It's our little
introduction to actix_web
[`middleware`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/middleware/index.html). This is one
of the provided middlewares, used to neatly log stuff.

### 3.2.2 Brief introduction to middlewares

By using
[`App::wrap`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.App.html#method.wrap) we set
up a hook for our whole application. This allows us to mess with requests (preprocess),
responses (post-process), app state, and external services (logging in our case).

Middlewares are a bit of a big topic, so I'll be expanding on them whenever we hit a new one,
instead of dumping all the information in one go. Just bear in mind that they can be used to do a
bunch of stuff: session management, authorization, [saying hi](https://actix.rs/docs/middleware/),
and plenty more.

### 3.2.3 The `errors` module
