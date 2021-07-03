# 1. Minimal

## 1.1 Drivers, start your servers

To start sending your "Hellos" to the world with actix, we'll be using
[`HttpServer`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.HttpServer.html#), and
[`App`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.App.html#), but what
are these things?

`App` is an application
[builder](https://rust-unofficial.github.io/patterns/patterns/creational/builder.html) used to
configure things like:
[routes](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.App.html#method.route),
[data](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.App.html#method.app_data),
and [middleware](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.App.html#method.wrap).
Instances of `App` are created and ran by the `HttpServer`, which does the actual heavy lifting (it
starts the threads, binds an address, listens for requests, and more). You don't have to get
intimately familiar with `HttpServer`, as most of the things are set in `App`, we'll be using
[`HttpServer::new`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.HttpServer.html#method.new)
to create actually create the thing,
[`HttpServer::bind`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.HttpServer.html#method.bind)
to bind an address to our server, and
[`HttpServer::run`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.HttpServer.html#method.run)
(why we use this function is left as an exercise for the reader).

## 1.2 Services and routes

The minimal project could've been written with
[`route`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.App.html#method.route) instead of
[`service`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.App.html#method.service), but I
like the `#[get("/path")]` service macro better than the `#[route("/path", method="GET")]`.
Even though they are similar, they're not the same thing, and you can get a `Service` from a `Route`
with the
[`Route::service`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.Route.html#method.service)
 function.
