use std::{self, fs::OpenOptions, io::BufReader};

use actix_web::{
    error, get, post, put,
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
        let body = serde_json::to_string_pretty(&self).unwrap();

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
async fn read_todos(data: web::Data<AppData>) -> Result<HttpResponse, Error> {
    let response = HttpResponse::Ok().json(&data.todos);
    Ok(response)
}

#[get("/todos/{id}")] // <- define path parameters
async fn read_todos_by_id(req: HttpRequest, id: web::Path<u64>) -> Result<String> {
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
    let path = std::path::Path::new("./todos.json");
    // TODO(alex) 2021-05-06: Create the file with initial data if it doesn't exist.
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
            .data(app_data.clone())
            .service(index)
            .service(hello_scope)
            .service(todos_scope)
        // WARNING(alex): Matching order matters, if `hello_scope` is put after `todos`, then it
        // won't match, and returns 404.
        // .service(hello_scope)
        // WARNING(alex): No compilation error on registering a service twice!
        // .service(read_todos_by_id)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
