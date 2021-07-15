# 6 Authorization

TODO(alex) [mid] 2021-07-13: This project changed quite a bit with the use of Bearer authentication,
it requires a decent rewrite.

## 6.1 The tour that never ends

We've been through a lot of `actix-web` and `actix-extras` so far. Hopefully you've learned enough
tricks to pull out some cool actix maneuvers. I would say that the biggest thing lacking is route
protection, and that's what we'll be tackling here.

## 6.2 Ladies and gentlemen, I present to you `HttpAuthentication`

The [actix-web-httpauth](https://github.com/actix/actix-extras/tree/master/actix-web-httpauth) crate
provides us with the
[`HttpAuthentication`](https://docs.rs/actix-web-httpauth/0.6.0-beta.2/actix_web_httpauth/middleware/struct.HttpAuthentication.html#)
middleware. We'll be using its
[`HttpAuthentication::basic`](https://docs.rs/actix-web-httpauth/0.6.0-beta.2/actix_web_httpauth/middleware/struct.HttpAuthentication.html#method.basic)
version.

Now our requests must include an HTTP
[`Authorization`](https://developer.mozilla.org/en-US/docs/Web/HTTP/Headers/Authorization) header.
This is required by every single service we're providing, because we're wrapping the whole `App`
with the `HttpAuthentication` middleware. There is a way to wrap only specific services if you want
that, just use the
[`wrap`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/attr.post.html) attribute on the service:

```rust
// Protect only index
#[get("/", wrap = "HttpAuthentication::basic(validator)")]
async fn index() -> Result<impl Responder, AppError>
```

This can be done for any middleware, not only `HttpAuthentication`.

## 6.3 Our use of `HtppAuthentication`

`HttpAuthentication::basic` expects us to give it a function that will be used to do the validation.

```rust
async fn validator(req: ServiceRequest, _credentials: BasicAuth) -> Result<ServiceRequest, Error>
```

Pay close attention that we're dealing with
[`ServiceRequest`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/dev/struct.ServiceRequest.html)
here, and not our familiar `HttpRequest`. The difference between the two is that `ServiceRequest`
gives **mutable** access to the request contents, so we may alter the request before it's passed
down to our services.

[`BasicAuth`](https://docs.rs/actix-web-httpauth/0.6.0-beta.2/actix_web_httpauth/extractors/basic/struct.BasicAuth.html)
is the extractor for this middleware, and may be used to look up the `Authorization` part of the
request header. We're not using it though.

The `validator` function will only allow requests that:

- are `GET` requests, we don't protect information search at all;
- have either `"login"`, or `"register"` in their path, so that we may insert a new user, and login;
- have a `"auth-cookie"` cookie, this is generated after the user logs in;

Trying to reach any other service without these conditions will give you a nice
`ErrorUnauthorized(UserError::NotLoggedIn)`. So you cannot insert, or delete a `Task` without being
logged in first.

## 6.4 And ... cut

That's it so far.

Thank you for reading, hoping it helped you in some way!
