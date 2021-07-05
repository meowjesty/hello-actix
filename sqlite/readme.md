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
- [`errors`](src/errors.rs) has our `AppError` and its `ResponseError` implementation, but now it
  has a new friend, the `TaskError`;
- [`routes`](src/routes.rs) contains our services;

That's it for the project structure, time to dig in!

## 3.2 Digging in
