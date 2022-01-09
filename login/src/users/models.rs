use actix_web::{
    body::BoxBody, dev::Payload, web::JsonBody, FromRequest, HttpRequest, HttpResponse, Responder,
};
use futures::{future::LocalBoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use super::{errors::*, *};
use crate::errors::AppError;

pub(crate) const MIN_USERNAME_LENGTH: usize = 3;
pub(crate) const MIN_PASSWORD_LENGTH: usize = 4;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub(crate) struct User {
    pub(crate) id: i64,
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct InsertUser {
    pub(crate) valid_username: String,
    pub(crate) valid_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UpdateUser {
    pub(crate) id: i64,
    pub(crate) valid_username: String,
    pub(crate) valid_password: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub(crate) struct LoginUser {
    pub(crate) username: String,
    pub(crate) password: String,
}

impl InsertUser {
    pub(crate) async fn insert(self, db_pool: &SqlitePool) -> Result<User, AppError> {
        let mut connection = db_pool.acquire().await?;
        let result = sqlx::query(INSERT)
            .bind(&self.valid_username)
            .bind(&self.valid_password)
            .execute(&mut connection)
            .await?;

        let user = User {
            id: result.last_insert_rowid(),
            username: self.valid_username,
            password: self.valid_password,
        };

        Ok(user)
    }

    fn validate(self) -> Result<Self, UserError> {
        if self.valid_username.trim().is_empty() {
            Err(UserError::EmptyUsername)
        } else if self.valid_username.len() < MIN_USERNAME_LENGTH {
            Err(UserError::UsernameLength)
        } else if self.valid_username.contains(" ") {
            Err(UserError::UsernameInvalidCharacter)
        } else if self.valid_password.trim().is_empty() {
            Err(UserError::EmptyPassword)
        } else if self.valid_password.len() < MIN_PASSWORD_LENGTH {
            Err(UserError::PasswordLength)
        } else if self.valid_password.contains(" ") {
            Err(UserError::PasswordInvalidCharacter)
        } else {
            Ok(self)
        }
    }
}

impl UpdateUser {
    pub(crate) async fn update(self, db_pool: &SqlitePool) -> Result<u64, AppError> {
        let mut connection = db_pool.acquire().await?;
        let result = sqlx::query(UPDATE)
            .bind(&self.valid_username)
            .bind(&self.valid_password)
            .bind(&self.id)
            .execute(&mut connection)
            .await?;

        Ok(result.rows_affected())
    }

    fn validate(self) -> Result<Self, UserError> {
        if self.valid_username.trim().is_empty() {
            Err(UserError::EmptyUsername)
        } else if self.valid_username.len() < MIN_USERNAME_LENGTH {
            Err(UserError::UsernameLength)
        } else if self.valid_username.contains(" ") {
            Err(UserError::UsernameInvalidCharacter)
        } else if self.valid_password.trim().is_empty() {
            Err(UserError::EmptyPassword)
        } else if self.valid_password.len() < MIN_PASSWORD_LENGTH {
            Err(UserError::PasswordLength)
        } else if self.valid_password.contains(" ") {
            Err(UserError::PasswordInvalidCharacter)
        } else {
            Ok(self)
        }
    }
}

impl User {
    pub(crate) async fn delete(db_pool: &SqlitePool, user_id: i64) -> Result<u64, AppError> {
        let mut connection = db_pool.acquire().await?;
        let result = sqlx::query(DELETE)
            .bind(user_id)
            .execute(&mut connection)
            .await?;

        Ok(result.rows_affected())
    }

    pub(crate) async fn find_all(db_pool: &SqlitePool) -> Result<Vec<Self>, AppError> {
        let result = sqlx::query_as(FIND_ALL).fetch_all(db_pool).await?;
        Ok(result)
    }

    pub(crate) async fn find_by_id(
        db_pool: &SqlitePool,
        user_id: i64,
    ) -> Result<Option<Self>, AppError> {
        let result = sqlx::query_as(FIND_BY_ID)
            .bind(user_id)
            .fetch_optional(db_pool)
            .await?;

        Ok(result)
    }
}

impl LoginUser {
    pub(crate) async fn login(self, db_pool: &SqlitePool) -> Result<Option<User>, AppError> {
        let result = sqlx::query_as(LOGIN)
            .bind(self.username)
            .bind(self.password)
            .fetch_optional(db_pool)
            .await?;

        Ok(result)
    }
}

impl Responder for User {
    type Body = BoxBody;

    fn respond_to(self, _req: &HttpRequest) -> HttpResponse<Self::Body> {
        let response = match serde_json::to_string(&self) {
            Ok(body) => {
                // Create response and set content type
                HttpResponse::Ok()
                    .content_type("application/json")
                    .body(body)
            }
            Err(fail) => HttpResponse::from_error(AppError::from(fail)),
        };

        response
    }
}

impl FromRequest for InsertUser {
    type Error = AppError;

    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        JsonBody::new(req, payload, None, false)
            .limit(4056)
            .map(|res: Result<InsertUser, _>| match res {
                Ok(insert_user) => insert_user.validate().map_err(|fail| AppError::from(fail)),
                Err(fail) => Err(AppError::from(fail)),
            })
            .boxed_local()
    }
}

impl FromRequest for UpdateUser {
    type Error = AppError;

    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        JsonBody::new(req, payload, None, false)
            .limit(4056)
            .map(|res: Result<UpdateUser, _>| match res {
                Ok(update_user) => update_user.validate().map_err(|fail| AppError::from(fail)),
                Err(fail) => Err(AppError::from(fail)),
            })
            .boxed_local()
    }
}
