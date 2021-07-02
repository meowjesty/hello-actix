# Voyage through [`actix-web`](https://github.com/actix/actix-web)

## What is this?

A bunch of **To-do** web apps written with an increasing amount of features.

1. [minimal](minimal/): A minimal "Hello, world" of sorts for
   [`actix-web`](https://github.com/actix/actix-web);
2. [in-memory](in-memory/): Using
   [`web::Data`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/web/struct.Data.html#) to hold an
   in-memory database of sorts (if you call having a `Mutex<Vec<T>>` as "using a database");
3. [sqlite](sqlite/): Gets rid of `Mutex<Vec<T>>` "database" in favor of a proper sqlite
   database [pool](https://docs.rs/sqlx/0.5.5/sqlx/struct.Pool.html), courtesy of
   [`sqlx`](https://github.com/launchbadge/sqlx);
4. [cookies](cookies/): We start playing with cookies (**DO NOT EAT**) and the
   [`actix_session`](https://github.com/actix/actix-extras/tree/master/actix-session) crate;
5. [login](login/): Identify who is eating all the cookies by tracking authentication with
   [`actix-identity`](https://github.com/actix/actix-extras/tree/master/actix-identity);
6. [authorization](authorization/): These are **MY** cookies! Allow and forbid access to routes
   with [`actix-web-httpauth`](https://github.com/actix/actix-extras/tree/master/actix-web-httpauth);

## How do I run this?

- You must have [Rust](https://www.rust-lang.org/) installed! These examples were compiled on
version [1.53.0](https://blog.rust-lang.org/2021/06/17/Rust-1.53.0.html).

Each project is completely self-contained, so if you want to run [`cookies`](cookies/), for example,
you can either run the project from its directory with a simple `cargo run`, or directly from the
workspace folder with `cargo run -p cookies`.

### List of dependencies used

- [actix-web](https://github.com/actix/actix-web): This one is kinda the whole point of the project;
- [actix-session](https://github.com/actix/actix-extras/tree/master/actix-session):
  Delicious (session) cookies;
- [actix-identity](https://github.com/actix/actix-extras/tree/master/actix-identity): Cookies for
  authentication;
- [actix-web-httpauth](https://github.com/actix/actix-extras/tree/master/actix-web-httpauth):
  Forbidden cookies (protect routes);
- [serde](https://github.com/serde-rs/serde): Serialize and deserialize our `Task`s;
- [serde_json](https://github.com/serde-rs/json): Does it with Json;
- [thiserror](https://github.com/dtolnay/thiserror): Helps us to derive our custom `Error`s;
- [sqlx](https://github.com/launchbadge/sqlx): `SQLite` and friends for our `Task` and `User`;
- [log](https://github.com/rust-lang/log): Fancy `println` to log my mistakes;
- [env_logger](https://github.com/env-logger-rs/env_logger): Actually displays the logs;
- [futures](https://github.com/rust-lang/futures-rs): We only use this for a very particular
  feature ([`impl FromRequest`](login/src/tasks/models.rs#L171));
- [time](https://github.com/time-rs/time): actix cookies expect a `Duration` from this crate when
  setting cookie expiration;
