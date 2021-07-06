# 4 Cookies

## 4.1 Cookies that we can't eat

The cookies project mainly introduces 2 new features:

1. [actix-session](https://github.com/actix/actix-extras/tree/master/actix-session), and
2. [`FromRequest`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/trait.FromRequest.html);

Most of the project remains the same, so this should be a quick chapter to read. Grab a cup of milk
and let's dip in!

## 4.2 Cookies middleware

 The `actix-session` crate has a ready to use
 [`CookieSession`](https://docs.rs/actix-session/0.5.0-beta.2/actix_session/struct.CookieSession.html)
 middleware, so you should know the drill. Let's jump into [main.rs](src/main.rs), and add a new
 `App::wrap` call.

 ```rust
wrap(CookieSession::signed(&[0; 32]).secure(false))
 ```

`CookieSession` has a limit of `4000 bytes`, and you'll get an error if you try going above it. The
[`signed`](https://docs.rs/actix-session/0.5.0-beta.2/actix_session/struct.CookieSession.html#method.signed)
call sets this cookie as plaintext to be stored on the client, with a signature, so it **cannot** be
modified by the client. The alternative is using
[`private`](https://docs.rs/actix-session/0.5.0-beta.2/actix_session/struct.CookieSession.html#method.private)
which encrypts the cookie, and cannot be viewed by the client.

We also call
[`secure`](https://docs.rs/actix-session/0.5.0-beta.2/actix_session/struct.CookieSession.html#method.secure)
setting it to `false`, meaning that we don't care about secure connections. If you set it to `true`,
then the cookie would travel only through HTTPS.

So, remember when I said we wouldn't be focusing on security, well this is just the first step on
our non-secure web server journey. We'll be sprinting through some features, before we go back to
use "best practices".

Now that we have `wrap`ped our `App` with the `CookieSession` middleware, if you have guessed that
accessing it means using an extractor, you would be right!

But, before we rush to [routes](src/routes.rs), let's first take a quick look at a new friend we
have in [models](src/models.rs).

## 4.3 Finally implementing `FromRequest` for our types

It's time to take off the mask from the extractors, to reveal who is behind all this ruckus.

Woah! It was `FromRequest` all along?!

Much like the `Responder` trait turns your things into `HttpResponse`s, the `FromRequest` trait
extracts your things from a
[`HttpRequest`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.HttpRequest.html).

This trait is quite a bit more complicated though, as you can see when we implement it for
`InsertTask`, and for `UpdateTask`. It expects you to fill in 3 types: `Config`, `Error`, and
the villainous `Future`.

### 4.3.1 The Config

We're not going to be using this one, in my futile attempt of keeping things as simple as possible,
but it still merits talking a bit about it.

The [`Config`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/trait.FromRequest.html#associatedtype.Config)
docs just say:

> Configuration for this extractor.

But what does this mean? What are we configuring exactly?

Well, the deal is, this is whatever configuration you want, actually. In our case we don't care
about setting anything, as we'll be taking advantage of the json infrastructure already built-in for
actix.

However, if you wanted, to limit the number of bytes that the request payload should have, or use
a custom error handler, or check that the request is of a certain content type, then you would a
struct such as `InsertTaskConfig`, for example, to do so. If you want a real example, then take a
look at [`JsonConfig`](https://docs.rs/actix-web/3.3.2/actix_web/web/struct.JsonConfig.html).

### 4.3.2 The Error

The error to be produced if our extraction process fail. That's it.

### 4.3.3 And the Future

Now this, partner, this one can't be tamed. To understand what is truly going on here, you need
another book. So, if you feel that you have to know the underpinnings of this type, I suggest you
do a little research on your own, there are some good posts on the internet, go get them, and come
back.

If you're more interested in going forward with actix, then I'll lend you my horse, already saddled
and ready to go.

Actix wants `async` everywhere, it lives on `async`, breathes `Pin`ned types, and eats `Future`s for
breakfast, so this trait wants to return a `Future`, but it doesn't know which (what is inside,
actually).

And so, to make matters simple, we use a
[`LocalBoxFuture`](https://docs.rs/futures/0.3.15/futures/future/type.LocalBoxFuture.html) to avoid
having to deal with the inner workings of the `Future` trait, and its requirements.

Inside this `LocalBoxFuture` will be the actual `Result<Self, Self::Error>` that we want when using
this extractor.

With this poor excuse of an explanation, we know have everything needed to implement the function.

### 4.3.4 `fn from_request`

```rust
fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future
```

The function signature gives a good hint of what is supposed to happen here. We use `req` to get
information out of the request, and `payload` contains the data we want to extract. Note that the
return is `Self::Future`, and inside is our `Result<T, E>`.

We're going to use
[`JsonBody`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/dev/enum.JsonBody.html) to keep our
extraction process simple, as it provides a nice
[`JsonBody::new`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/dev/enum.JsonBody.html#method.new)
function that does the heavy lifting of extracting a json from the request, payload pair.

We then `map` the result of `JsonBody::new` to be of our appropriate type
`Result<InsertTask, AppError>`, and finally, we call the new `InsertTask::validate` (or
`UpdateTask::validate`) function that checks if the input doesn't have an empty `title` field.

Lastly we use `boxed_local` to wrap our result in `Self::Future`.

This trait is a "bit" more complicated than the others we've seen so far, if you want to dip deeper,
I suggest you read the
[`Json<T>`](https://docs.rs/actix-web/4.0.0-beta.8/src/actix_web/types/json.rs.html#136-158)
implementation of `FromRequest`. There you'll see a `JsonConfig`, and a `JsonExtractFut` future. A
true behind the scenes for your learning.

Tired yet? Buckle up we're not finished, it's time to head into [routes](src/routes.rs).

## 4.4 Route to the cookies

I've promised you cookies, and I'll give you cookies! We have 2 new services being provided:

```rust
#[get("/tasks/favorite")]
async fn find_favorite(session: Session) -> Result<impl Responder, AppError>
```

Our first time seeing the
[`Session`](https://docs.rs/actix-session/0.5.0-beta.2/actix_session/struct.Session.html) extractor.
It's our way of using the `CookieSession` and accessing the cookies.

The `find_favorite` function just uses the
[`session::get::<T>`](https://docs.rs/actix-session/0.5.0-beta.2/actix_session/struct.Session.html#method.get)
 to find a cookie with the key `"favorite_task"`. In case there is none, we return the new
 `TaskError::NoneFavorite`.

```rust
#[get("/tasks/favorite/{id}")]
async fn favorite(db_pool: web::Data<SqlitePool>, session: Session, id: web::Path<i64>) -> Result<impl Responder, AppError>
```

This one is a bit more elaborate, as it first calls
[`session.remove`](https://docs.rs/actix-session/0.5.0-beta.2/actix_session/struct.Session.html#method.remove)
to, well, remove whatever `"favorite_task"` is stored in the cookie. It then goes into the database
searching for a `Task` if the `Task::id` is different (this function toggles a _favorite_), when a
`Task` is found we use
[`session.insert`](https://docs.rs/actix-session/0.5.0-beta.2/actix_session/struct.Session.html#method.insert)
to put the key (`&str`) value (`Task`) pair in the cookie.

The last change of notice appears in `find_by_id`:

```rust
#[get("/tasks/{id:\\d+}")]
async fn find_by_id(db_pool: web::Data<SqlitePool>, id: web::Path<i64>) -> Result<impl Responder, AppError>
```

The `/tasks/{id}` route changed to use a custom regex `d+` to match only digits. This was necessary
to avoid a route conflict with `/tasks/favorite`. If you recall, `{id}` is actually a match-all
regex, so there are 2 ways of solving this conflict:

1. Either write a specific regex (what we did);
2. Or set up the `cfg.service` in the appropriate order for the request extractors.

If you go with option 2, and try to do a request that contains something that can be extracted via
`Path<i64>`, then the route would match correctly.

## 4.5 To be continued

We've talked about using the `CookieSession` middleware to store cookies that are retrieved with the
`Session` extractor. Dipped our cookies into the `FromRequest` trait, and left some crumbles on the
`Future`.

Hopefully you still have an appetite, because up next we'll be using new types of cookies to handle
[login](../login/)! Leave it in the comments below, what's your favorite kind of cookie (mine is
chocolate, boring I know).
