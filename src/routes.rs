use actix_web::{delete, get, post, put, web, Responder};
use anyhow::Result;
use sqlx::SqlitePool;

use crate::model::{InputTodo, Todo};

// TODO(alex): Figure out how to integrate anyhow error handling properly.
#[get("/")]
pub(crate) async fn index(pool: web::Data<SqlitePool>) -> Result<impl Responder> {
    let todos = Todo::find_ongoing(pool.get_ref()).await?;
    let response = serde_json::to_string_pretty(&todos)?;
    Ok(response)
}

#[get("/todos")]
pub(crate) async fn find_ongoing(pool: web::Data<SqlitePool>) -> impl Responder {
    let todos = Todo::find_ongoing(pool.get_ref()).await;
    let response = serde_json::to_string_pretty(&todos).unwrap();
    response
}

#[get("/todos/all")]
pub(crate) async fn find_all(pool: web::Data<SqlitePool>) -> impl Responder {
    let todos = Todo::find_all(pool.get_ref()).await;
    let response = serde_json::to_string_pretty(&todos).unwrap();
    response
}

#[get("/todos/{id}")] // <- define path parameters
pub(crate) async fn find_by_id(pool: web::Data<SqlitePool>, id: web::Path<i64>) -> impl Responder {
    let response = Todo::find_by_id(pool.get_ref(), *id).await;
    response
}

#[post("/todos")]
pub(crate) async fn create_todo(
    pool: web::Data<SqlitePool>,
    input: web::Json<InputTodo>,
) -> impl Responder {
    let response = Todo::create(pool.get_ref(), &input).await;
    response.to_string()
}

#[delete("/todos/{id}")]
pub(crate) async fn delete_todo(pool: web::Data<SqlitePool>, id: web::Path<i64>) -> impl Responder {
    let response = Todo::delete(pool.get_ref(), *id).await;
    response.to_string()
}

#[put("/todos/{id}")]
pub(crate) async fn update_todo(
    pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
    input: web::Json<InputTodo>,
) -> impl Responder {
    let response = Todo::update(pool.get_ref(), *id, &input).await;
    response.to_string()
}

#[post("/todos/{id}")]
pub(crate) async fn done_todo(pool: web::Data<SqlitePool>, id: web::Path<i64>) -> impl Responder {
    let response = Todo::done(pool.get_ref(), *id).await;
    response.to_string()
}

#[delete("/todos/undo/{id}")]
pub(crate) async fn undo_todo(pool: web::Data<SqlitePool>, id: web::Path<i64>) -> impl Responder {
    let response = Todo::undo(pool.get_ref(), *id).await;
    response.to_string()
}

// function that will be called on new Application to configure routes for this module
pub(crate) fn todo_service(cfg: &mut web::ServiceConfig) {
    cfg.service(find_ongoing);
    cfg.service(find_all);
    cfg.service(find_by_id);
    cfg.service(create_todo);
    cfg.service(delete_todo);
    cfg.service(update_todo);
    cfg.service(done_todo);
    cfg.service(undo_todo);
}
