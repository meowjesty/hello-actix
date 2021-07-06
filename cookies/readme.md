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



```rust
// TODO(alex) 2021-07-06:
// - FromRequest;
// - CookieSession;
// - #[get("/tasks/{id:\\d+}")] regex to avoid matching #[get("/tasks/favorite")];
```
