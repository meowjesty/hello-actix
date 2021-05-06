use std::{self, fs::OpenOptions, io::BufReader};

use actix_web::{
    error, get, post,
    web::{self, Data},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder, Result,
};
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
    // NOTE(alex): `OpenOptions` puts the file pointer at the end, this means that overwritting
    // the file is wonky.
    // let file = OpenOptions::new().write(true).open(path).unwrap();
    let mut new_todos = data.todos.clone();
    new_todos.push(new_todo);
    let new_app_data = AppData {
        id_tracker: data.id_tracker + 1,
        todos: new_todos,
    };

    std::fs::write(path, serde_json::to_string(&new_app_data).unwrap()).unwrap();

    Ok(format!("{:?}", data))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Open the file in read-only mode with buffer.
    let path = std::path::Path::new("./todos.json");
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .read(true)
        .open(path)
        .unwrap();
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
