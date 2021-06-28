use std::env;

#[cfg(not(test))]
const DATABASE_FILENAME: &'static str = concat!(env!("CARGO_PKG_NAME"), ".db");

#[cfg(test)]
const DATABASE_FILENAME: &'static str = concat!(env!("CARGO_PKG_NAME"), ".test.db");

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=queries/create_database.sql");

    let out_dir = env::var("OUT_DIR").unwrap();
    let database_file = &format!("{}/{}", out_dir, DATABASE_FILENAME);

    // NOTE(alex): Quick and dirty way of checking if we should re-create the database.
    println!("cargo:rustc-env=CREATE_DATABASE=1");

    println!("cargo:rustc-env=DATABASE_FILE={}", database_file);
    println!("cargo:rustc-env=ADDRESS=127.0.0.1:8080");
    println!("cargo:rustc-env=RUST_LOG=info");
}
