# 7 Integration

## 7.1 The three Ts: Tests, tests, tests

It's time to finally dig-in into testing our actix services. So far I've asked of you to overlook
the tests, and focus on the actix features. Now we're going to make sure that the things we've been
using actually work!

One of the reasons I delayed going further into test explanations is that we've been putting them
in the `routes.rs` files of our projects, plus they also contained a fair bit of duplication.
Doing integration tests on _binary_ crates is a bit less clean than on _library_ crates.

That's why the _integration_ project has been divided into 2 crates:

- [integration](Cargo.toml) contains a single (and very short) [main.rs](src/main.rs) file, while;
- [integration-lib](integration-lib/Cargo.toml) has the bulk of our application, basically
  everything is now done in `integration-lib`, plus tests;

Doing integration tests in a _lib_ crate is easy, you just create a [tests](integration-lib/tests/)
folder that lives next to [src](integration-lib/src/), and cargo will do its magic. If you want to
understand a bit more, check out the
Rust [book](https://doc.rust-lang.org/book/ch11-03-test-organization.html) chapter on this.

If you understand how the tests are working here, going back to earlier projects you'll see that
they're pretty similar (a bunch will be exactly the same).

We won't be looking at each test individually, this would become repetitive pretty fast, I'll
be showing you only the interesting parts.

We'll start with `/users` tests, as we're dealing with authorization and testing some `/tasks`
services will require `User` setup.

## 7.2 Testing [users](integration-lib/tests/test_user_routes.rs)

First off, ignore the macro at the start, we'll come back to it later!

The first test we'll be looking at is:

```rust
#[actix_rt::test]
async fn test_user_insert_valid_user()
```

As I've told you before, `actix_rt::test` is the _async_ runtime for our test. We must set up a
`App`, but instead of using `users::user_service` as the configuration for `App::configure`, we'll
be setting only the services we're interested in testing, in this case `users::insert` (here aliased
to `users::user_insert`).

`test::init_service(app)` starts our server from our `App` builder, it must be mutable to comply
with the `test::call_service` function.

We then create our `test::TestRequest` with `POST` to the URI `/users/register`, remember that this
route is not protected (not wrapped in `HttpAuthentication` middleware), so we just need to set the
request body, and no headers.

`test::call_service(&mut app, request)` is aptly named, it'll call our service with the request
we've just created, and returns as `ServiceResponse` (not a `HttpResponse`!). And finally, we just
assert the response as successful.

If for some reason your `response` appears as type `{unknown}`, just add the type manually:

```rust
let response: ServiceResponse = test_call_service(&mut app, request).await;
```

The code pattern of this test is pretty consistent with what other tests want, so I'll be using a
couple of macros to keep things from being repeated.

```rust
#[actix_rt::test]
pub async fn test_user_update_valid_user()
```

This is our first test to take advantage of our pair of macros: `setup_app!` and `pre_insert_user`.
So let's take a small detour to explain each, before we come back to the `users` tests.

## 7.3 The macro rules! detour

Let's start with the simpler of the two macros:

### 7.3.1 pre_insert_user

```rust
macro_rules! pre_insert_user {
    ($app: expr) => {{
        // ...
        user
    }}
```

If you look at it, this is almost a copy of our `test_user_insert_valid_user` test function, except
that: it doesn't start a server, it just inserts a user and returns it. We're not using any mocking
library, and we're not pre-inserting into our test database either, so this macro is a handy way of
inserting a user that we'll call before any test that requires an existing `User`.

Instead of creating the server, we take it as the macro parameter `$app: expr`.

There is one new thing in there:

```rust
let user: User = test::read_body_json(insert_user_response).await;
```

[`test::read_body_json`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/test/fn.read_body_json.html)
is a neat actix helper that takes a `ServiceResponse` and extracts the json body into our `User`
type. It takes a bit of fiddling with a `ServiceResponse` to do the same without this function and
its friend
[`test::read_body`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/test/fn.read_body.html).

The second macro you'll see is the `setup_app!` call, in
[mod.rs](integration-lib/tests/common/mod.rs), and it's a tiny bit bigger than this one.

### 7.3.2 setup_app

```rust
#[macro_export]
macro_rules! setup_app {
    ($configure: expr) => {{
        // ...
        (app, bearer_token, cookies)
    }}
```

Well, the reason for this macro existence comes mainly from the fact that we must deal with a bunch
of protected routes. Which means that for many tests we would have to do the following:

1. Set up `App` with `users::insert` and `users::login` services, so that our requests may go
   through authentication;
2. Do the work of inserting a user and logging in with it;
3. Extract the authentication token from the login response, to use it on requests that require it;

Doing this for each test gets old really fast, thus a macro to rescue us.

The `$configure: expr` parameter is used to pass the tests' relevant services to the `App` builder.
Tests that use this macro will start with a closure `|cfg: &mut ServiceConfig|`, much like our
`users::user_service` and `tasks::task_service` functions.

This macro also sets the necessary middlewares to handle login: `IdentityService` and
`CookieSession`.

The bulk of it is the call to register a user, then use it to login. We retrieve the `auth-cookie`,
and the `session-cookie` from the login `ServiceResponse`, and return the initialized `App`, the
bearer token, and the cookies we extracted.

These 3 pieces are all we'll be needing to go on with the tests.

## 7.4 Back to the [users](integration-lib/tests/test_user_routes.rs)

Let's jump to the `/users` update test:

```rust
#[actix_rt::test]
pub async fn test_user_update_valid_user()
```

We start by creating a configuration closure with the routes we're interested in testing. Even
though the `user_insert` service is already part of the `setup_app!` macro, I left it there to make
it clear that we depend on this service for the test.

The macro invocation of `setup_app!` returns the running server (`app`), the `bearer_token` and
`cookies`, both of which will be inserted in the test's request headers.

The next invocation of `pre_insert_user!` is our shorthand for inserting a `User` into the database,
this is the `User` that we will be updating.

After all this setup, we're finally ready to create the `TestRequest` we're interested in, with the
help of
[`TestRequest::insert_header`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/test/struct.TestRequest.html#method.insert_header),
and
[`TestRequest::cookie`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/test/struct.TestRequest.html#method.cookie).

Finally, we just assert if the `ServiceResponse::status()` was successful. Some tests will compare
the [`StatusCode`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/http/struct.StatusCode.html)
directly against what we expect from the service, instead of if they were just successful, this is
to cover some services that respond with
[`StatusCode::FOUND`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/http/struct.StatusCode.html#associatedconstant.FOUND), or
[`StatusCode::NOT_MODIFIED`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/http/struct.StatusCode.html#associatedconstant.NOT_MODIFIED).

Most of the tests will look like this, except the ones that don't require authentication. I've left
the `test_user_logout` as an "expanded" test case, so it doesn't make use of macros.

## 7.5 Testing [tasks](integration-lib/tests/test_task_routes.rs)

These tests are structured in much the same way as the
[test_user_routes](integration-lib/tests/test_user_routes.rs) are. It comes with its own macro
`pre_insert_task` that inserts a `Task` into the database.

`test_task_insert_valid_task` was left as the expanded version, much like `test_user_logout` was,
and it covers the whole process of setting up `App`, a `LoggedUser`, and finally using the `/tasks`
insert service.

I don't feel that there is much to be gained by going over every test here, as they're using the
same features you already saw in [test_user_routes](integration-lib/tests/test_user_routes.rs). If
you feel that I should explain something here, please open up an issue!

## 7.6 Some notes on testing

The main issue I've run into when writing these tests was getting a `404` because I kept forgetting
to add the service to `ServiceConfig`, so if you get a `404`, check you've added the services you're
using (in the case of these projects, also check the macros), and check that the `TestRequest::uri`s
are correct.

We're not using any mocking library, we use a test database instead, this means that these tests may
not play well with concurrency, plus are limited with by the
[`PoolOptions::max_connections`](https://docs.rs/sqlx/0.5.5/sqlx/pool/struct.PoolOptions.html#method.max_connections).

I've been running these tests in single-threaded mode with:

```sh
# Runs every test in a single thread
cargo test -- --test-threads=1
```

## 7.7 The end (or is it?)

That's it so far, if you read this and have some suggestions / critics, please open up an issue!
