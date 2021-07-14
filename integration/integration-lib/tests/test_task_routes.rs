mod common;

use std::convert::TryFrom;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_session::CookieSession;
use actix_web::{
    cookie::Cookie,
    dev::{AnyBody, Service, ServiceResponse},
    http::header::{self, Header, IntoHeaderPair},
    test,
    web::{self},
    App, HttpResponse,
};
use common::setup_data;
use integration_lib::{
    tasks::{
        models::{InsertTask, Task, UpdateTask},
        routes::{delete as task_delete, insert as task_insert, update as task_update},
    },
    users::{
        models::{InsertUser, LoggedUser, LoginUser, User},
        routes::{insert as user_insert, login},
    },
};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use time::Duration;

#[actix_rt::test]
pub async fn test_task_insert_valid_task() {
    // TODO(alex) [high] 2021-07-14: Everything up to the last request creation could be extracted
    // from here, a macro will probably do the trick.
    let data = setup_data().await;
    let app = App::new()
        .app_data(data.clone())
        .service(task_insert)
        .service(user_insert)
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
        let register_user_service_response: ServiceResponse =
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
        let login_service_response: ServiceResponse =
            test::call_service(&mut app, login_request).await;
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
        details: "It's a good show.".to_string(),
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
