use std::sync::atomic::Ordering;

use actix_web::{delete, get, post, put, web, HttpResponse};

use crate::{AppData, AppError, InsertTask, Task, UpdateTask};

#[post("/tasks")]
async fn insert(
    app_data: web::Data<AppData>,
    input: web::Json<InsertTask>,
) -> Result<HttpResponse, AppError> {
    if input.non_empty_title.trim().is_empty() {
        Err(AppError::EmptyTitle)
    } else {
        let new_task = Task {
            id: app_data.id_tracker.load(Ordering::Relaxed),
            title: input.non_empty_title.to_owned(),
            details: input.details.to_owned(),
        };

        let mut task_list = app_data
            .task_list
            // Try to acquire lock, convert to a 'catch-all' error on failure.
            .try_lock()
            .map_err(|_| AppError::Internal)?;

        task_list.push(new_task.clone());
        app_data.id_tracker.fetch_add(1, Ordering::Relaxed);

        let response = HttpResponse::Ok().json(new_task);
        Ok(response)
    }
}

#[get("/tasks")]
async fn find_all(app_data: web::Data<AppData>) -> Result<HttpResponse, AppError> {
    let task_list = app_data
        .task_list
        .try_lock()
        .map_err(|_| AppError::Internal)?;

    let response = HttpResponse::Ok().json(&task_list[..]);
    Ok(response)
}

#[get("/tasks/{id}")]
async fn find_by_id(
    app_data: web::Data<AppData>,
    id: web::Path<u64>,
) -> Result<HttpResponse, AppError> {
    let task_list = app_data
        .task_list
        .try_lock()
        .map_err(|_| AppError::Internal)?;

    let task = task_list
        .iter()
        .find(|t| t.id == *id)
        .ok_or(AppError::IdNotFound(*id))?;

    let response = HttpResponse::Ok().json(task);
    Ok(response)
}

#[delete("/tasks/{id}")]
async fn delete(
    app_data: web::Data<AppData>,
    id: web::Path<u64>,
) -> Result<HttpResponse, AppError> {
    let mut task_list = app_data
        .task_list
        .try_lock()
        .map_err(|_| AppError::Internal)?;

    let (index, _) = task_list
        .iter()
        .enumerate()
        .find(|(_, t)| t.id == *id)
        .ok_or(AppError::IdNotFound(*id))?;

    let task = task_list.remove(index);

    let response = HttpResponse::Ok().json(task);
    Ok(response)
}

#[put("/tasks")]
async fn update(
    app_data: web::Data<AppData>,
    input: web::Json<UpdateTask>,
) -> Result<HttpResponse, AppError> {
    if input.new_title.trim().is_empty() {
        Err(AppError::EmptyTitle)
    } else {
        let mut task_list = app_data
            .task_list
            .try_lock()
            .map_err(|_| AppError::Internal)?;

        let mut task = task_list
            .iter_mut()
            .find(|t| t.id == input.id)
            .ok_or(AppError::IdNotFound(input.id))?;

        task.title = input.new_title.to_owned();
        task.details = input.details.to_owned();

        let response = HttpResponse::Ok().json(task);
        Ok(response)
    }
}

pub(crate) fn task_service(cfg: &mut web::ServiceConfig) {
    cfg.service(insert);
    cfg.service(find_all);
    cfg.service(find_by_id);
    cfg.service(delete);
    cfg.service(update);
}
