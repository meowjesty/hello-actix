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

    let task = input.into_inner().insert(db_pool.get_ref()).await?;
    Ok(HttpResponse::Created().json(task))
}

#[put("/tasks")]
async fn update(
    db_pool: web::Data<SqlitePool>,
    input: web::Json<UpdateTask>,
) -> Result<HttpResponse, AppError> {
    if input.new_title.trim().is_empty() {
        return Err(TaskError::EmptyTitle.into());
    }

    let num_modified = input.into_inner().update(db_pool.get_ref()).await?;

    if num_modified == 0 {
        Ok(HttpResponse::NotModified().finish())
    } else {
        Ok(HttpResponse::Ok().body(format!("Updated {} tasks.", num_modified)))
    }
}

#[delete("/tasks/{id}")]
async fn delete(
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

#[post("/tasks/{id}/done")]
async fn done(
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

#[delete("/tasks/{id}/undo")]
async fn undo(
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
async fn find_all(db_pool: web::Data<SqlitePool>) -> Result<impl Responder, AppError> {
    let tasks = Task::find_all(db_pool.get_ref()).await?;
    Ok(HttpResponse::Found().json(&tasks))
}

#[get("/tasks/ongoing")]
async fn find_ongoing(db_pool: web::Data<SqlitePool>) -> Result<impl Responder, AppError> {
    let tasks = Task::find_ongoing(db_pool.get_ref()).await?;
    Ok(HttpResponse::Found().json(&tasks))
}

#[get("/tasks")]
async fn find_by_pattern(
    db_pool: web::Data<SqlitePool>,
    pattern: web::Query<QueryTask>,
) -> Result<impl Responder, AppError> {
    let tasks = Task::find_by_pattern(db_pool.get_ref(), &format!("%{}%", pattern.title)).await?;
    Ok(HttpResponse::Found().json(&tasks))
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

#[cfg(test)]
mod tests {
    use actix_web::{
        http::StatusCode,
        test,
        web::{self, ServiceConfig},
        App,
    };
    use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

    use super::*;
    use crate::create_database;

    #[macro_export]
    macro_rules! setup_app {
        ($configure: expr) => {{
            let data = setup_data().await;
            let app = App::new().app_data(data.clone()).configure($configure);
            let app = test::init_service(app).await;

            app
        }};
    }

    macro_rules! pre_insert_task {
        ($app: expr) => {{
            let insert_task = InsertTask {
                non_empty_title: "Re-watch Cowboy Bebop".to_string(),
                details: "Good show.".to_string(),
            };

            let insert_task_request = test::TestRequest::post()
                .uri("/tasks")
                .set_json(&insert_task)
                .to_request();
            let insert_task_response = test::call_service(&mut $app, insert_task_request).await;
            assert!(insert_task_response.status().is_success());

            let task: Task = test::read_body_json(insert_task_response).await;
            task
        }};
    }

    async fn setup_data() -> web::Data<Pool<Sqlite>> {
        let db_options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(env!("DATABASE_FILE"))
            .create_if_missing(true);

        let database_pool = SqlitePoolOptions::new()
            .max_connections(15)
            .connect_with(db_options)
            .await
            .unwrap();

        create_database(&database_pool).await.unwrap();

        web::Data::new(database_pool)
    }

    #[actix_rt::test]
    pub async fn test_task_insert_valid_task() {
        let data = setup_data().await;
        let mut app = test::init_service(App::new().app_data(data).service(insert)).await;

        let valid_insert_task = InsertTask {
            non_empty_title: "Re-watch Cowboy Bebop".to_string(),
            details: "Good show.".to_string(),
        };

        let request = test::TestRequest::post()
            .uri("/tasks")
            .set_json(&valid_insert_task)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    pub async fn test_task_insert_invalid_task_title() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(insert);
        };

        let mut app = setup_app!(configure);

        let invalid_insert_task = InsertTask {
            non_empty_title: " \n\t".to_string(),
            details: "Good show.".to_string(),
        };

        let request = test::TestRequest::post()
            .uri("/tasks")
            .set_json(&invalid_insert_task)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_client_error());
    }

    #[actix_rt::test]
    pub async fn test_task_update_valid_task() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(insert);
            cfg.service(update);
        };

        let mut app = setup_app!(configure);
        let task = pre_insert_task!(app);

        let update_task = UpdateTask {
            id: task.id,
            new_title: format!("{}, and Yu Yu Hakusho", task.title),
            details: format!("{} Classic.", task.details),
        };

        // NOTE(alex): Update
        let request = test::TestRequest::put()
            .uri("/tasks")
            .set_json(&update_task)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    pub async fn test_task_update_with_invalid_task_title() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(insert);
            cfg.service(update);
        };

        let mut app = setup_app!(configure);
        let task = pre_insert_task!(app);

        let update_task = UpdateTask {
            id: task.id,
            new_title: " \n\t".to_string(),
            details: format!("{} Classic.", task.details),
        };

        // NOTE(alex): Update
        let request = test::TestRequest::put()
            .uri("/tasks")
            .set_json(&update_task)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_client_error());
    }

    #[actix_rt::test]
    pub async fn test_task_delete_existing_task() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(insert);
            cfg.service(delete);
        };

        let mut app = setup_app!(configure);
        let task = pre_insert_task!(app);

        // NOTE(alex): Delete
        let request = test::TestRequest::delete()
            .uri(&format!("/tasks/{}", task.id))
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    pub async fn test_task_delete_non_existent_task() {
        let database_pool = setup_data().await;
        let mut app = test::init_service(App::new().app_data(database_pool).service(delete)).await;

        // NOTE(alex): Delete
        let request = test::TestRequest::delete()
            .uri(&format!("/tasks/{}", 1000))
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    }

    #[actix_rt::test]
    pub async fn test_task_mark_as_done() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(insert);
            cfg.service(done);
        };

        let mut app = setup_app!(configure);
        let task = pre_insert_task!(app);

        // NOTE(alex): Done
        let request = test::TestRequest::post()
            .uri(&format!("/tasks/{}/done", task.id))
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    pub async fn test_task_undo() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(insert);
            cfg.service(done);
            cfg.service(undo);
        };

        let mut app = setup_app!(configure);
        let task = pre_insert_task!(app);

        // NOTE(alex): Done
        let task_done_request = test::TestRequest::post()
            .uri(&format!("/tasks/{}/done", task.id))
            .to_request();
        let task_done_response = test::call_service(&mut app, task_done_request).await;
        assert!(task_done_response.status().is_success());

        // NOTE(alex): Undo
        let request = test::TestRequest::delete()
            .uri(&format!("/tasks/{}/undo", task.id))
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    pub async fn test_task_find_all() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(insert);
            cfg.service(find_all);
        };

        let mut app = setup_app!(configure);
        let _ = pre_insert_task!(app);

        // NOTE(alex): Find all
        let request = test::TestRequest::get().uri("/tasks").to_request();
        let response = test::call_service(&mut app, request).await;

        assert_eq!(response.status(), StatusCode::FOUND);
    }

    #[actix_rt::test]
    pub async fn test_task_ongoing_tasks() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(insert);
            cfg.service(done);
            cfg.service(find_ongoing);
        };

        let mut app = setup_app!(configure);
        let _ = pre_insert_task!(app);
        let task = pre_insert_task!(app);

        // NOTE(alex): Done
        let task_done_request = test::TestRequest::post()
            .uri(&format!("/tasks/{}/done", task.id))
            .to_request();
        let task_done_response = test::call_service(&mut app, task_done_request).await;
        assert!(task_done_response.status().is_success());

        // NOTE(alex): Find ongoing tasks only
        let request = test::TestRequest::get().uri("/tasks/ongoing").to_request();
        let response = test::call_service(&mut app, request).await;

        assert_eq!(response.status(), StatusCode::FOUND);
    }

    #[actix_rt::test]
    pub async fn test_task_find_by_pattern() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(insert);
            cfg.service(find_by_pattern);
        };

        let mut app = setup_app!(configure);
        let _ = pre_insert_task!(app);

        let title_pattern = "?title=Watch&details=.";
        // NOTE(alex): Find tasks with title pattern
        let request = test::TestRequest::get()
            .uri(&format!("/tasks{}", title_pattern))
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert_eq!(response.status(), StatusCode::FOUND);
    }

    #[actix_rt::test]
    pub async fn test_task_find_by_id() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(insert);
            cfg.service(find_by_id);
        };

        let mut app = setup_app!(configure);
        let task = pre_insert_task!(app);

        // NOTE(alex): Find with id
        let request = test::TestRequest::get()
            .uri(&format!("/tasks/{}", task.id))
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }
}
