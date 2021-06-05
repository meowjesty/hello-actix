use std::{fs::OpenOptions, net::SocketAddr};

use actix_web::{App, HttpServer};
use log::info;
use model::Todo;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqlitePoolOptions;

use crate::routes::{index, todo_service};

mod errors;
mod model;
mod routes;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Config {
    address: SocketAddr,
    database: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config_file = include_bytes!("./../config.json");
    let config: Config = serde_json::from_slice(config_file)?;

    // std::env::set_var("DATABASE_URL", config.database);
    {
        let _ = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&config.database.split("//").last().unwrap());
    }

    // Create a connection pool
    //  for MySQL, use MySqlPoolOptions::new()
    //  for SQLite, use SqlitePoolOptions::new()
    //  etc.
    // WARNING(alex): This fails if there is no `databases/todo.db` file.
    let database_pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&config.database)
        .await
        .unwrap();

    Todo::create_database(&database_pool).await.unwrap();

    let server = HttpServer::new(move || {
        App::new()
            .data(database_pool.clone())
            .service(index)
            .configure(todo_service)
        // WARNING(alex): Matching order matters, if `hello_scope` is put after `todos`, then it
        // won't match, and returns 404.
        // .service(hello_scope)
        // WARNING(alex): No compilation error on registering a service twice!
        // .service(read_todos_by_id)
    })
    .bind(config.address)?;

    info!("Starting server!");
    server.run().await?;

    Ok(())
}
