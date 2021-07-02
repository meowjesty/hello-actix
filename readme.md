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
