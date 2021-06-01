use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use sqlx::SqlitePool;

use crate::{
    errors::TodoError,
    model::{InputTodo, Todo},
};

#[get("/")]
pub(crate) async fn index(pool: web::Data<SqlitePool>) -> Result<impl Responder, TodoError> {
    let todos = Todo::find_ongoing(pool.get_ref()).await?;
    let response = serde_json::to_string_pretty(&todos)?;
    Ok(response)
}

#[get("/todos")]
pub(crate) async fn find_ongoing(pool: web::Data<SqlitePool>) -> Result<impl Responder, TodoError> {
    let todos = Todo::find_ongoing(pool.get_ref()).await?;
    let response = serde_json::to_string_pretty(&todos)?;
    Ok(response)
}

#[get("/todos/all")]
pub(crate) async fn find_all(pool: web::Data<SqlitePool>) -> Result<impl Responder, TodoError> {
    let todos = Todo::find_all(pool.get_ref()).await?;
    let response = serde_json::to_string_pretty(&todos)?;
    Ok(response)
}

#[get("/todos/{id}")] // <- define path parameters
pub(crate) async fn find_by_id(
    pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, TodoError> {
    let response = Todo::find_by_id(pool.get_ref(), *id).await?;
    Ok(response)
}

/// NOTE(alex): There is no json validation, this will be an error if the fields passed don't match
/// what the struct has, but extra json fields are fine, and will be ignored.
#[post("/todos")]
pub(crate) async fn create_todo(
    pool: web::Data<SqlitePool>,
    input: web::Json<InputTodo>,
) -> Result<impl Responder, TodoError> {
    let created_id = Todo::create(pool.get_ref(), &input).await?;
    Ok(created_id.to_string())
}

#[delete("/todos/{id}")]
pub(crate) async fn delete_todo(
    pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, TodoError> {
    let num_modified = Todo::delete(pool.get_ref(), *id).await?;

    if num_modified == 0 {
        Ok(HttpResponse::NotModified().finish())
    } else {
        // TODO(alex) [high] 2021-06-01: This won't work, doesn't `u64` implement `Responder`?
        // Ok(num_modified.to_string())
        Ok(HttpResponse::Ok().body(num_modified.to_string()))
    }
}

#[put("/todos/{id}")]
pub(crate) async fn update_todo(
    pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
    input: web::Json<InputTodo>,
) -> Result<impl Responder, TodoError> {
    let num_modified = Todo::update(pool.get_ref(), *id, &input).await?;

    if num_modified == 0 {
        Ok(HttpResponse::NotModified().finish())
    } else {
        Ok(HttpResponse::Ok().body(num_modified.to_string()))
    }
}

#[post("/todos/{id}")]
pub(crate) async fn done_todo(
    pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, TodoError> {
    let created_id = Todo::done(pool.get_ref(), *id).await?;

    if created_id == 0 {
        Ok(HttpResponse::NotModified().finish())
    } else {
        Ok(HttpResponse::Ok().body(created_id.to_string()))
    }
}

#[delete("/todos/undo/{id}")]
pub(crate) async fn undo_todo(
    pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, TodoError> {
    let num_modified = Todo::undo(pool.get_ref(), *id).await?;

    if num_modified == 0 {
        Ok(HttpResponse::NotModified().finish())
    } else {
        Ok(HttpResponse::Ok().body(num_modified.to_string()))
    }
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
