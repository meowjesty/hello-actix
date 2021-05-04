#![feature(try_find)]

use std::{
    fs::File,
    io::{BufReader, BufWriter},
    sync::{
        atomic::{AtomicU64, Ordering},
        Mutex,
    },
};

use actix_web::{
    error, get, post,
    web::{self, Data},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError, Result,
};
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AppData {
    id_tracker: u64,
    todos: Vec<Todo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Todo {
    id: u64,
    task: String,
    details: String,
}

// Responder
impl Responder for Todo {
    fn respond_to(self, req: &HttpRequest) -> HttpResponse {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        HttpResponse::Ok()
            .content_type("application/json")
            .body(body)
    }
}

#[get("/")]
async fn index(data: web::Data<AppData>) -> Result<HttpResponse, Error> {
    let response = HttpResponse::Ok().json(&data.todos);
    Ok(response)
}

#[get("/todos")]
async fn get_todos(data: web::Data<AppData>) -> Result<HttpResponse, Error> {
    let response = HttpResponse::Ok().json(&data.todos);
    Ok(response)
}

#[get("/todos/{id}")] // <- define path parameters
async fn todos_by_id(req: HttpRequest, id: web::Path<u64>) -> Result<String> {
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
async fn insert_todo(data: web::Data<AppData>, item: web::Json<InputTodo>) -> Result<String> {
    let new_todo = Todo {
        id: data.id_tracker,
        task: item.task.clone(),
        details: item.details.clone(),
    };

    let path = std::path::Path::new("./todos.json");
    let file = File::open(path).unwrap();
    let writer = BufWriter::new(file);
    let mut new_todos = data.todos.clone();
    new_todos.push(new_todo);
    let new_app_data = AppData {
        id_tracker: data.id_tracker + 1,
        todos: new_todos,
    };
    // TODO(alex) 2021-05-04: Figure out why this isn't writing to the file correctly (it doesn't
    // add data to the file, but changes its modified date).
    serde_json::to_writer(writer, &new_app_data)
        .map_err(|fail| error::ErrorPreconditionFailed(format!("{}", fail)))?;

    Ok(format!("{:?}", data))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Open the file in read-only mode with buffer.
    let path = std::path::Path::new("./todos.json");
    let file = File::open(path).unwrap();
    let reader = BufReader::new(file);

    // WARNING(alex): If you forget to type the result, some unit type will be figured out by rust
    // (incorrectely), even though the json structure is fine.
    let app_data: AppData = serde_json::from_reader(reader)?;
    println!("appdata {:#?}", app_data);

    HttpServer::new(move || {
        App::new()
            .data(app_data.clone())
            .service(index)
            .service(todos_by_id)
            .service(insert_todo)
            .service(get_todos)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
