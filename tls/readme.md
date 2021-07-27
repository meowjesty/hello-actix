# 8 TLS

## 8.1 You can't spell HTTPS without TLS

We're finally saying goodbye to our old pal `HTTP`, and entering the new era with `HTTPS`, and to
make this work we'll need a few things:

1. A [TLS](https://developer.mozilla.org/en-US/docs/Web/Security/Transport_Layer_Security)
   certificate, and;
2. Setting up our actix-web server to use said certificate;

Creating and validating a certificate is out of scope for this project, but I'll give you some
pointers:

- [mkcert](https://github.com/FiloSottile/mkcert) is a tool that creates locally trusted
  certificates;
- [openssl](https://letsencrypt.org/docs/certificates-for-localhost/#making-and-trusting-your-own-certificates)
  may also be used to generate a certificate, but requires you to manually handle the _trust_ setup;
- You'll find many online tools that create a _cert_, and _key_ pair, any of these approaches will
  work just fine for learning purposes;

Now that you have a certificate, you can add it to the trusted list on your OS, browser, API testing
tool, or you can just set whatever tool you're using to skip validation (this is the approach I'm
taking here).

This project already contains a [cert](/tls/tls-lib/certificates/cert.pem) and
[key](/tls/tls-lib/certificates/key.pem) files, I **strongly** suggest
not sharing your private certificates! Remember that we're skimping on security here!

## 8.2 A new dependency enters the ring: `rustls`

We'll be using [rustls](https://github.com/ctz/rustls) to handle the TLS configuration, as it pairs
nicely with actix-web.

```rust
pub fn setup_tls() -> Result<rustls::ServerConfig, rustls::TLSError>
```

This function will load and parse our certificate into the type that actix-web wants.

We use the [`rustls::ServerConfig`](https://docs.rs/rustls/0.19.1/rustls/struct.ServerConfig.html)
returned, and pass it into
[`HttpServer::bind_rustls`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.HttpServer.html#method.bind_rustls).
Previously, we were just using
[`HttpServer::bind`](https://docs.rs/actix-web/4.0.0-beta.8/actix_web/struct.HttpServer.html#method.bind)
to bind an address, but now we bind the address, and the TLS configuration.

## 8.3 🔒

With these simple steps out of the way, we now have our server running on HTTPS.
