# 6 Authorization

## 6.1 The tour that never ends

We've been through a lot of `actix-web` and `actix-extras` so far. Hopefully you've learned enough
tricks to pull out some cool actix maneuvers. I would say that the biggest thing lacking is route
protection, and that's what we'll be tackling here.

## 6.2 Ladies and gentlemen, I present to you `HttpAuthentication`

The [actix-web-httpauth](https://github.com/actix/actix-extras/tree/master/actix-web-httpauth) crate
provides us with the
[`HttpAuthentication`](https://docs.rs/actix-web-httpauth/latest/actix_web_httpauth/middleware/struct.HttpAuthentication.html)
middleware. We'll be using its
[`HttpAuthentication::bearer`](https://docs.rs/actix-web-httpauth/latest/actix_web_httpauth/middleware/struct.HttpAuthentication.html#method.bearer)
version.

The [_Bearer_](https://datatracker.ietf.org/doc/html/rfc6750) authorization uses an access token to
grant access. This token is part of our request header, so requests to services that use the
`HttpAuthentication` middleware must have `Authorization: Bearer [token]` in their header.

So far we've been wrapping the whole `App` with every middleware, and we could do the same for
`HttpAuthentication`, but I want some routes to be unprotected (`users/login`, `users/register`,
and `GET` services in general), so we'll be wrapping individual services in the middleware instead.

### 6.2.1 Wrapping

There are a few ways of using `wrap`:

- [`App::wrap`](https://docs.rs/actix-web/latest/actix_web/struct.App.html#method.wrap) as
  we've been doing, it registers the middleware for the whole `App`;
- [`Resource::wrap`](https://docs.rs/actix-web/latest/actix_web/struct.Resource.html#method.wrap)
  this wraps a specific
  [`Resource`](https://docs.rs/actix-web/latest/actix_web/struct.Resource.html) only (we'll
  be using this version to protect some services, but in its macro form);
- [`Scope::wrap`](https://docs.rs/actix-web/latest/actix_web/struct.Scope.html#method.wrap)
  which protects by [`Scope`](https://docs.rs/actix-web/latest/actix_web/struct.Scope.html)
  (a way of grouping services under a singular scope);

The `App::wrap` version we've been using would require us to send requests with an authorization
header for every service, as `HttpAuthentication` will forbid access without it, and there is no way
to open exceptions. Every request without this header would return a forbidden response, and we have
no control over it.

The other 2 are more malleable ways of setting middleware, as they don't wrap the whole application.
We're not using `Scope` in any of these projects, for no particular reason, other than we have few
routes, so going the individual service routes ends up being more explicit and not cumbersome enough
to justify using those.

### 6.2.2 Macro wrap

Service macros (`get`, `put`, `post`, ...) have a `wrap` attribute that does the wrapping for a
`Resource`:

```rust
// This form could've been used to individually set up any middleware for specific services,
// it's not just an authentication thing.
#[post("/tasks", wrap = "HttpAuthentication::bearer(validator)")]
async fn insert(...) -> Result<impl Responder, AppError>
```

We set up `HttpAuthentication` with the `bearer` function, and pass into it a function that does
the validation (keep in mind that this function won't even be called if there is no authorization
header). This is the function that will handle our authorization process.

```rust
async fn validator(req: ServiceRequest, credentials: BearerAuth) -> Result<ServiceRequest, Error>
```

`HttpAuthentication` middleware provides 2 forms of authentication, `BasicAuth` and `BearerAuth`, so
this function signature must match the kind you want.
The [`BearerAuth`](https://docs.rs/actix-web-httpauth/latest/actix_web_httpauth/extractors/bearer/struct.BearerAuth.html)
extractor gives us with a way to check the
[`token`](https://docs.rs/actix-web-httpauth/latest/actix_web_httpauth/extractors/bearer/struct.BearerAuth.html#method.token)
provided by the request header, without us having to manually dig into the `req` parameter.

This middleware function will be called pre-services, so it must return a
[`ServiceRequest`](https://docs.rs/actix-web/latest/actix_web/dev/struct.ServiceRequest.html)
that will be passed onwards. Note that this is not a `HttpRequest`, but a `ServiceRequest`, the
difference being that `ServiceRequest` gives **mutable** access to the request contents, so we may
alter the request before it's passed down to our services.

Our [`validator`](src/main.rs) function uses the `IdentityService` cookie embedded in the `req`uest
to check if our user is currently logged in by matching the `credentials.token()` with the
`logged_user.token`. If they match we just pass the `req`uest forward, otherwise we return some
error.

## 6.3 Our use of `HtppAuthentication`

We'll be protecting services that change the database (`POST`, `PUT`, `DELETE`), and allowing access
to any `GET` request without login. There are only 2 `POST` services allowed, `/users/register` and
`/users/login`, as we don't start the database with any user pre-registered, and logging-in has to
be accessible.

Protected services will require requests to contain a `Authorization: Bearer token`, and for the
user to be previously logged in, which generates a `LoggedUser` with a token, stored in the
`auth-cookie`.

Trying to reach any other service without these conditions will give you a nice
`ErrorUnauthorized(..)` of some sort. So you cannot insert, or delete a `Task` without being logged
in first.

Up until now we were just using `Identity::remember` to save the user in the `auth-cookie`, but now
`login` must have a way of generating an authorization token.
Our token creation is just a [`Hash`](https://doc.rust-lang.org/std/hash/trait.Hash.html) of the
user, not a secure way of handling things, but it's simple and good enough for this particular
project.

## 6.4 Next up

Things stayed relatively the same in this one, `HttpAuthentication` came in as an add-on, with the
biggest changes being token generation in `login`, and `HttpAuthentication::bearer` being sprinkled
in services that we want to protect.

The next project, [integration](../integration/), contains another "project" re-structuring to allow
tests and services to live in different files. We'll be separating it into a `lib` project and a
`bin` that uses it.
