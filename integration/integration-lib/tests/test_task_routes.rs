mod common;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_session::CookieSession;
use actix_web::{dev::AnyBody, test, web, App};
use common::setup_data;
use integration_lib::{
    tasks::{
        models::{InsertTask, Task, UpdateTask},
        routes::{delete as task_delete, insert as task_insert, update as task_update},
    },
    users::routes::{insert as user_insert, login},
};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use time::Duration;

#[actix_rt::test]
pub async fn test_task_insert_valid() {
    let data = setup_data().await;
    let app = App::new()
        .app_data(data.clone())
        .service(task_insert)
        .service(user_insert)
        .service(login)
        .wrap(IdentityService::new(
            CookieIdentityPolicy::new(&[0; 32])
                .name("auth-cookie")
                .login_deadline(Duration::seconds(120))
                .secure(false),
        ));
    let mut app = test::init_service(app).await;

    // TODO(alex) [high] 2021-07-13: Register user, login, and insert task.

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
pub async fn test_task_insert_invalid_title() {
    let database_pool = setup_data().await;
    let mut app = test::init_service(
        App::new()
            .app_data(database_pool)
            .service(task_insert)
            .service(user_insert),
    )
    .await;

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
pub async fn test_task_update_valid() {
    let database_pool = setup_data().await;
    let mut app = test::init_service(
        App::new()
            .app_data(database_pool)
            .service(task_insert)
            .service(task_update),
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
pub async fn test_task_update_invalid_title() {
    let database_pool = setup_data().await;
    let mut app = test::init_service(
        App::new()
            .app_data(database_pool)
            .service(task_insert)
            .service(task_update),
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
pub async fn test_task_delete() {
    let database_pool = setup_data().await;
    let mut app = test::init_service(
        App::new()
            .app_data(database_pool)
            .service(task_insert)
            .service(task_delete),
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
pub async fn test_task_delete_nothing() {
    let database_pool = setup_data().await;
    let mut app = test::init_service(App::new().app_data(database_pool).service(task_delete)).await;

    let request = test::TestRequest::delete().uri("/tasks/1000").to_request();
    let response = test::call_service(&mut app, request).await;
    assert!(response.status().is_redirection());
}
