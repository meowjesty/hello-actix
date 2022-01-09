use actix_identity::Identity;
use actix_web::{delete, get, post, put, web, HttpResponse, Responder};
use sqlx::SqlitePool;

use super::{
    errors::UserError,
    models::{InsertUser, LoginUser, UpdateUser, User},
};
use crate::errors::AppError;

#[post("/users/register")]
async fn insert(
    db_pool: web::Data<SqlitePool>,
    input: InsertUser,
) -> Result<impl Responder, AppError> {
    let user = input.insert(db_pool.get_ref()).await?;
    Ok(HttpResponse::Created().json(user))
}

#[put("/users")]
async fn update(
    db_pool: web::Data<SqlitePool>,
    input: UpdateUser,
) -> Result<impl Responder, AppError> {
    let num_modified = input.update(db_pool.get_ref()).await?;

    if num_modified == 0 {
        Ok(HttpResponse::NotModified().finish())
    } else {
        Ok(HttpResponse::Created().body(num_modified.to_string()))
    }
}

#[delete("/users/{id}")]
async fn delete(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let num_modified = User::delete(db_pool.get_ref(), *id).await?;

    if num_modified == 0 {
        Ok(HttpResponse::NotModified().finish())
    } else {
        Ok(HttpResponse::Ok().body(num_modified.to_string()))
    }
}

#[get("/users")]
async fn find_all(db_pool: web::Data<SqlitePool>) -> Result<impl Responder, AppError> {
    let users = User::find_all(db_pool.get_ref()).await?;
    Ok(HttpResponse::Found().json(&users))
}

#[get("/users/{id:\\d+}")]
async fn find_by_id(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let user = User::find_by_id(db_pool.get_ref(), *id).await?;
    Ok(user)
}

#[post("/users/login")]
async fn login(
    db_pool: web::Data<SqlitePool>,
    identity: Identity,
    input: web::Json<LoginUser>,
) -> Result<impl Responder, AppError> {
    let user = input.into_inner().login(&db_pool).await?;
    match user {
        Some(user) => {
            identity.remember(serde_json::to_string_pretty(&user)?);
            Ok(user)
        }
        None => Err(UserError::NotFound(-100000).into()),
    }
}

#[delete("/users/logout")]
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

#[cfg(test)]
mod tests {
    use actix_identity::{CookieIdentityPolicy, IdentityService};
    use actix_web::{
        cookie::Cookie,
        http::StatusCode,
        test,
        web::{self, ServiceConfig},
        App,
    };
    use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
    use time::Duration;

    use super::*;
    use crate::create_database;

    macro_rules! setup_app {
        ($configure: expr) => {{
            let data = setup_data().await;
            let app = App::new()
                .app_data(data.clone())
                .configure($configure)
                .wrap(IdentityService::new(
                    CookieIdentityPolicy::new(&[0; 32])
                        .name("auth-cookie")
                        .login_deadline(Duration::minutes(5))
                        .secure(false),
                ));

            let app = test::init_service(app).await;

            app
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

    async fn setup_data() -> web::Data<Pool<Sqlite>> {
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

    #[actix_rt::test]
    pub async fn test_user_insert_valid_user() {
        let data = setup_data().await;
        let mut app = test::init_service(App::new().app_data(data).service(insert)).await;

        let valid_insert_user = InsertUser {
            valid_username: "yusuke".to_string(),
            valid_password: "toguro".to_string(),
        };

        let request = test::TestRequest::post()
            .uri("/users/register")
            .set_json(&valid_insert_user)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    pub async fn test_user_insert_invalid_user_username() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(insert);
        };

        let mut app = setup_app!(configure);

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
            cfg.service(insert);
            cfg.service(update);
        };

        let mut app = setup_app!(configure);
        let user = pre_insert_user!(app);

        let update_user = UpdateUser {
            id: user.id,
            valid_username: format!("{}_urameshi", user.username),
            valid_password: format!("{}_young.", user.password),
        };

        // NOTE(alex): Update
        let request = test::TestRequest::put()
            .uri("/users")
            .set_json(&update_user)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    pub async fn test_user_update_with_invalid_user_username() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(insert);
            cfg.service(update);
        };

        let mut app = setup_app!(configure);
        let user = pre_insert_user!(app);

        let update_user = UpdateUser {
            id: user.id,
            valid_username: " \n\t".to_string(),
            valid_password: format!("{}_young.", user.password),
        };

        // NOTE(alex): Update
        let request = test::TestRequest::put()
            .uri("/users")
            .set_json(&update_user)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_client_error());
    }

    #[actix_rt::test]
    pub async fn test_user_delete_existing_user() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(insert);
            cfg.service(delete);
        };

        let mut app = setup_app!(configure);
        let user = pre_insert_user!(app);

        // NOTE(alex): Delete
        let request = test::TestRequest::delete()
            .uri(&format!("/users/{}", user.id))
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    pub async fn test_user_delete_non_existent_user() {
        let database_pool = setup_data().await;
        let mut app = test::init_service(App::new().app_data(database_pool).service(delete)).await;

        // NOTE(alex): Delete
        let request = test::TestRequest::delete()
            .uri(&format!("/users/{}", 1000))
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    }

    #[actix_rt::test]
    pub async fn test_user_find_all() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(insert);
            cfg.service(find_all);
        };

        let mut app = setup_app!(configure);
        let _ = pre_insert_user!(app);

        // NOTE(alex): Find all
        let request = test::TestRequest::get().uri("/users").to_request();
        let response = test::call_service(&mut app, request).await;

        assert_eq!(response.status(), StatusCode::FOUND);
    }

    #[actix_rt::test]
    pub async fn test_user_find_by_id() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(insert);
            cfg.service(find_by_id);
        };

        let mut app = setup_app!(configure);
        let user = pre_insert_user!(app);

        // NOTE(alex): Find with id
        let request = test::TestRequest::get()
            .uri(&format!("/users/{}", user.id))
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }

    #[actix_rt::test]
    pub async fn test_user_login() {
        let configure = |cfg: &mut ServiceConfig| {
            cfg.service(insert);
            cfg.service(login);
        };

        let mut app = setup_app!(configure);
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
            cfg.service(insert);
            cfg.service(login);
            cfg.service(logout);
        };

        let mut app = setup_app!(configure);

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

        let cookies = Cookie::parse_encoded(cookies_str).unwrap();

        // NOTE(alex): Logout
        let request = test::TestRequest::delete()
            .uri("/users/logout")
            .cookie(cookies)
            .to_request();
        let response = test::call_service(&mut app, request).await;

        assert!(response.status().is_success());
    }
}
