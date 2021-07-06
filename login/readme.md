# 5 Login

## 5.1 Project facelift

The project comes with a new folder structure and a couple of new modules to keep things properly
separated. With the introduction of the `User` concept, in order to avoid a mix of
[users](src/users.rs) and [tasks](src/tasks.rs) routes, and model definitions, we're separating
them into their own modules.

- The [tasks](src/tasks.rs) module has the code we've been working with so far;
- while the [users](src/users.rs) module looks about the same, but contains `User` related code;

I've moved the `include_str!` for SQL constants to the module file to shorten the
[tasks::models](src/tasks/models.rs) file (same thing is done for `User`
[users::models](src/users/models.rs)).

Remember to use `App::configure` to set up the [users](src/users.rs) services, much like
we did with [tasks](src/tasks.rs), but this time passing it `user_service`.

To handle login / logout we'll be using the
[`actix-identity`](https://github.com/actix/actix-extras/tree/master/actix-identity) crate.

## 5.2 Identity middleware

At this point you're probably very familiar with how to set up a middleware in actix. We first call
`App::wrap` and pass it the new
[`IdentityService`](https://docs.rs/actix-identity/0.4.0-beta.2/actix_identity/struct.IdentityService.html)
middleware, that we'll be later accessing via some extractor, which for `IdentityService` is called
[`Identity`](https://docs.rs/actix-identity/0.4.0-beta.2/actix_identity/struct.Identity.html).

`IdentityService` creation is a lot like `CookieSession`, we give it a name for the cookie, set an
expiration time
([`login_deadline`](https://docs.rs/actix-identity/0.4.0-beta.2/actix_identity/struct.CookieIdentityPolicy.html#method.login_deadline)), and tell if it should be `secure` or not.

That's all we'll be using to handle our `User` login.

## 5.3 The new [users](src/users.rs) module

A brief introduction:

- [users::errors](src/users/errors.rs) contains the new `UserError` enum;
- [users::models](src/users/models.rs) has all the functions you've seen on
  [tasks::models](src/tasks/models.rs) before, plus the `FromRequest` and `Responder` traits
  implementations;
- [users::routes](src/users/routes.rs) also has much of the same looking code;

### 5.3.1 The `LoginUser` struct

```rust
// TODO(alex) 2021-07-06:
// - New `users` module;
// - IdentityService cookies to handle login
```
