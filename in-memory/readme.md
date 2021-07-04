# 2. In-Memory

## 2.1 The Dive-bomb into actix-web

This is the first "real" project we're going to tackle, and it has some of the more fundamental
types that will be evolving in the next examples, so before we dive too deep into actix, let's first
take a look at the structs we have defined.

### 2.1.1 Before we dive

The first in line is the `Task` struct, this is the bread and butter of our app, after all you can't
have a task management application without tasks. There is nothing fancy about it, this is just the
bare minimum of what a task is supposed to be.

Next, we have 2 structs that share a similar reason for existing, `InsertTask` and `UpdateTask`.
Both are there to help us when extracting data from a request:

- `InsertTask` for a `post` request;
- `UpdateTask` for a `put` request;

Nothing fancy so far, until we hit the `AppError` enum, which we'll be skipping in favor of
explaining the `AppData` struct first.

`AppData` is our in memory database (if you're comfortable stretching the definition of database to
be just a list). It's just a list of tasks inside a
[`Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html), and a
[`AtomicU64`](https://doc.rust-lang.org/std/sync/atomic/struct.AtomicU64.html) to track the task
ID generation.

### 2.1.2 Error: could not think of a good section title

Now we come back to explore the `AppError` enum. First, we're using
[`thiserror`](https://docs.rs/thiserror/1.0.26/thiserror/) crate to get a nice
`#[derive(Error)]` macro and `#[error("")]` attribute, this makes life easier when we want to use
custom errors. But `AppError` also implements
[`ResponseError`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/trait.ResponseError.html).

The `ResponseError` trait is how actix will be able to generate responses when (try to guess), a
request generates an error. You must implement two functions:

- [`status_code`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/trait.ResponseError.html#method.status_code): what status code should we respond with, we'll be matching on our error and try to
  respond with an appropriate HTTP code;
- [`error_response`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/trait.ResponseError.html#method.error_response): the `HttpResponse`, we'll be just converting our error into a string
  and putting it inside the body of the response;

Actix provides the HTTP status codes you expect through
[`actix_web::http::StatusCode`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/http/struct.StatusCode.html).
There are a bunch of provided by actix itself, and we'll be using those.

With this we're done with our own types, it's time to do the dive.

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
