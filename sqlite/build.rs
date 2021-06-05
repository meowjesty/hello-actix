use std::{env, fs::OpenOptions};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=queries/create_database.sql");

    let out_dir = env::var("OUT_DIR").unwrap();
    let database_filename = "sqlite_todo.db";
    let database_file = &format!("{}/{}", out_dir, database_filename);
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
