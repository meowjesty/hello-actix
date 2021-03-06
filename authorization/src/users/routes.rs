use actix_identity::Identity;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use actix_web_httpauth::middleware::HttpAuthentication;
use sqlx::SqlitePool;

use super::{
    errors::UserError,
    models::{InsertUser, LoginUser, UpdateUser, User},
};
use crate::{errors::AppError, validator};

#[post("/users/register")]
async fn insert(
    db_pool: web::Data<SqlitePool>,
    input: InsertUser,
) -> Result<impl Responder, AppError> {
    let user = input.insert(db_pool.get_ref()).await?;
    Ok(HttpResponse::Created().json(user))
}

#[put("/users", wrap = "HttpAuthentication::bearer(validator)")]
async fn update(
    db_pool: web::Data<SqlitePool>,
    input: UpdateUser,
) -> Result<impl Responder, AppError> {
    let num_modified = input.update(db_pool.get_ref()).await?;

    if num_modified == 0 {
        Ok(HttpResponse::NotModified().body("No users were updated."))
    } else {
        Ok(HttpResponse::Ok().body(format!("Updated {} users.", num_modified)))
    }
}

#[delete("/users/{id:\\d+}", wrap = "HttpAuthentication::bearer(validator)")]
async fn delete(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let num_modified = User::delete(db_pool.get_ref(), *id).await?;

    if num_modified == 0 {
        Ok(HttpResponse::NotModified().body("No users were deleted."))
    } else {
        Ok(HttpResponse::Ok().body(format!("Deleted {} users.", num_modified)))
    }
}

#[get("/users")]
async fn find_all(db_pool: web::Data<SqlitePool>) -> Result<impl Responder, AppError> {
    let users = User::find_all(db_pool.get_ref()).await?;

    if users.is_empty() {
        Err(UserError::Empty.into())
    } else {
        Ok(HttpResponse::Found().json(&users))
    }
}

#[get("/users/{id:\\d+}")]
async fn find_by_id(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let user = User::find_by_id(db_pool.get_ref(), *id).await?;

    match user {
        Some(user) => Ok(HttpResponse::Found().json(user)),
        None => Err(UserError::NotFound(*id).into()),
    }
}

pub(crate) fn create_auth_token(user: &User) -> u64 {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    let mut hasher = DefaultHasher::new();
    user.hash(&mut hasher);
    hasher.finish()
}

#[post("/users/login")]
async fn login(
    db_pool: web::Data<SqlitePool>,
    identity: Identity,
    input: web::Json<LoginUser>,
) -> Result<impl Responder, AppError> {
    let login_user = input.into_inner();
    let user = login_user.login(&db_pool).await?;
    match user {
        Some(user) => {
            let auth_token = create_auth_token(&user);
            let logged_user = user.to_logged(auth_token);

            // NOTE(alex): We'll use this identity cookie to check if the user is logged in for
            // routes that require it.
            identity.remember(serde_json::to_string_pretty(&logged_user)?);

            let response: HttpResponse = HttpResponse::Ok()
                .append_header(("X-Auth-Token", auth_token.to_string()))
                .json(logged_user);
            Ok(response)
        }
        None => Err(UserError::LoginFailed.into()),
    }
}

#[delete("/users/logout", wrap = "HttpAuthentication::bearer(validator)")]
async fn logout(identity: Identity) -> impl Responder {
    identity.forget();
    HttpResponse::Ok().body("Logged out.")
}

pub(crate) fn user_service(cfg: &mut web::ServiceConfig) {
    cfg.service(insert);
    cfg.service(update);
    cfg.service(delete);
    cfg.service(find_all);
    cfg.service(find_by_id);
    cfg.service(login);
    cfg.service(logout);
}

// WARNING(alex): Please ignore these tests for now, we'll take a better look at them on the
// `integration` project!
#[cfg(test)]
mod tests {
    use actix_identity::{CookieIdentityPolicy, IdentityService};
    use actix_session::CookieSession;
    use actix_web::{
        cookie::Cookie,
        http::StatusCode,
        test,
        web::{self, ServiceConfig},
        App,
    };
    use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
    use time::Duration;

    use crate::{
        create_database,
        users::{
            models::{InsertUser, LoggedUser, LoginUser, UpdateUser, User},
            routes::{
                delete as user_delete, find_all as user_find_all, find_by_id as user_find_by_id,
                insert as user_insert, login, logout, update as user_update,
            },
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
    // Running them in parallel may fail
    macro_rules! setup_app {
        ($configure: expr) => {{
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
        let app =
            App::new()
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
        let login_service_response =
            test::call_service(&mut app, login_request).await;
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
}
