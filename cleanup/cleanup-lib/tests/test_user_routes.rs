mod common;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_session::CookieSession;
use actix_web::{
    cookie::Cookie, dev::ServiceResponse, http::StatusCode, test, web::ServiceConfig, App,
};
use cleanup_lib::users::{
    models::{InsertUser, LoggedUser, LoginUser, UpdateUser, User},
    routes::{
        delete as user_delete, find_all as user_find_all, find_by_id as user_find_by_id,
        insert as user_insert, login, logout, update as user_update,
    },
};
use common::setup_data;
use time::Duration;

macro_rules! pre_insert_user {
    ($app: expr) => {{
        let insert_user = InsertUser {
            valid_username: "yusuke".to_string(),
            valid_password: "toguro".to_string(),
        };

        let insert_user_request = test::TestRequest::post()
            .uri("/users/register")
            .set_json(&insert_user)
            .to_request();
        let insert_user_response = test::call_service(&mut $app, insert_user_request).await;
        assert!(insert_user_response.status().is_success());

        let user: User = test::read_body_json(insert_user_response).await;
        user
    }};
}

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
    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(user_insert);
        cfg.service(user_update);
    };

    let (mut app, bearer_token, cookies) = setup_app!(configure);
    let user = pre_insert_user!(app);

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
    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(user_insert);
        cfg.service(user_update);
    };

    let (mut app, bearer_token, cookies) = setup_app!(configure);
    let user = pre_insert_user!(app);

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
    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(user_insert);
        cfg.service(user_delete);
    };

    let (mut app, bearer_token, cookies) = setup_app!(configure);
    let user = pre_insert_user!(app);

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
    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(user_insert);
        cfg.service(user_delete);
    };

    let (mut app, bearer_token, cookies) = setup_app!(configure);
    let user = pre_insert_user!(app);

    // NOTE(alex): Delete
    let request = test::TestRequest::delete()
        .uri(&format!("/users/{}", user.id + 1000))
        .insert_header(("Authorization".to_string(), bearer_token))
        .cookie(cookies)
        .to_request();
    let response = test::call_service(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
}

#[actix_rt::test]
pub async fn test_user_find_all() {
    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(user_insert);
        cfg.service(user_find_all);
    };

    let (mut app, _, _) = setup_app!(configure);
    let _ = pre_insert_user!(app);

    // NOTE(alex): Find all
    let request = test::TestRequest::get().uri("/users").to_request();
    let response = test::call_service(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::FOUND);
}

#[actix_rt::test]
pub async fn test_user_find_by_id() {
    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(user_insert);
        cfg.service(user_find_by_id);
    };

    let (mut app, _, _) = setup_app!(configure);
    let user = pre_insert_user!(app);

    // NOTE(alex): Find by id
    let request = test::TestRequest::get()
        .uri(&format!("/users/{}", user.id))
        .to_request();
    let response = test::call_service(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::FOUND);
}

#[actix_rt::test]
pub async fn test_user_login() {
    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(user_insert);
        cfg.service(login);
    };

    let (mut app, _, _) = setup_app!(configure);
    let user = pre_insert_user!(app);

    // NOTE(alex): Login
    let request = test::TestRequest::post()
        .uri("/users/login")
        .set_json(&user)
        .to_request();
    let response = test::call_service(&mut app, request).await;

    assert!(response.status().is_success());
}

#[actix_rt::test]
pub async fn test_user_logout() {
    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(user_insert);
        cfg.service(login);
        cfg.service(logout);
    };

    let data = setup_data().await;
    let app = App::new()
        .app_data(data.clone())
        .configure(configure)
        .wrap(IdentityService::new(
            CookieIdentityPolicy::new(&[0; 32])
                .name("auth-cookie")
                .login_deadline(Duration::minutes(10))
                .secure(false),
        ));
    let mut app = test::init_service(app).await;

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
    let login_service_response: ServiceResponse = test::call_service(&mut app, login_request).await;
    assert!(login_service_response.status().is_success());

    let cookies = login_service_response.response().cookies();
    let cookies_str = cookies
        .flat_map(|cookie| cookie.to_string().chars().collect::<Vec<_>>())
        .collect::<String>();

    let logged_user: LoggedUser = test::read_body_json(login_service_response).await;

    let bearer_token = format!("Bearer {}", logged_user.token);
    let cookies = Cookie::parse_encoded(cookies_str).unwrap();

    // NOTE(alex): Logout
    let request = test::TestRequest::delete()
        .uri("/users/logout")
        .insert_header(("Authorization".to_string(), bearer_token))
        .cookie(cookies)
        .to_request();
    let response = test::call_service(&mut app, request).await;

    assert!(response.status().is_success());
}
