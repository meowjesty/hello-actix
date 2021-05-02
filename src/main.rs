#![feature(try_find)]

use std::{fs::File, io::BufReader};

use actix_web::{
    error, get, post,
    web::{self, Data},
    App, Error, HttpRequest, HttpResponse, HttpServer, Responder, ResponseError, Result,
};
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
struct AppData {
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
    type Error = Error;
    type Future = Ready<Result<HttpResponse, Error>>;

    fn respond_to(self, _req: &HttpRequest) -> Self::Future {
        let body = serde_json::to_string(&self).unwrap();

        // Create response and set content type
        ready(Ok(HttpResponse::Ok()
            .content_type("application/json")
            .body(body)))
    }
}

#[get("/")]
async fn index(data: web::Data<AppData>) -> Result<HttpResponse, Error> {
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
        .ok_or_else(|| error::ErrorBadRequest(format!("No todo with id {:?}", id)))?;

    Ok(format!("Todo {:#?}", todo))
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
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
