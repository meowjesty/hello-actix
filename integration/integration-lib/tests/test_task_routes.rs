mod common;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use actix_session::CookieSession;
use actix_web::{
    cookie::Cookie, dev::ServiceResponse, http::StatusCode, test, web::ServiceConfig, App,
};
use common::setup_data;
use integration_lib::{
    tasks::{
        models::{InsertTask, Task, UpdateTask},
        routes::{
            delete as task_delete, done as task_done, favorite, find_all as task_find_all,
            find_by_id as task_find_by_id, find_by_pattern as task_find_by_pattern, find_favorite,
            find_ongoing, insert as task_insert, undo as task_undo, update as task_update,
        },
    },
    users::{
        models::{InsertUser, LoggedUser, LoginUser, User},
        routes::{find_by_id as user_find_by_id, insert as user_insert, login},
    },
};
use time::Duration;

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
    let task_favorite_response: ServiceResponse =
        test::call_service(&mut app, task_favorite_request).await;
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
