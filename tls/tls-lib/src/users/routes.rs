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
pub async fn insert(
    db_pool: web::Data<SqlitePool>,
    input: InsertUser,
) -> Result<impl Responder, AppError> {
    let user = input.insert(db_pool.get_ref()).await?;
    Ok(HttpResponse::Created().json(user))
}

#[put("/users", wrap = "HttpAuthentication::bearer(validator)")]
pub async fn update(
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
pub async fn delete(
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
pub async fn find_all(db_pool: web::Data<SqlitePool>) -> Result<impl Responder, AppError> {
    let users = User::find_all(db_pool.get_ref()).await?;

    if users.is_empty() {
        Err(UserError::Empty.into())
    } else {
        Ok(HttpResponse::Found().json(&users))
    }
}

#[get("/users/{id:\\d+}")]
pub async fn find_by_id(
    db_pool: web::Data<SqlitePool>,
    id: web::Path<i64>,
) -> Result<impl Responder, AppError> {
    let user = User::find_by_id(db_pool.get_ref(), *id).await?;

    match user {
        Some(user) => Ok(HttpResponse::Found().json(user)),
        None => Err(UserError::NotFound(*id).into()),
    }
}

pub fn create_auth_token(user: &User) -> u64 {
    use std::{
        collections::hash_map::DefaultHasher,
        hash::{Hash, Hasher},
    };

    let mut hasher = DefaultHasher::new();
    user.hash(&mut hasher);
    hasher.finish()
}

#[post("/users/login")]
pub async fn login(
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
pub async fn logout(identity: Identity) -> impl Responder {
    identity.forget();
    HttpResponse::Ok().body("Logged out.")
}

pub fn user_service(cfg: &mut web::ServiceConfig) {
    cfg.service(insert);
    cfg.service(update);
    cfg.service(delete);
    cfg.service(find_all);
    cfg.service(find_by_id);
    cfg.service(login);
    cfg.service(logout);
}
