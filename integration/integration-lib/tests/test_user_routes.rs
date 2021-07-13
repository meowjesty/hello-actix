mod common;

use std::env;

use actix_web::{test, web, App};
use common::setup_data;
use integration_lib::users::{
    models::{InsertUser, UpdateUser, User},
    routes::{delete, insert, update},
};

#[actix_rt::test]
pub async fn test_insert_valid() {
    let data = setup_data().await;
    let mut app = test::init_service(App::new().app_data(data.clone()).service(insert)).await;

    let valid_insert = InsertUser {
        valid_username: "valid_user".to_string(),
        valid_password: "valid_password".to_string(),
    };

    let request = test::TestRequest::post()
        .uri("/users")
        .set_json(&valid_insert)
        .to_request();
    let response = test::call_service(&mut app, request).await;

    assert!(response.status().is_success());
}

#[actix_rt::test]
pub async fn test_insert_invalid_username() {
    let database_pool = setup_data().await;
    let mut app = test::init_service(App::new().app_data(database_pool).service(insert)).await;

    let invalid_insert = InsertUser {
        valid_username: " \n\t".to_string(),
        valid_password: "valid_password".to_string(),
    };

    let request = test::TestRequest::post()
        .uri("/users")
        .set_json(&invalid_insert)
        .to_request();
    let response = test::call_service(&mut app, request).await;

    assert!(response.status().is_client_error());
}

#[actix_rt::test]
pub async fn test_update_valid() {
    let database_pool = setup_data().await;
    let mut app = test::init_service(
        App::new()
            .app_data(database_pool)
            .service(insert)
            .service(update),
    )
    .await;

    let user = InsertUser {
        valid_username: "valid_username".to_string(),
        valid_password: "valid_password".to_string(),
    };

    // NOTE(alex): Insert before updating.
    let request = test::TestRequest::post()
        .uri("/users")
        .set_json(&user)
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert!(response.status().is_success());

    // TODO(alex) [low] 2021-06-21: Why doesn't it implement `try_into` for string?
    let user: User = match response.into_body() {
        actix_web::body::AnyBody::Bytes(bytes) => {
            serde_json::from_slice(&bytes).expect("Failed deserializing created user!")
        }
        _ => panic!("Unexpected body!"),
    };
    let valid_update = UpdateUser {
        id: user.id,
        valid_username: format!("{}updated", user.username),
        valid_password: format!("{}updated", user.password),
    };

    // NOTE(alex): Update
    let request = test::TestRequest::put()
        .uri("/users")
        .set_json(&valid_update)
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert!(response.status().is_success());
}

#[actix_rt::test]
pub async fn test_update_invalid_username() {
    let database_pool = setup_data().await;
    let mut app = test::init_service(
        App::new()
            .app_data(database_pool)
            .service(insert)
            .service(update),
    )
    .await;

    let user = InsertUser {
        valid_username: "valid_username".to_string(),
        valid_password: "valid_password".to_string(),
    };

    // NOTE(alex): Insert before updating.
    let request = test::TestRequest::post()
        .uri("/users")
        .set_json(&user)
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert!(response.status().is_success());

    // TODO(alex) [low] 2021-06-21: Why doesn't it implement `try_into` for string?
    let user: User = match response.into_body() {
        actix_web::body::AnyBody::Bytes(bytes) => {
            serde_json::from_slice(&bytes).expect("Failed deserializing created user!")
        }
        _ => panic!("Unexpected body!"),
    };
    let invalid_update = UpdateUser {
        id: user.id,
        valid_username: " \n\t".to_string(),
        valid_password: format!("{}updated", user.password),
    };

    // NOTE(alex): Update
    let request = test::TestRequest::put()
        .uri("/users")
        .set_json(&invalid_update)
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert!(response.status().is_client_error());
}

#[actix_rt::test]
pub async fn test_delete() {
    let database_pool = setup_data().await;
    let mut app = test::init_service(
        App::new()
            .app_data(database_pool)
            .service(insert)
            .service(delete),
    )
    .await;

    let user = InsertUser {
        valid_username: "valid_username".to_string(),
        valid_password: "valid_password".to_string(),
    };

    // NOTE(alex): Insert
    let request = test::TestRequest::post()
        .uri("/users")
        .set_json(&user)
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert!(response.status().is_success());

    // NOTE(alex): Delete
    let request = test::TestRequest::delete()
        .uri("/users/1")
        // TODO(alex) [low] 2021-06-06: Why doesn't this work?
        // .uri("/users")
        // .param("id", "1")
        .to_request();

    let response = test::call_service(&mut app, request).await;
    assert!(response.status().is_success());
}

#[actix_rt::test]
pub async fn test_delete_nothing() {
    let database_pool = setup_data().await;
    let mut app = test::init_service(App::new().app_data(database_pool).service(delete)).await;

    let request = test::TestRequest::delete().uri("/users/1000").to_request();
    let response = test::call_service(&mut app, request).await;
    println!("{:#?}", response);
    assert!(response.status().is_redirection());
}
