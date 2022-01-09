use actix_session::Session;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use sqlx::SqlitePool;

use super::{errors::*, models::*};
use crate::{errors::AppError, validator};

#[post("/tasks", wrap = "HttpAuthentication::bearer(validator)")]
async fn insert(
    db_pool: web::Data<SqlitePool>,
    input: InsertTask,
) -> Result<impl Responder, AppError> {
    let task = input.insert(db_pool.get_ref()).await?;
    Ok(HttpResponse::Created().json(task))
}

#[put("/tasks", wrap = "HttpAuthentication::bearer(validator)")]
async fn update(
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

#[post("/tasks/{id}/done", wrap = "HttpAuthentication::bearer(validator)")]
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

#[delete("/tasks/{id}/undo", wrap = "HttpAuthentication::bearer(validator)")]
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

    if tasks.is_empty() {
        Err(TaskError::Empty.into())
    } else {
        Ok(HttpResponse::Found().json(&tasks))
    }
}

#[get("/tasks/ongoing")]
async fn find_ongoing(db_pool: web::Data<SqlitePool>) -> Result<impl Responder, AppError> {
    let tasks = Task::find_ongoing(db_pool.get_ref()).await?;

    if tasks.is_empty() {
        Err(TaskError::Empty.into())
    } else {
        Ok(HttpResponse::Found().json(&tasks))
    }
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

    match task {
        Some(task) => Ok(HttpResponse::Found().json(task)),
        None => Err(TaskError::NotFound(*id).into()),
    }
}

const FAVORITE_TASK_STR: &'static str = "favorite_task";

#[post("/tasks/favorite/{id}")]
async fn favorite(
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
async fn find_favorite(session: Session) -> Result<impl Responder, AppError> {
    if let Some(task) = session.get::<Task>(FAVORITE_TASK_STR)? {
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

// WARNING(alex): Please ignore these tests for now, we'll take a better look at them on the
// `integration` project!
#[cfg(test)]
mod tests {
    use actix_identity::{CookieIdentityPolicy, IdentityService};
    use actix_web::{cookie::Cookie, http::StatusCode, test, web, web::ServiceConfig, App};
    use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
    use time::Duration;

    use crate::{
        create_database,
        tasks::{
            models::{InsertTask, Task, UpdateTask},
            routes::{
                delete as task_delete, done as task_done, favorite, find_all as task_find_all,
                find_by_id as task_find_by_id, find_by_pattern as task_find_by_pattern,
                find_favorite, find_ongoing, insert as task_insert, undo as task_undo,
                update as task_update,
            },
        },
        users::{
            models::{InsertUser, LoggedUser, LoginUser, User},
            routes::{find_by_id as user_find_by_id, insert as user_insert, login},
        },
    };

    pub async fn setup_data() -> web::Data<Pool<Sqlite>> {
        let db_options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(env!("DATABASE_FILE"))
            .create_if_missing(true);

        let database_pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(db_options)
            .await
            .unwrap();

        create_database(&database_pool).await.unwrap();

        web::Data::new(database_pool)
    }

    // WARNING(alex): This macro doesn't check if there is an user register already, or if some user
    // is logged in, so the tests must be run with:
    // cargo test -- --test-threads=1
    // Running them in parallel may fail!
    macro_rules! setup_app {
        ($configure: expr) => {{
            // TODO(alex) [low] 2021-07-16: Why is rust complaining when I put this outside?
            use actix_session::CookieSession;

            let data = setup_data().await;
            let app = App::new()
                .app_data(data.clone())
                .configure($configure)
                .service(user_insert)
                .service(login)
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .login_deadline(Duration::minutes(10))
                        .secure(false),
                ))
                .wrap(
                    CookieSession::signed(&[0; 32])
                        .name("session-cookie")
                        .secure(false)
                        // WARNING(alex): This uses the `time` crate, not `std::time`!
                        .expires_in_time(Duration::minutes(5)),
                );
            let mut app = test::init_service(app).await;

            let (cookies, bearer_token) = {
                let new_user = InsertUser {
                    valid_username: "spike".to_string(),
                    valid_password: "vicious".to_string(),
                };
                let register_user_request = test::TestRequest::post()
                    .uri("/users/register")
                    .set_json(&new_user)
                    .to_request();
                let register_user_service_response =
                    test::call_service(&mut app, register_user_request).await;
                assert!(register_user_service_response.status().is_success());

                let user: User = test::read_body_json(register_user_service_response).await;

                let login_user = LoginUser {
                    username: user.username,
                    password: user.password,
                };
                let login_request = test::TestRequest::post()
                    .uri("/users/login")
                    .set_json(&login_user)
                    .to_request();
                let login_service_response = test::call_service(&mut app, login_request).await;
                assert!(login_service_response.status().is_success());

                let cookies = login_service_response.response().cookies();
                let cookies_str = cookies
                    .flat_map(|cookie| cookie.to_string().chars().collect::<Vec<_>>())
                    .collect::<String>();

                let logged_user: LoggedUser = test::read_body_json(login_service_response).await;

                let bearer_token = format!("Bearer {}", logged_user.token);
                let cookies = Cookie::parse_encoded(cookies_str).unwrap();

                (cookies, bearer_token)
            };

            (app, bearer_token, cookies)
        }};
    }

    macro_rules! pre_insert_task {
        ($bearer_token: expr, $cookies: expr, $app: expr) => {{
            let insert_task = InsertTask {
                non_empty_title: "Re-watch Cowboy Bebop".to_string(),
                details: "Good show.".to_string(),
            };

            let insert_task_request = test::TestRequest::post()
                .uri("/tasks")
                .insert_header(("Authorization".to_string(), $bearer_token.clone()))
                .cookie($cookies.clone())
                .set_json(&insert_task)
                .to_request();
            let insert_task_response = test::call_service(&mut $app, insert_task_request).await;
            assert!(insert_task_response.status().is_success());

            let task: Task = test::read_body_json(insert_task_response).await;
            task
        }};
    }

    // NOTE(alex): I'm leaving this function without the `setup_app` macro to make messing with it
    // easier.
    #[actix_rt::test]
    pub async fn test_task_insert_valid_task() {
        let data = setup_data().await;
        let app = App::new()
            .app_data(data.clone())
            .service(user_insert)
            .service(user_find_by_id)
            .configure(|cfg| {
                cfg.service(task_insert);
            })
            .service(login)
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("auth-cookie")
                    .login_deadline(Duration::minutes(10))
                    .secure(false),
            ));
        let mut app = test::init_service(app).await;

        let (cookies, bearer_token) = {
            let new_user = InsertUser {
                valid_username: "spike".to_string(),
                valid_password: "vicious".to_string(),
            };
            let register_user_request = test::TestRequest::post()
                .uri("/users/register")
                .set_json(&new_user)
                .to_request();
            let register_user_service_response =
                test::call_service(&mut app, register_user_request).await;
            assert!(register_user_service_response.status().is_success());

            let user: User = test::read_body_json(register_user_service_response).await;

            let login_user = LoginUser {
                username: user.username,
                password: user.password,
            };
            let login_request = test::TestRequest::post()
                .uri("/users/login")
                .set_json(&login_user)
                .to_request();
            let login_service_response = test::call_service(&mut app, login_request).await;
            assert!(login_service_response.status().is_success());

            let cookies = login_service_response.response().cookies();
            let cookies_str = cookies
                .flat_map(|cookie| cookie.to_string().chars().collect::<Vec<_>>())
                .collect::<String>();

            let logged_user: LoggedUser = test::read_body_json(login_service_response).await;

            let bearer_token = format!("Bearer {}", logged_user.token);
            let cookies = Cookie::parse_encoded(cookies_str).unwrap();

            (cookies, bearer_token)
        };

        let valid_insert_task = InsertTask {
            non_empty_title: "Re-watch Cowboy Bebop".to_string(),
            details: "Good show.".to_string(),
        };

        let request = test::TestRequest::post()
            .uri("/tasks")
            .insert_header(("Authorization".to_string(), bearer_token))
            .cookie(cookies)
            .set_json(&valid_insert_task)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    pub async fn test_task_insert_invalid_task_title() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(task_insert);
        };

        let (mut app, bearer_token, cookies) = setup_app!(configure);

        let invalid_insert_task = InsertTask {
            non_empty_title: " \n\t".to_string(),
            details: "Good show.".to_string(),
        };

        let request = test::TestRequest::post()
            .uri("/tasks")
            .insert_header(("Authorization".to_string(), bearer_token))
            .cookie(cookies)
            .set_json(&invalid_insert_task)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_client_error());
    }

    #[actix_rt::test]
    pub async fn test_task_update_valid_task() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(task_insert);
            cfg.service(task_update);
        };

        let (mut app, bearer_token, cookies) = setup_app!(configure);
        let task = pre_insert_task!(bearer_token, cookies, app);

        let update_task = UpdateTask {
            id: task.id,
            new_title: format!("{}, and Yu Yu Hakusho", task.title),
            details: format!("{} Classic.", task.details),
        };

        // NOTE(alex): Update
        let request = test::TestRequest::put()
            .uri("/tasks")
            .insert_header(("Authorization".to_string(), bearer_token))
            .cookie(cookies)
            .set_json(&update_task)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    pub async fn test_task_update_with_invalid_task_title() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(task_insert);
            cfg.service(task_update);
        };

        let (mut app, bearer_token, cookies) = setup_app!(configure);
        let task = pre_insert_task!(bearer_token, cookies, app);

        let update_task = UpdateTask {
            id: task.id,
            new_title: " \n\t".to_string(),
            details: format!("{} Classic.", task.details),
        };

        // NOTE(alex): Update
        let request = test::TestRequest::put()
            .uri("/tasks")
            .insert_header(("Authorization".to_string(), bearer_token))
            .cookie(cookies)
            .set_json(&update_task)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_client_error());
    }

    #[actix_rt::test]
    pub async fn test_task_delete_existing_task() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(task_insert);
            cfg.service(task_delete);
        };

        let (mut app, bearer_token, cookies) = setup_app!(configure);
        let task = pre_insert_task!(bearer_token, cookies, app);

        // NOTE(alex): Delete
        let request = test::TestRequest::delete()
            .uri(&format!("/tasks/{}", task.id))
            .insert_header(("Authorization".to_string(), bearer_token))
            .cookie(cookies)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    pub async fn test_task_delete_non_existent_task() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(task_delete);
        };

        let (mut app, bearer_token, cookies) = setup_app!(configure);

        // NOTE(alex): Delete
        let request = test::TestRequest::delete()
            .uri(&format!("/tasks/{}", 1000))
            .insert_header(("Authorization".to_string(), bearer_token))
            .cookie(cookies)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    }

    #[actix_rt::test]
    pub async fn test_task_mark_as_done() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(task_insert);
            cfg.service(task_done);
        };

        let (mut app, bearer_token, cookies) = setup_app!(configure);
        let task = pre_insert_task!(bearer_token, cookies, app);

        // NOTE(alex): Done
        let request = test::TestRequest::post()
            .uri(&format!("/tasks/{}/done", task.id))
            .insert_header(("Authorization".to_string(), bearer_token))
            .cookie(cookies)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    pub async fn test_task_undo() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(task_insert);
            cfg.service(task_done);
            cfg.service(task_undo);
        };

        let (mut app, bearer_token, cookies) = setup_app!(configure);
        let task = pre_insert_task!(bearer_token, cookies, app);

        // NOTE(alex): Done
        let task_done_request = test::TestRequest::post()
            .uri(&format!("/tasks/{}/done", task.id))
            .insert_header(("Authorization".to_string(), bearer_token.clone()))
            .cookie(cookies.clone())
            .to_request();
        let task_done_response = test::call_service(&mut app, task_done_request).await;
        assert!(task_done_response.status().is_success());

        // NOTE(alex): Undo
        let request = test::TestRequest::delete()
            .uri(&format!("/tasks/{}/undo", task.id))
            .insert_header(("Authorization".to_string(), bearer_token))
            .cookie(cookies)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    pub async fn test_task_find_all() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(task_insert);
            cfg.service(task_find_all);
        };

        let (mut app, bearer_token, cookies) = setup_app!(configure);
        let _ = pre_insert_task!(bearer_token, cookies, app);

        // NOTE(alex): Find all
        let request = test::TestRequest::get().uri("/tasks").to_request();
        let response = test::call_service(&mut app, request).await;

        assert_eq!(response.status(), StatusCode::FOUND);
    }

    #[actix_rt::test]
    pub async fn test_task_ongoing_tasks() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(task_insert);
            cfg.service(task_done);
            cfg.service(find_ongoing);
        };

        let (mut app, bearer_token, cookies) = setup_app!(configure);
        let _ = pre_insert_task!(bearer_token, cookies, app);
        let task = pre_insert_task!(bearer_token, cookies, app);

        // NOTE(alex): Done
        let task_done_request = test::TestRequest::post()
            .uri(&format!("/tasks/{}/done", task.id))
            .insert_header(("Authorization".to_string(), bearer_token.clone()))
            .cookie(cookies.clone())
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
            cfg.service(task_insert);
            cfg.service(task_find_by_pattern);
        };

        let (mut app, bearer_token, cookies) = setup_app!(configure);
        let _ = pre_insert_task!(bearer_token, cookies, app);

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
            cfg.service(task_insert);
            cfg.service(task_find_by_id);
        };

        let (mut app, bearer_token, cookies) = setup_app!(configure);
        let task = pre_insert_task!(bearer_token, cookies, app);

        // NOTE(alex): Find with id
        let request = test::TestRequest::get()
            .uri(&format!("/tasks/{}", task.id))
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert_eq!(response.status(), StatusCode::FOUND);
    }

    #[actix_rt::test]
    pub async fn test_task_favorite() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(task_insert);
            cfg.service(favorite);
        };

        let (mut app, bearer_token, cookies) = setup_app!(configure);
        let task = pre_insert_task!(bearer_token, cookies, app);

        // NOTE(alex): Favorite
        let request = test::TestRequest::post()
            .uri(&format!("/tasks/favorite/{}", task.id))
            .insert_header(("Authorization".to_string(), bearer_token.clone()))
            .cookie(cookies.clone())
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert_eq!(response.status(), StatusCode::FOUND);
    }

    #[actix_rt::test]
    pub async fn test_task_find_favorite() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(task_insert);
            cfg.service(favorite);
            cfg.service(find_favorite);
        };

        let (mut app, bearer_token, cookies) = setup_app!(configure);
        let task = pre_insert_task!(bearer_token, cookies, app);

        // NOTE(alex): Favorite
        let task_favorite_request = test::TestRequest::post()
            .uri(&format!("/tasks/favorite/{}", task.id))
            .insert_header(("Authorization".to_string(), bearer_token.clone()))
            .cookie(cookies.clone())
            .to_request();
        let task_favorite_response = test::call_service(&mut app, task_favorite_request).await;
        assert_eq!(task_favorite_response.status(), StatusCode::FOUND);

        // NOTE(alex): Retrieve the session cookies to insert them into the find favorite request.
        let session_cookies = task_favorite_response.response().cookies();
        let cookies_str = session_cookies
            .flat_map(|cookie| cookie.to_string().chars().collect::<Vec<_>>())
            .collect::<String>();
        let cookies = Cookie::parse_encoded(cookies_str).unwrap();

        // NOTE(alex): Find favorite
        let request = test::TestRequest::get()
            .uri("/tasks/favorite")
            .cookie(cookies.clone())
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert_eq!(response.status(), StatusCode::FOUND);
    }
}
