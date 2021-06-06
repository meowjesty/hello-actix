use std::{env, fs::OpenOptions};

#[cfg(not(test))]
const DATABASE_FILENAME: &'static str = "sqlite_todo.db";

#[cfg(test)]
const DATABASE_FILENAME: &'static str = "sqlite_todo.test.db";

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=queries/create_database.sql");

    println!("DATABASE_FILENAME is {}", DATABASE_FILENAME);

    let out_dir = env::var("OUT_DIR").unwrap();
    let database_file = &format!("{}/{}", out_dir, DATABASE_FILENAME);
    let database_url = &format!("sqlite://{}", database_file);

    {
        // NOTE(alex): Create a new file, or ignore the error if it already exists.
        let _ = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(database_file);
    }

    // NOTE(alex): Quick and dirty way of checking if we should re-create the database.
    println!("cargo:rustc-env=CREATE_DATABASE=1");

    println!("cargo:rustc-env=ADDRESS=127.0.0.1:8080");
    println!("cargo:rustc-env=DATABASE_URL={}", database_url);
}
