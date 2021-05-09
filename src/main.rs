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
    pub async fn find_all(pool: &SqlitePool) -> impl Responder {
        let todos: Vec<Todo> = sqlx::query_as(
            r#"
            select * from OngoingTodo
        "#,
        )
        .fetch_all(pool)
        .await
        .unwrap();

        let body = serde_json::to_string_pretty(&todos).unwrap();
        // Create response and set content type
        HttpResponse::Ok()
            .content_type("application/json")
            .body(body)
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
async fn index(pool: web::Data<SqlitePool>) -> Result<HttpResponse, Error> {
    let response = HttpResponse::Ok().json(&pool.todos);
    Ok(response)
}

#[get("/todos")]
async fn read_todos(pool: web::Data<SqlitePool>) -> Result<HttpResponse, Error> {
    let response = HttpResponse::Ok().json(&data.todos);
    Ok(response)
}

#[get("/todos/{id}")] // <- define path parameters
async fn read_todos_by_id(pool: web::Data<SqlitePool>, id: web::Path<u64>) -> Result<String> {
    let data = req.app_data::<Data<AppData>>().unwrap();
    let todo = data
        .todos
        .iter()
        .find(|todo| todo.id == *id)
        .ok_or(error::ErrorNotFound(format!("No todo with id {:#?}", id)))?;

    Ok(format!("Todo {:#?}", todo))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InputTodo {
    task: String,
    details: String,
}

#[post("/todos")]
async fn create_todo(req: HttpRequest, item: web::Json<InputTodo>) -> Result<String> {
    let data = req.app_data::<Data<AppData>>().unwrap();
    let new_todo = Todo {
        id: data.id_tracker,
        task: item.task.clone(),
        details: item.details.clone(),
    };

    let path = std::path::Path::new("./todos.json");
    // NOTE(alex): `OpenOptions` puts the file pointer at the end, this means that overwritting
    // the file is wonky.
    // let file = OpenOptions::new().write(true).open(path).unwrap();
    let mut new_todos = data.todos.clone();
    new_todos.push(new_todo);
    let new_app_data = AppData {
        id_tracker: data.id_tracker + 1,
        todos: new_todos,
    };

    std::fs::write(path, serde_json::to_string_pretty(&new_app_data).unwrap()).unwrap();

    Ok(format!("{:?}", data))
}

#[post("/todos/{id}")]
async fn delete_todo(req: HttpRequest, id: web::Path<u64>) -> Result<String> {
    let data = req.app_data::<Data<AppData>>().unwrap();
    // let todos = data
    //     .todos
    //     .clone()
    //     .into_iter()
    //     .filter(|todo| todo.id != *id)
    //     .collect::<Vec<_>>();
    // let deleted = data.todos.len() - todos.len();

    // let new_app_data = AppData {
    //     id_tracker: data.id_tracker,
    //     todos,
    // };

    // let path = std::path::Path::new("./todos.json");
    // std::fs::write(path, serde_json::to_string(&new_app_data).unwrap()).unwrap();
    // Ok(format!("deleted {:?}", deleted))

    match data
        .todos
        .iter()
        .enumerate()
        .find(|(_, todo)| todo.id == *id)
    {
        Some((i, todo)) => {
            let mut new_todos = data.todos.clone();
            new_todos.remove(i);

            let new_app_data = AppData {
                id_tracker: data.id_tracker,
                todos: new_todos,
            };

            let path = std::path::Path::new("./todos.json");
            std::fs::write(path, serde_json::to_string_pretty(&new_app_data).unwrap()).unwrap();
            Ok(format!("{:?}", todo))
        }
        None => Err(error::ErrorNotFound(format!("No todo with id {:#?}", id))),
    }
}

#[put("/todos/{id}")]
async fn update_todo(
    req: HttpRequest,
    id: web::Path<u64>,
    item: web::Json<InputTodo>,
) -> Result<String> {
    let data = req.app_data::<Data<AppData>>().unwrap();
    match data
        .todos
        .iter()
        .enumerate()
        .find(|(_, todo)| todo.id == *id)
    {
        Some((i, todo)) => {
            let mut new_todos = data.todos.clone();
            new_todos.insert(
                i,
                Todo {
                    id: todo.id,
                    task: item.task.to_string(),
                    details: item.details.to_string(),
                },
            );
            new_todos.remove(i + 1);

            let new_app_data = AppData {
                id_tracker: data.id_tracker,
                todos: new_todos,
            };
            let path = std::path::Path::new("./todos.json");
            std::fs::write(path, serde_json::to_string_pretty(&new_app_data).unwrap()).unwrap();
            Ok(format!("{:?}", todo))
        }
        None => Err(error::ErrorNotFound(format!("No todo with id {:#?}", id))),
    }
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

    std::env::set_var("DATABASE_URL", config.database);

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

    let server = HttpServer::new(move || {
        // NOTE(alex): Scopes are a little messy, what are the actual benefits? For such a simple
        // example I can't see any, but maybe as the project grows, who knows...
        let hello_scope = web::scope("/api").service(hello).service(hello_id);

        // NOTE(alex): `scope` expands into `/(service)`.
        let todos_scope = web::scope("/")
            .service(read_todos_by_id)
            .service(create_todo)
            .service(read_todos)
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
