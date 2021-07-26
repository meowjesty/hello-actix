use actix_session::Session;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use sqlx::SqlitePool;

use super::{errors::*, models::*};
use crate::{errors::AppError, validator};

#[post("/tasks", wrap = "HttpAuthentication::bearer(validator)")]
pub async fn insert(
    db_pool: web::Data<SqlitePool>,
    input: InsertTask,
) -> Result<impl Responder, AppError> {
    let task = input.insert(db_pool.get_ref()).await?;
    Ok(HttpResponse::Created().json(task))
}

#[put("/tasks", wrap = "HttpAuthentication::bearer(validator)")]
pub async fn update(
    db_pool: web::Data<SqlitePool>,
    input: UpdateTask,
) -> Result<impl Responder, AppError> {
    let num_modified = input.update(db_pool.get_ref()).await?;

    if num_modified == 0 {
        Ok(HttpResponse::NotModified().body("No tasks were updated."))
    } else {
        Ok(HttpResponse::Ok().body(format!("Updated {} tasks.", num_modified)))
    }
}

#[delete("/tasks/{id}", wrap = "HttpAuthentication::bearer(validator)")]
pub async fn delete(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let num_modified = Task::delete(db_pool.get_ref(), *id).await?;

    if num_modified == 0 {
        Ok(HttpResponse::NotModified().body("No tasks were deleted."))
    } else {
        Ok(HttpResponse::Ok().body(format!("Deleted {} tasks.", num_modified)))
    }
}

// TODO(alex): Cleanup error returned:
// `error returned from database: FOREIGN KEY constraint failed`
#[post("/tasks/{id}/done", wrap = "HttpAuthentication::bearer(validator)")]
pub async fn done(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let done_id = Task::done(db_pool.get_ref(), *id).await?;

    if done_id == 0 {
        Ok(HttpResponse::NotModified().body(format!("Task with id {} not done.", id)))
    } else {
        Ok(HttpResponse::Created().body(done_id.to_string()))
    }
}

// TODO(alex): Cleanup error:
// Error: Stream error in the HTTP/2 framing layer
#[delete("/tasks/{id}/undo", wrap = "HttpAuthentication::bearer(validator)")]
pub async fn undo(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let num_modified = Task::undo(db_pool.get_ref(), *id).await?;

    if num_modified == 0 {
        Ok(HttpResponse::NotModified().body(format!("Task with id {} not undone.", id)))
    } else {
        Ok(HttpResponse::Ok().body(format!("Undone {} tasks.", num_modified)))
    }
}

#[get("/tasks")]
pub async fn find_all(db_pool: web::Data<SqlitePool>) -> Result<impl Responder, AppError> {
    let tasks = Task::find_all(db_pool.get_ref()).await?;

    if tasks.is_empty() {
        Err(TaskError::Empty.into())
    } else {
        Ok(HttpResponse::Found().json(&tasks))
    }
}

#[get("/tasks/ongoing")]
pub async fn find_ongoing(db_pool: web::Data<SqlitePool>) -> Result<impl Responder, AppError> {
    let tasks = Task::find_ongoing(db_pool.get_ref()).await?;

    if tasks.is_empty() {
        Err(TaskError::Empty.into())
    } else {
        Ok(HttpResponse::Found().json(&tasks))
    }
}

#[get("/tasks")]
pub async fn find_by_pattern(
    db_pool: web::Data<SqlitePool>,
    pattern: web::Query<QueryTask>,
) -> Result<impl Responder, AppError> {
    let tasks = Task::find_by_pattern(db_pool.get_ref(), &format!("%{}%", pattern.title)).await?;

    if tasks.is_empty() {
        Err(TaskError::Empty.into())
    } else {
        Ok(HttpResponse::Found().json(&tasks))
    }
}

/// NOTE(alex): Regex to match only digits, otherwise it matches the "/tasks/favorite" find route.
/// This issue may be solved in one of two ways:
///
/// 1. Include a regex or a `guard` to check which route is the best representative for this type of
/// request;
/// 2. Order the routes during setup in a way that avoids conflicts, such as a `{id}` pattern, which
// is the equivalent of the `[^/]+` regex.
///
/// There is a 3rd way of sorts, which boils down to: avoid possible route conflicting paths.
#[get("/tasks/{id:\\d+}")]
pub async fn find_by_id(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let task = Task::find_by_id(db_pool.get_ref(), *id).await?;

    match task {
        Some(task) => Ok(HttpResponse::Found().json(task)),
        None => Err(TaskError::NotFound(*id).into()),
    }
}

const FAVORITE_TASK_STR: &'static str = "favorite_task";

#[post("/tasks/favorite/{id}")]
pub async fn favorite(
    db_pool: web::Data<SqlitePool>,
    session: Session,
    id: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    if let Some(old) = session.remove(FAVORITE_TASK_STR) {
        let old_favorite: Task = serde_json::from_str(&old)?;

        if old_favorite.id == *id {
            // NOTE(alex): Just remove the task, this is basically "unfavorite".
            Ok(HttpResponse::NoContent().body(format!("Task {} unfavorited", old_favorite.id)))
        } else {
            match Task::find_by_id(&db_pool, *id).await? {
                Some(task) => {
                    session.insert(FAVORITE_TASK_STR, task.clone())?;
                    Ok(HttpResponse::Found().json(task))
                }
                None => Err(TaskError::NotFound(*id).into()),
            }
        }
    } else {
        match Task::find_by_id(&db_pool, *id).await? {
            Some(task) => {
                session.insert(FAVORITE_TASK_STR, task.clone())?;
                Ok(HttpResponse::Found().json(task))
            }
            None => Err(TaskError::NoneFavorite.into()),
        }
    }
}

#[get("/tasks/favorite")]
pub async fn find_favorite(session: Session) -> Result<impl Responder, AppError> {
    if let Some(task) = session.get::<Task>(FAVORITE_TASK_STR)? {
        Ok(HttpResponse::Found().json(task))
    } else {
        Err(TaskError::NoneFavorite.into())
    }
}

pub fn task_service(cfg: &mut web::ServiceConfig) {
    cfg.service(insert);
    cfg.service(update);
    cfg.service(delete);
    cfg.service(done);
    cfg.service(undo);
    cfg.service(find_all);
    cfg.service(find_ongoing);
    cfg.service(find_by_pattern);
    cfg.service(find_by_id);
    cfg.service(favorite);
    cfg.service(find_favorite);
}
