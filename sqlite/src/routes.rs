use actix_web::{delete, get, post, put, web, HttpResponse};
use sqlx::SqlitePool;

use crate::{
    errors::AppError,
    models::{InsertTask, UpdateTask},
};

#[post("/tasks")]
async fn insert(
    db_pool: web::Data<SqlitePool>,
    input: web::Json<InsertTask>,
) -> Result<HttpResponse, AppError> {
    unimplemented!();
}

#[get("/tasks")]
async fn find_all(db_pool: web::Data<SqlitePool>) -> Result<HttpResponse, AppError> {
    unimplemented!();
}

#[get("/tasks/{id}")]
async fn find_by_id(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    unimplemented!();
}

#[delete("/tasks/{id}")]
async fn delete(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<HttpResponse, AppError> {
    unimplemented!();
}

#[put("/tasks")]
async fn update(
    db_pool: web::Data<SqlitePool>,
    input: web::Json<UpdateTask>,
) -> Result<HttpResponse, AppError> {
    unimplemented!();
}

pub(crate) fn task_service(cfg: &mut web::ServiceConfig) {
    cfg.service(insert);
    cfg.service(find_all);
    cfg.service(find_by_id);
    cfg.service(delete);
    cfg.service(update);
}
