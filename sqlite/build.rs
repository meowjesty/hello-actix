use std::{env, fs::OpenOptions};

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = env::var("OUT_DIR").unwrap();
    let database_file = "sqlite_todo.db";
    let file = &format!("{}/{}", out_dir, database_file);

    {
        // NOTE(alex): Create a new file, or ignore the error if it already exists.
        let _ = OpenOptions::new().write(true).create_new(true).open(file);
    }

    println!("cargo:rustc-env=ADDRESS=127.0.0.1:8080");
    println!("cargo:rustc-env=DATABASE_URL=sqlite://{}", file);
}
