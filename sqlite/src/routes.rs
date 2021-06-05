use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use sqlx::SqlitePool;

use crate::{
    errors::{AppError, TaskError},
    models::{InsertTask, QueryTask, Task, UpdateTask},
};

#[post("/tasks")]
async fn insert(
    db_pool: web::Data<SqlitePool>,
    input: web::Json<InsertTask>,
) -> Result<impl Responder, AppError> {
    if input.non_empty_title.trim().is_empty() {
        return Err(TaskError::EmptyTitle.into());
    }

    let created_id = input.insert(db_pool.get_ref()).await?;
    Ok(created_id.to_string())
}

#[put("/tasks")]
async fn update(
    db_pool: web::Data<SqlitePool>,
    input: web::Json<UpdateTask>,
) -> Result<HttpResponse, AppError> {
    if input.new_title.trim().is_empty() {
        return Err(TaskError::EmptyTitle.into());
    }

    let num_modified = input.update(db_pool.get_ref()).await?;

    if num_modified == 0 {
        Ok(HttpResponse::NotModified().finish())
    } else {
        Ok(HttpResponse::Ok().body(num_modified.to_string()))
    }
}

#[delete("/tasks/{id}")]
async fn delete(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let num_modified = Task::delete(db_pool.get_ref(), *id).await?;

    if num_modified == 0 {
        Ok(HttpResponse::NotModified().finish())
    } else {
        // NOTE(alex): This doesn't work, rust expects it to be a `HttpResponse`, but we pass a
        // string, and the type won´t check.
        // Ok(num_modified.to_string())
        Ok(HttpResponse::Ok().body(num_modified.to_string()))
    }
}

#[post("/tasks/{id}/done")]
async fn done(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let created_id = Task::done(db_pool.get_ref(), *id).await?;

    if created_id == 0 {
        Ok(HttpResponse::NotModified().finish())
    } else {
        Ok(HttpResponse::Ok().body(created_id.to_string()))
    }
}

#[delete("/tasks/{id}/undo")]
async fn undo(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let num_modified = Task::undo(db_pool.get_ref(), *id).await?;

    if num_modified == 0 {
        Ok(HttpResponse::NotModified().finish())
    } else {
        Ok(HttpResponse::Ok().body(num_modified.to_string()))
    }
}

#[get("/tasks")]
async fn find_all(db_pool: web::Data<SqlitePool>) -> Result<impl Responder, AppError> {
    let tasks = Task::find_all(db_pool.get_ref()).await?;

    // NOTE(alex): For times when we want to convert `Vec<T>` into an `impl Responder`, this
    // requires creating a new wrapper type such as `struct TList(Vec<T>)`, otherwise rust doesn't
    // allow implementing a trait `Responder` for a type not defined in this crate.
    // Bonus: implement `Deref` to avoid having `tlist.0.push` throughout your code.
    //
    // I'll be taking the `to_string` route here to avoid adding more types, at least for now.
    let response = serde_json::to_string_pretty(&tasks)?;
    Ok(response)
}

#[get("/tasks/ongoing")]
async fn find_ongoing(db_pool: web::Data<SqlitePool>) -> Result<impl Responder, AppError> {
    let tasks = Task::find_ongoing(db_pool.get_ref()).await?;
    let response = serde_json::to_string_pretty(&tasks)?;
    Ok(response)
}

#[get("/tasks")]
async fn find_by_pattern(
    db_pool: web::Data<SqlitePool>,
    pattern: web::Query<QueryTask>,
) -> Result<impl Responder, AppError> {
    let tasks = Task::find_by_pattern(db_pool.get_ref(), &format!("%{}%", pattern.title)).await?;
    let response = serde_json::to_string_pretty(&tasks)?;
    Ok(response)
}

#[get("/tasks/{id}")]
async fn find_by_id(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let task = Task::find_by_id(db_pool.get_ref(), *id).await?;
    Ok(task)
}

pub(crate) fn task_service(cfg: &mut web::ServiceConfig) {
    cfg.service(insert);
    cfg.service(update);
    cfg.service(delete);
    cfg.service(done);
    cfg.service(undo);
    cfg.service(find_all);
    cfg.service(find_ongoing);
    cfg.service(find_by_pattern);
    cfg.service(find_by_id);
}
