mod common;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_web::{
    cookie::Cookie, dev::ServiceResponse, http::StatusCode, test, web::ServiceConfig, App,
};
use common::setup_data;
use integration_lib::{
    tasks::{
        models::{InsertTask, Task, UpdateTask},
        routes::{delete as task_delete, insert as task_insert, update as task_update},
    },
    users::{
        models::{InsertUser, LoggedUser, LoginUser, User},
        routes::{find_by_id as user_find_by_id, insert as user_insert, login},
    },
};
use time::Duration;

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
        details: "Good show.".to_string(),
    };

    let request = test::TestRequest::post()
        .uri("/tasks")
        .insert_header(("Authorization".to_string(), bearer_token))
        .cookie(cookies)
        .set_json(&valid_insert_task)
        .to_request();
    // NOTE(alex): `response` will be of `uknown` type in rust-analyzer. Its concrete type is:
    // ServiceResponse
    let response = test::call_service(&mut app, request).await;

    assert!(response.status().is_success());
}

#[actix_rt::test]
pub async fn test_task_insert_invalid_task_title() {
    let invalid_insert_task = InsertTask {
        non_empty_title: " \n\t".to_string(),
        details: "Good show.".to_string(),
    };

    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(task_insert);
    };

    let (mut app, bearer_token, cookies) = setup_app!(configure);

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
    let insert_task = InsertTask {
        non_empty_title: "Re-watch Cowboy Bebop".to_string(),
        details: "Good show.".to_string(),
    };

    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(task_insert);
        cfg.service(task_update);
    };

    let (mut app, bearer_token, cookies) = setup_app!(configure);

    let insert_task_request = test::TestRequest::post()
        .uri("/tasks")
        .insert_header(("Authorization".to_string(), bearer_token.clone()))
        .cookie(cookies.clone())
        .set_json(&insert_task)
        .to_request();
    let insert_task_response = test::call_service(&mut app, insert_task_request).await;
    assert!(insert_task_response.status().is_success());

    let task: Task = test::read_body_json(insert_task_response).await;
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
    let insert_task = InsertTask {
        non_empty_title: "Re-watch Cowboy Bebop".to_string(),
        details: "Good show.".to_string(),
    };

    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(task_insert);
        cfg.service(task_update);
    };

    let (mut app, bearer_token, cookies) = setup_app!(configure);

    let insert_task_request = test::TestRequest::post()
        .uri("/tasks")
        .insert_header(("Authorization".to_string(), bearer_token.clone()))
        .cookie(cookies.clone())
        .set_json(&insert_task)
        .to_request();
    let insert_task_response = test::call_service(&mut app, insert_task_request).await;
    assert!(insert_task_response.status().is_success());

    let task: Task = test::read_body_json(insert_task_response).await;
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
    let insert_task = InsertTask {
        non_empty_title: "Re-watch Cowboy Bebop".to_string(),
        details: "Good show.".to_string(),
    };

    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(task_insert);
        cfg.service(task_delete);
    };

    let (mut app, bearer_token, cookies) = setup_app!(configure);

    let insert_task_request = test::TestRequest::post()
        .uri("/tasks")
        .insert_header(("Authorization".to_string(), bearer_token.clone()))
        .cookie(cookies.clone())
        .set_json(&insert_task)
        .to_request();
    let insert_task_response = test::call_service(&mut app, insert_task_request).await;
    assert!(insert_task_response.status().is_success());

    let task: Task = test::read_body_json(insert_task_response).await;

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
pub async fn test_task_delete_non_existant_task() {
    let insert_task = InsertTask {
        non_empty_title: "Re-watch Cowboy Bebop".to_string(),
        details: "Good show.".to_string(),
    };

    let configure = |cfg: &mut ServiceConfig| {
        cfg.service(task_insert);
        cfg.service(task_delete);
    };

    let (mut app, bearer_token, cookies) = setup_app!(configure);

    let insert_task_request = test::TestRequest::post()
        .uri("/tasks")
        .insert_header(("Authorization".to_string(), bearer_token.clone()))
        .cookie(cookies.clone())
        .set_json(&insert_task)
        .to_request();
    let insert_task_response = test::call_service(&mut app, insert_task_request).await;
    assert!(insert_task_response.status().is_success());

    let task: Task = test::read_body_json(insert_task_response).await;

    // NOTE(alex): Delete
    let request = test::TestRequest::delete()
        .uri(&format!("/tasks/{}", task.id + 1000))
        .insert_header(("Authorization".to_string(), bearer_token))
        .cookie(cookies)
        .to_request();
    let response: ServiceResponse = test::call_service(&mut app, request).await;

    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
}
