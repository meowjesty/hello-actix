mod common;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_session::CookieSession;
use actix_web::{
    cookie::Cookie, dev::ServiceResponse, http::StatusCode, test, web::ServiceConfig, App,
};
use common::setup_data;
use integration_lib::users::{
    models::{InsertUser, LoggedUser, LoginUser, UpdateUser, User},
    routes::{delete as user_delete, insert as user_insert, login, update as user_update},
};
use time::Duration;

// TODO(alex) [high] 2021-07-15: Some routes do not have dedicated tests yet.

#[actix_rt::test]
pub async fn test_user_insert_valid_user() {
    let data = setup_data().await;
    let app = App::new().app_data(data.clone()).configure(|cfg| {
        cfg.service(user_insert);
    });
    let mut app = test::init_service(app).await;

    let insert_user = InsertUser {
        valid_username: "yusuke".to_string(),
        valid_password: "toguro".to_string(),
    };

    let request = test::TestRequest::post()
        .uri("/users/register")
        .set_json(&insert_user)
        .to_request();
    let response = test::call_service(&mut app, request).await;

    assert!(response.status().is_success());
}

#[actix_rt::test]
pub async fn test_user_insert_invalid_username() {
    let data = setup_data().await;
    let app = App::new().app_data(data.clone()).configure(|cfg| {
        cfg.service(user_insert);
    });
    let mut app = test::init_service(app).await;

    let invalid_insert_user = InsertUser {
        valid_username: " \n\t".to_string(),
        valid_password: "toguro".to_string(),
    };

    let request = test::TestRequest::post()
        .uri("/users/register")
        .set_json(&invalid_insert_user)
        .to_request();
    let response = test::call_service(&mut app, request).await;

    assert!(response.status().is_client_error());
}

#[actix_rt::test]
pub async fn test_user_update_valid_user() {
    let insert_user = InsertUser {
        valid_username: "yusuke".to_string(),
        valid_password: "toguro".to_string(),
    };

    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(user_insert);
        cfg.service(user_update);
    };

    let (mut app, bearer_token, cookies) = setup_app!(configure);

    let insert_user_request = test::TestRequest::post()
        .uri("/users/register")
        .insert_header(("Authorization".to_string(), bearer_token.clone()))
        .cookie(cookies.clone())
        .set_json(&insert_user)
        .to_request();
    let insert_user_response = test::call_service(&mut app, insert_user_request).await;
    assert!(insert_user_response.status().is_success());

    let user: User = test::read_body_json(insert_user_response).await;
    let update_user = UpdateUser {
        id: user.id,
        valid_username: format!("{}_urameshi", user.username),
        valid_password: format!("{}_young", user.password),
    };

    // NOTE(alex): Update
    let request = test::TestRequest::put()
        .uri("/users")
        .insert_header(("Authorization".to_string(), bearer_token))
        .cookie(cookies)
        .set_json(&update_user)
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert!(response.status().is_success());
}

#[actix_rt::test]
pub async fn test_user_update_with_invalid_username() {
    let insert_user = InsertUser {
        valid_username: "yusuke".to_string(),
        valid_password: "toguro".to_string(),
    };

    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(user_insert);
        cfg.service(user_update);
    };

    let (mut app, bearer_token, cookies) = setup_app!(configure);

    let insert_user_request = test::TestRequest::post()
        .uri("/users/register")
        .insert_header(("Authorization".to_string(), bearer_token.clone()))
        .cookie(cookies.clone())
        .set_json(&insert_user)
        .to_request();
    let insert_user_response = test::call_service(&mut app, insert_user_request).await;
    assert!(insert_user_response.status().is_success());

    let user: User = test::read_body_json(insert_user_response).await;
    let update_user = UpdateUser {
        id: user.id,
        valid_username: " \n\t".to_string(),
        valid_password: format!("{}_young", user.password),
    };

    // NOTE(alex): Update
    let request = test::TestRequest::put()
        .uri("/users")
        .insert_header(("Authorization".to_string(), bearer_token))
        .cookie(cookies)
        .set_json(&update_user)
        .to_request();
    let response = test::call_service(&mut app, request).await;
    assert!(response.status().is_client_error());
}

#[actix_rt::test]
pub async fn test_user_delete_existing_user() {
    let insert_user = InsertUser {
        valid_username: "yusuke".to_string(),
        valid_password: "toguro".to_string(),
    };

    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(user_insert);
        cfg.service(user_delete);
    };

    let (mut app, bearer_token, cookies) = setup_app!(configure);

    let insert_user_request = test::TestRequest::post()
        .uri("/users/register")
        .insert_header(("Authorization".to_string(), bearer_token.clone()))
        .cookie(cookies.clone())
        .set_json(&insert_user)
        .to_request();
    let insert_user_response = test::call_service(&mut app, insert_user_request).await;
    assert!(insert_user_response.status().is_success());

    let user: User = test::read_body_json(insert_user_response).await;

    // NOTE(alex): Delete
    let request = test::TestRequest::delete()
        .uri(&format!("/users/{}", user.id))
        .insert_header(("Authorization".to_string(), bearer_token))
        .cookie(cookies)
        .to_request();
    let response = test::call_service(&mut app, request).await;

    assert!(response.status().is_success());
}

#[actix_rt::test]
pub async fn test_user_delete_non_existant_user() {
    let insert_user = InsertUser {
        valid_username: "yusuke".to_string(),
        valid_password: "toguro".to_string(),
    };

    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(user_insert);
        cfg.service(user_delete);
    };

    let (mut app, bearer_token, cookies) = setup_app!(configure);

    let insert_user_request = test::TestRequest::post()
        .uri("/users/register")
        .insert_header(("Authorization".to_string(), bearer_token.clone()))
        .cookie(cookies.clone())
        .set_json(&insert_user)
        .to_request();
    let insert_user_response = test::call_service(&mut app, insert_user_request).await;
    assert!(insert_user_response.status().is_success());

    let user: User = test::read_body_json(insert_user_response).await;

    // NOTE(alex): Delete
    let request = test::TestRequest::delete()
        .uri(&format!("/users/{}", user.id + 1000))
        .insert_header(("Authorization".to_string(), bearer_token))
        .cookie(cookies)
        .to_request();
    let response = test::call_service(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
}
