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
        Ok(HttpResponse::NotModified().finish())
    } else {
        Ok(HttpResponse::Created().body(num_modified.to_string()))
    }
}

#[delete("/users/{id}", wrap = "HttpAuthentication::bearer(validator)")]
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
    Ok(user)
}

#[post("/users/login", wrap = "HttpAuthentication::bearer(validator)")]
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

#[cfg(test)]
mod tests {
    use std::env;

    use actix_web::{test, web, App};
    use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};

    use super::*;
    use crate::create_database;

    async fn setup_data() -> web::Data<Pool<Sqlite>> {
        let db_options = sqlx::sqlite::SqliteConnectOptions::new()
            .filename(env!("DATABASE_FILE"))
            .create_if_missing(true);

        let database_pool = SqlitePoolOptions::new()
            .max_connections(20)
            .connect_with(db_options)
            .await
            .unwrap();

        create_database(&database_pool).await.unwrap();

        web::Data::new(database_pool)
    }

    #[actix_rt::test]
    async fn test_insert_valid() {
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
    async fn test_insert_invalid_username() {
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
    async fn test_update_valid() {
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
    async fn test_update_invalid_username() {
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
    async fn test_delete() {
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
    async fn test_delete_nothing() {
        let database_pool = setup_data().await;
        let mut app = test::init_service(App::new().app_data(database_pool).service(delete)).await;

        let request = test::TestRequest::delete().uri("/users/1000").to_request();
        let response = test::call_service(&mut app, request).await;
        println!("{:#?}", response);
        assert!(response.status().is_redirection());
    }
}
