# 1. Minimal

## 1.1 Drivers, start your servers

To start sending our "Hello"s to the world with actix, we'll be using
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
intimately familiar with `HttpServer`, as most of the things are set in `App`. We'll be using
[`HttpServer::new`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.HttpServer.html#method.new),
[`HttpServer::bind`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.HttpServer.html#method.bind), and
[`HttpServer::run`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.HttpServer.html#method.run).
This is everything we'll need to start and run the web server, but note that `HttpServer::new` takes
some `factory: F` as parameter, but we're passing a closure that captures by `move`, what is
going on here? To avoid digging too deep here, this `F` generic parameter represents a function
trait ([`Fn`](https://doc.rust-lang.org/core/ops/trait.Fn.html)), so we feed it a closure that
fulfills the `HttpServer::new` requirements.

## 1.2 Services and routes

The minimal project could've been written with the
[`route`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.App.html#method.route) macro
instead of the
[`get`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/attr.get.html) service macro:

```rust
// actix-web::route macro example
#[route("/", method="GET")]
async fn index() -> HttpResponse {
    let response = HttpResponse::Ok()
        .content_type("text/html; charset=UTF-8")
        .body(WELCOME_MSG);
    response
}
```

Using the `route` macro (and the `App::route` function) allows more than one HTTP method to be
handled by the same function, while the service macro is specific to one HTTP method:

```rust
// actix-web::route macro example, multiple methods
#[route("/", method="GET", method="POST", method="PUT")]
async fn index() -> HttpResponse {
    let response = HttpResponse::Ok()
        .content_type("text/html; charset=UTF-8")
        .body(WELCOME_MSG);
    response
}
```

I prefer to have separate functions for each method, thus the examples will be going the service
route (hehe), even though we could do the same thing with `route`.

## 1.3 That's it

This project doesn't have many interesting things going on. To sum everything it does:

1. Create a route handler with the `get` macro;
2. Configure our `App` with only one service, `index`;
3. Build and run the `HttpServer`;

Instead of hard-coding the response string in the
[`HttpResponse::body`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.HttpResponseBuilder.html#method.body) (note that we're using another builder here to create the
[`HttpResponse`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.HttpResponse.html#))
function, we include a file with the handy rust macro
[`include_str!`](https://doc.rust-lang.org/core/macro.include_str.html).
