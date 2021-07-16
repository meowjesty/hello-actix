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

TODO(alex) [mid] 2021-07-15: Write about the changes:

- Split project into bin and lib for test purposes;
- Dive deeper on testing:
  - Talk a bit about the macro;
  - Mention single-threaded only rule for tests;
    - `test::read_*` functions as a way to extract data;
    - If you get 404, check that every required service was added to the test;
