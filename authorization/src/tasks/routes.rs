use actix_session::Session;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use sqlx::SqlitePool;

use super::{errors::*, models::*};
use crate::errors::AppError;

#[post("/tasks")]
async fn insert(
    db_pool: web::Data<SqlitePool>,
    input: InsertTask,
) -> Result<impl Responder, AppError> {
    let task = input.insert(db_pool.get_ref()).await?;
    Ok(HttpResponse::Created().json(task))
}

#[put("/tasks")]
async fn update(
    db_pool: web::Data<SqlitePool>,
    input: UpdateTask,
) -> Result<impl Responder, AppError> {
    let num_modified = input.update(db_pool.get_ref()).await?;

    if num_modified == 0 {
        Ok(HttpResponse::NotModified().finish())
    } else {
        Ok(HttpResponse::Created().body(num_modified.to_string()))
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
        Ok(HttpResponse::Created().body(created_id.to_string()))
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

    if tasks.is_empty() {
        Err(TaskError::Empty.into())
    } else {
        Ok(HttpResponse::Found().json(&tasks))
    }
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
async fn find_by_id(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let task = Task::find_by_id(db_pool.get_ref(), *id).await?;
    Ok(task)
}

const FAVORITE_TASK_STR: &'static str = "favorite_task";

#[get("/tasks/favorite/{id}")]
async fn favorite(
    db_pool: web::Data<SqlitePool>,
    session: Session,
    id: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    if let Some(old) = session.remove(FAVORITE_TASK_STR) {
        let old_favorite: Task = serde_json::from_str(&old)?;

        if old_favorite.id == *id {
            // NOTE(alex): Just remove the task, this is basically "unfavorite".
            Ok(HttpResponse::NoContent().finish())
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
async fn find_favorite(session: Session) -> Result<impl Responder, AppError> {
    if let Some(task) = session.get::<Task>("favorite_task")? {
        Ok(HttpResponse::Found().json(task))
    } else {
        Err(TaskError::NoneFavorite.into())
    }
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
    cfg.service(favorite);
    cfg.service(find_favorite);
}

/// NOTE(alex): Binary crates cannot use the integration test convention of having a separate
/// `tests` folder living alongside `src`. To have proper testing (smaller file size) you could
/// create a library crate, and move the implementation and tests there.
///
/// Keep in mind that the library approach requires every testable function to have `pub`
/// visilibity!
#[cfg(test)]
mod tests {
    use std::env;

    use actix_web::{test, web, App};
    use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

    use super::*;
    use crate::create_database;

    async fn setup_data() -> web::Data<Pool<Sqlite>> {
        let db_options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(env!("DATABASE_FILE"))
            .create_if_missing(true);

        let database_pool = SqlitePoolOptions::new()
            .max_connections(20)
            .connect_with(db_options)
            .await
            .unwrap();

        create_database(&database_pool).await.unwrap();

        web::Data::new(database_pool)
    }

    #[actix_rt::test]
    async fn test_insert_valid() {
        let data = setup_data().await;
        let mut app = test::init_service(App::new().app_data(data.clone()).service(insert)).await;

        let valid_insert = InsertTask {
            non_empty_title: "Valid title".to_string(),
            details: "details".to_string(),
        };

        let request = test::TestRequest::post()
            .uri("/tasks")
            .set_json(&valid_insert)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    async fn test_insert_invalid_title() {
        let database_pool = setup_data().await;
        let mut app = test::init_service(App::new().app_data(database_pool).service(insert)).await;

        let invalid_insert = InsertTask {
            non_empty_title: " \n\t".to_string(),
            details: "details".to_string(),
        };

        let request = test::TestRequest::post()
            .uri("/tasks")
            .set_json(&invalid_insert)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_update_valid() {
        let database_pool = setup_data().await;
        let mut app = test::init_service(
            App::new()
                .app_data(database_pool)
                .service(insert)
                .service(update),
        )
        .await;

        let task = InsertTask {
            non_empty_title: "Valid title".to_string(),
            details: "details".to_string(),
        };

        // NOTE(alex): Insert before updating.
        let request = test::TestRequest::post()
            .uri("/tasks")
            .set_json(&task)
            .to_request();
        let response = test::call_service(&mut app, request).await;
        assert!(response.status().is_success());

        // TODO(alex) [low] 2021-06-21: Why doesn't it implement `try_into` for string?
        let task: Task = match response.into_body() {
            actix_web::body::AnyBody::Bytes(bytes) => {
                serde_json::from_slice(&bytes).expect("Failed deserializing created task!")
            }
            _ => panic!("Unexpected body!"),
        };
        let valid_update = UpdateTask {
            id: task.id,
            new_title: format!("{} Updated", task.title),
            details: format!("{} Updated", task.details),
        };

        // NOTE(alex): Update
        let request = test::TestRequest::put()
            .uri("/tasks")
            .set_json(&valid_update)
            .to_request();
        let response = test::call_service(&mut app, request).await;
        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    async fn test_update_invalid_title() {
        let database_pool = setup_data().await;
        let mut app = test::init_service(
            App::new()
                .app_data(database_pool)
                .service(insert)
                .service(update),
        )
        .await;

        let task = InsertTask {
            non_empty_title: "Title".to_string(),
            details: "details".to_string(),
        };

        // NOTE(alex): Insert before updating.
        let request = test::TestRequest::post()
            .uri("/tasks")
            .set_json(&task)
            .to_request();
        let response = test::call_service(&mut app, request).await;
        assert!(response.status().is_success());

        // TODO(alex) [low] 2021-06-21: Why doesn't it implement `try_into` for string?
        let task: Task = match response.into_body() {
            actix_web::body::AnyBody::Bytes(bytes) => {
                serde_json::from_slice(&bytes).expect("Failed deserializing created task!")
            }
            _ => panic!("Unexpected body!"),
        };
        let invalid_update = UpdateTask {
            id: task.id,
            new_title: " \n\t".to_string(),
            details: format!("{} Updated", task.details),
        };

        // NOTE(alex): Update
        let request = test::TestRequest::put()
            .uri("/tasks")
            .set_json(&invalid_update)
            .to_request();
        let response = test::call_service(&mut app, request).await;
        assert!(response.status().is_client_error());
    }

    #[actix_rt::test]
    async fn test_delete() {
        let database_pool = setup_data().await;
        let mut app = test::init_service(
            App::new()
                .app_data(database_pool)
                .service(insert)
                .service(delete),
        )
        .await;

        let task = InsertTask {
            non_empty_title: "Valid title".to_string(),
            details: "details".to_string(),
        };

        // NOTE(alex): Insert
        let request = test::TestRequest::post()
            .uri("/tasks")
            .set_json(&task)
            .to_request();
        let response = test::call_service(&mut app, request).await;
        assert!(response.status().is_success());

        // NOTE(alex): Delete
        let request = test::TestRequest::delete()
            .uri("/tasks/1")
            // TODO(alex) [low] 2021-06-06: Why doesn't this work?
            // .uri("/tasks")
            // .param("id", "1")
            .to_request();

        let response = test::call_service(&mut app, request).await;
        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    async fn test_delete_nothing() {
        let database_pool = setup_data().await;
        let mut app = test::init_service(App::new().app_data(database_pool).service(delete)).await;

        let request = test::TestRequest::delete().uri("/tasks/1000").to_request();
        let response = test::call_service(&mut app, request).await;
        assert!(response.status().is_redirection());
    }
}
