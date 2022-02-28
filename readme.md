# Hello [`actix-web`](https://github.com/actix/actix-web)

## ⚠️🚧🛠️ **Under Construction** 🛠️🚧⚠️

If you find mistakes, typos, horrible practices being used, please consider opening up an
[issue](https://github.com/meowjesty/hello-actix/issues/new) and telling me all about it!

## What is this?

A bunch of **To-do** web apps written with an increasing amount of features using the
[actix-web](https://github.com/actix/actix-web) framework, and its friends in
[actix-extras](https://github.com/actix/actix-extras).

This uses the `actix-web: 4` release!

### Things to keep in mind

The intention here is **not** to show how to implement a robust, **production** web service, these
examples are exploratory projects. We'll be taking a tour in Actixland by implementing a very
simple task management service, and increasing the amount of features as we get more familiar with
what actix can offer. Starting with a basic actix app with a single route, and moving towards an app
that supports protected routes.

I've tried to keep dependencies to a minimum, focusing on the basics of the framework and its
ecosystem. Another focal point was in keeping things "simple", this means that in places were things
could get complicated even a tiny little bit more (security), and take the focus away from the
exploration of actix (SECURITY), I chose to not go there (**SECURITY**).

> But there is a project literally named Authorization, how does it work then?

Well, have you heard the tales of websites that store your username and password in plaintext? This
is the level of security you should expect here. Don't get me wrong, I'll show you how to use the
authorization middleware to forbid and allow users from accessing certain routes, but we won't be
going much further than that into security practices here.

### Projects in ascending order of features

1. [minimal](minimal/): A minimal "Hello, world" of sorts for
   [`actix-web`](https://github.com/actix/actix-web);

2. [in-memory](in-memory/): Using
   [`web::Data`](https://docs.rs/actix-web/latest/actix_web/web/struct.Data.html) to hold an
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

7. [integration](integration/): We go a bit deeper in testing an actix-web app, bring a lantern and
   snacks;

8. [tls](tls/): It's HTTPS time;

## How do I run this?

- You must have [Rust](https://www.rust-lang.org/) installed! These examples were compiled on
version [1.57.0](https://blog.rust-lang.org/2021/12/02/Rust-1.57.0.html).

Each project is completely self-contained, so if you want to run [`cookies`](cookies/), for example,
you can either run the project from its directory with a simple `cargo run`, or directly from the
workspace folder with `cargo run -p cookies`.

If you want to compile all the projects in one go, just do a `cargo build` or `cargo check` in the
workspace folder.

## How do I test this?

The tests were designed to be run in **single-threaded** mode only!

You may run the following commands from the root (workspace) folder:

```sh
# Runs every test from each project
cargo test -- --test-threads=1

# Runs every test from [project]
cargo test -p sqlite -- --test-threads=1
```

### List of dependencies used

- [actix-web](https://github.com/actix/actix-web): This one is kinda the whole point of the project;
- [actix-session](https://github.com/actix/actix-extras/tree/master/actix-session):
  Delicious (session) cookies;
- [actix-identity](https://github.com/actix/actix-extras/tree/master/actix-identity): Cookies for
  authentication;
- [actix-web-httpauth](https://github.com/actix/actix-extras/tree/master/actix-web-httpauth):
  Forbidden cookies (protected routes);
- [actix-rt](https://github.com/actix/actix-net/tree/master/actix-rt): Used as a runtime by our
  tests (this is a
  [`dev-dependency`](https://doc.rust-lang.org/rust-by-example/testing/dev_dependencies.html) only);
- [serde](https://github.com/serde-rs/serde): Serialize and deserialize our `Task`s;
- [serde_json](https://github.com/serde-rs/json): Does it with JSON;
- [thiserror](https://github.com/dtolnay/thiserror): Helps us to derive our custom `Error`s;
- [sqlx](https://github.com/launchbadge/sqlx): `SQLite` and friends for our `Task` and `User`;
- [log](https://github.com/rust-lang/log): Fancy `println` to log my mistakes;
- [env_logger](https://github.com/env-logger-rs/env_logger): Actually displays the logs;
- [futures](https://github.com/rust-lang/futures-rs): We only use this for a very particular
  feature ([`impl FromRequest`](login/src/tasks/models.rs#L171));
- [time](https://github.com/time-rs/time): actix cookies expect a `Duration` from this crate when
  setting cookie expiration;
- [rustls](https://github.com/ctz/rustls): TLS library for our HTTPS server;
