use std::{
    self,
    fs::{File, OpenOptions},
    io::BufReader,
    net::{SocketAddr, ToSocketAddrs},
};

use actix_web::{
    error, get, post, put,
    web::{self, Data},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
use log::info;
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, FromRow, SqlitePool};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Config {
    address: SocketAddr,
    database: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AppData {
    id_tracker: u64,
    todos: Vec<Todo>,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
struct Todo {
    id: i64,
    task: String,
    details: String,
}

impl Todo {
    const CREATE_DATABASE: &'static str =
        include_str!("./../databases/queries/create_database.sql");
    const FIND_ALL: &'static str = include_str!("./../databases/queries/find_all.sql");
    const FIND_BY_ID: &'static str = include_str!("./../databases/queries/find_by_id.sql");
    const INSERT: &'static str = include_str!("./../databases/queries/insert.sql");

    pub async fn create_database(pool: &SqlitePool) -> i64 {
        let mut connection = pool.acquire().await.unwrap();

        sqlx::query(Self::CREATE_DATABASE)
            .execute(&mut connection)
            .await
            .unwrap()
            .last_insert_rowid()
    }

    pub async fn find_all(pool: &SqlitePool) -> impl Responder {
        let todos: Vec<Todo> = sqlx::query_as(Self::FIND_ALL)
            .fetch_all(pool)
            .await
            .unwrap();

        let body = serde_json::to_string_pretty(&todos).unwrap();
        // Create response and set content type
        HttpResponse::Ok()
            .content_type("application/json")
            .body(body)
    }

    pub async fn find_by_id(pool: &SqlitePool, id: i64) -> impl Responder {
        let todos: Option<Todo> = sqlx::query_as(Self::FIND_BY_ID)
            .bind(id)
            .fetch_optional(pool)
            .await
            .unwrap();

        let body = serde_json::to_string_pretty(&todos).unwrap();
        // Create response and set content type
        HttpResponse::Ok()
            .content_type("application/json")
            .body(body)
    }

    pub async fn create(pool: &SqlitePool, input: &InputTodo) -> i64 {
        let mut connection = pool.acquire().await.unwrap();
        sqlx::query(Self::INSERT)
            .bind(&input.task)
            .bind(&input.details)
            .execute(&mut connection)
            .await
            .unwrap()
            .last_insert_rowid()
    }
}

// Responder
impl Responder for Todo {
    fn respond_to(self, req: &HttpRequest) -> HttpResponse {
        let body = serde_json::to_string_pretty(&self).unwrap();

        // Create response and set content type
        HttpResponse::Ok()
            .content_type("application/json")
            .body(body)
    }
}

#[get("/")]
async fn index(pool: web::Data<SqlitePool>) -> impl Responder {
    let response = Todo::find_all(pool.get_ref()).await;
    response
}

#[get("/todos")]
async fn find_all(pool: web::Data<SqlitePool>) -> impl Responder {
    let response = Todo::find_all(pool.get_ref()).await;
    response
}

#[get("/todos/{id}")] // <- define path parameters
async fn find_by_id(pool: web::Data<SqlitePool>, id: web::Path<i64>) -> impl Responder {
    let response = Todo::find_by_id(pool.get_ref(), *id).await;
    response
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InputTodo {
    task: String,
    details: String,
}

#[post("/todos")]
async fn create_todo(pool: web::Data<SqlitePool>, item: web::Json<InputTodo>) -> impl Responder {
    let response = Todo::create(pool.get_ref(), &item).await;
    response.to_string()
}

#[post("/todos/{id}")]
async fn delete_todo(pool: web::Data<SqlitePool>, id: web::Path<u64>) -> impl Responder {
    todo!();
    let response = Todo::find_all(pool.get_ref()).await;
    response
}

#[put("/todos/{id}")]
async fn update_todo(
    pool: web::Data<SqlitePool>,
    id: web::Path<u64>,
    item: web::Json<InputTodo>,
) -> impl Responder {
    todo!();
    let response = Todo::find_all(pool.get_ref()).await;
    response
}

#[get("/hello")]
async fn hello() -> Result<String> {
    Ok(format!("Hello from api!"))
}

#[get("/hello/{id}")]
async fn hello_id(id: web::Path<u64>) -> Result<String> {
    Ok(format!("Hello from api {}!", id))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let config_file = include_bytes!("./../config.json");
    let config: Config = serde_json::from_slice(config_file)?;

    // std::env::set_var("DATABASE_URL", config.database);

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

    Todo::create_database(&database_pool).await;

    let server = HttpServer::new(move || {
        // NOTE(alex): Scopes are a little messy, what are the actual benefits? For such a simple
        // example I can't see any, but maybe as the project grows, who knows...
        let hello_scope = web::scope("/api").service(hello).service(hello_id);

        // NOTE(alex): `scope` expands into `/(service)`.
        let todos_scope = web::scope("/")
            .service(find_all)
            .service(find_by_id)
            .service(create_todo)
            .service(delete_todo)
            .service(update_todo);

        App::new()
            .data(database_pool.clone())
            .service(index)
            .service(hello_scope)
            .service(todos_scope)
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
