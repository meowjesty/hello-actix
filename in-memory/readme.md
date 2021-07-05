# 2. In-Memory

## 2.1 The Dive-bomb into actix-web

This is the first "real" project we're going to tackle, and it has some of the foundational
types that will be evolving in the next examples, so before we dive too deep into actix, let's first
take a look at the "business" structs we have defined.

### 2.1.1 Before we dive

The first in line is the `Task` struct, this is the bread and butter of our app, after all you can't
have a task management application without tasks. There is nothing fancy here, this is just the
bare minimum of what a task is supposed to be.

Next, we have 2 structs that share a similar reason for existing, `InsertTask` and `UpdateTask`:

- `InsertTask` for a `post` request (task creation);
- `UpdateTask` for a `put` request (updating a task);

`AppData` is our in memory database (if you're comfortable stretching the definition of database to
be just a list). It's just a list of tasks wrapped in a
[`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html), and a
[`AtomicU64`](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicU64.html) to track the task
ID generation (more on why these fields have to be thread-safe later).

### 2.1.2 Error: could not think of a good section title

Time to explore the `AppError` enum. First, we're using
[`thiserror`](https://docs.rs/thiserror/1.0.26/thiserror/) crate to get a nice
`#[derive(Error)]` macro and `#[error("")]` attribute, this makes life easier when we want to use
custom errors. But `AppError` also implements
[`ResponseError`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/trait.ResponseError.html).

The `ResponseError` trait is how actix will be able to generate responses when a request generates
an error. You must implement two functions:

- [`status_code`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/trait.ResponseError.html#method.status_code): what status code should we respond with, we'll be matching on our error and trying to
  respond with an appropriate HTTP code;
- [`error_response`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/trait.ResponseError.html#method.error_response): the `HttpResponse` that we reply with, we'll be just converting our
  error into a string and putting it inside the body of the response;

Actix provides the HTTP status codes you expect through
[`actix_web::http::StatusCode`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/http/struct.StatusCode.html).
There are a bunch of provided by actix itself, and we'll be using those.

With this we're done with our own types, it's time to dive.

## 2.2 App configuration

As we've seen in the [`minimal`](../minimal/) project, to set up a route we just call `App::service`
and pass it our route function (`index`), but now we have 2 new friends:
[`App::app_data`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.App.html#method.app_data),
and
[`App::configure`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.App.html#method.configure).

### 2.2.1 `App::app_data` and how it relates to our struct conveniently called `AppData`

Things that we put inside `App::app_data` are stored at the _root_ level of our app, this means
that we can access it in many places throughout actix (we'll be using it in our request routes, more
about this later).

Recall that `HttpServer::new` is handling the creation of our server, and that `App` is a recipe
rather than the actual application, so when we put something in `App::app_data`, whatever we
want to store there will be in a different state for every new instance of `App` that `HttpServer`
creates (and actix uses multiple worker threads, so you'll end up with many different instances).

We want users to share the same global "database" of tasks, so we must make our `AppData` something
that can be shared across multiple threads. This is why we have `AtomicU64`, instead of just `u64`,
and our task list is wrapped in a `Mutex`.

You may have noticed that we're not passing our `AppData` struct directly into it though, first we
wrap it with some
[`Data::new`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/web/struct.Data.html#) function. This
will be clearer later when we talk about our routes, but to not leave you hanging, `Data` helps
to extract whatever we put in `App::app_data` in our routes.

### 2.2.2 `App::configure` helper

This one is just a helper to allow setting up routes in other places, rather than having to write
everything as a huge chain of `.service(index).service(insert)`. You create a function that takes
a `&mut ServiceConfig` and just chain the `.service()` calls there. We'll be using this approach to
separate different kinds of services, even though we only have task related services, later on we'll
also have user services.

Our services are set up, our `App` is configured, now let's explore the routes.

## 2.3 get post, put delete

Actix provides a macro for each HTTP method, and we'll be taking advantage of those to keep the
route handling functions really simple. The heart of our app will live on the `/tasks` path, and
I'll be using `POST` to do _insertion_, `PUT` for _update_, `DELETE` for _deleting_, and `GET` for
the different ways of _finding_.

### 2.3.1 Meet the services

If you look at each service function defined, you'll see some common parameters and the same return
type:

```rust
#[post("/tasks")]
async fn insert(app_data: Data<AppData>, input: Json<InsertTask>) -> Result<HttpResponse, AppError>

#[get("/tasks")]
async fn find_all(app_data: web::Data<AppData>) -> Result<HttpResponse, AppError>
```

Each HTTP method macro expects a `async fn` and returns a `HttpResponse`, but you may have noticed
that these functions return `Result<HttpResponse, AppError>` instead. Well, you've already seen the
`ResponseError` trait that we've implemented for `AppError`, and actix will use that trait's
`error_response` function to convert `Err(...)` into a `HttpResponse`.

There is another trait that we're not using explicitly (yet), called
[`Responder`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/trait.Responder.html) which is very
similar to `ResponseError`, but not error specific. Actix implements this trait for many
[types](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/trait.Responder.html#foreign-impls), and
`Result<T, E>` happens to be one of those, so it knows how to make a response out of `Ok(...)`.

I'm being very explicit in this project with the return types, constructing a `HttpResponse` and
returning it as `Ok(response)`, but things could be done differently, we could convert the result
of `find_all` into a string and return `Ok(task_list_string)` for example. In later projects we'll
be implementing `Responder` for our types.

Looking at the parameters, we see `(_: Data<AppData>, _: Json<InsertTask>)`. You already know what
the inner types are, and I gave a brief explanation about `Data<T>`, but now it's time to dive
deeper.

### 2.3.2 Detour to extractors-ville

These parameters are called **extractors**, and they're nifty little helpers to extract data from a
request. If not for them, you would need to define these services with a `request: HttpRequest`
parameter, and manually take the data from within `request`. Not a very productive way of doing
things, check out
[`HttpRequest`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.HttpRequest.html) if you
want to learn a bit more about doing it this way.

- `Data<AppData>`

So, back to our extractors, one that is present in every function is `Data<T>`, which extracts from
the request whatever we registered in our global `App::app_data`, and tries to convert it into `T`.
If `T` doesn't match a type registered with `App::app_data`, then you'll receive a nice `500` error
response for free.

Each request thread will have its own copy of data, and the `Data<T>` extractor only holds a
read-only reference, that's why we made the `AppData` fields multi-thread "aware", and we register
it with `App::app_data`, this is what allows us to have mutable shared access (how we create the
"global database", instead of it being just a "per request database").

- `Json<InsertTask>`, `Json<UpdateTask>`

This one is pretty straightforward, it'll extract from the request some type that may be
deserialized from json. We implement `serde::Serialize` and `serde::Deserialize` for every one of
our types.

You may implement a custom error handler for this kind of extractor with
[`JsonConfig::error_handler`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/web/struct.JsonConfig.html#method.error_handler). There are also custom `error_handler`s for the `Form`, `Path`, and
`Query` (config) extractors.

- `Path<u64>`

We use `Path<T>` to extract data from the URL, in our case the `id` for `find_by_id` and `delete`
services.

```rust
#[get("/tasks/{id}")]

#[delete("/tasks/{id}")]
```

Be careful with `{something}` path notation, as this will match on anything (it's the equivalent
to a `[^/]+` regex). So in our case we expect a number, but we're not being very explicit about it.

A `Path<T>` may also be used to extract into structs that implement `Deserialize`, and it'll match
on the struct's fields.

And with this we've covered every extractor used in the `in-memory` project, more will be coming
later, but for now this is plenty of information to extract.

### 2.3.3 POST

```rust
#[post("/tasks")]
async fn insert(app_data: Data<AppData>, input: Json<InsertTask>) -> Result<HttpResponse, AppError>
```
