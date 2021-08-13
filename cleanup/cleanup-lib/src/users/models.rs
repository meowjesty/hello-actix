use std::convert::TryFrom;

use actix_identity::Identity;
use actix_web::{
    dev::{JsonBody, Payload},
    FromRequest, HttpRequest, HttpResponse, Responder,
};
use futures::{future::LocalBoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use super::{errors::*, *};
use crate::errors::AppError;

pub const MIN_USERNAME_LENGTH: usize = 3;
pub const MIN_PASSWORD_LENGTH: usize = 4;

#[derive(Hash, PartialEq, Eq, Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertUser {
    pub valid_username: String,
    pub valid_password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateUser {
    pub id: i64,
    pub valid_username: String,
    pub valid_password: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct LoggedUser {
    pub id: i64,
    pub username: String,
    pub password: String,
    pub token: u64,
}

impl InsertUser {
    pub async fn insert(self, db_pool: &SqlitePool) -> Result<User, AppError> {
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
    pub async fn update(self, db_pool: &SqlitePool) -> Result<u64, AppError> {
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
    pub async fn delete(db_pool: &SqlitePool, user_id: i64) -> Result<u64, AppError> {
        let mut connection = db_pool.acquire().await?;
        let result = sqlx::query(DELETE)
            .bind(user_id)
            .execute(&mut connection)
            .await?;

        Ok(result.rows_affected())
    }

    pub async fn find_all(db_pool: &SqlitePool) -> Result<Vec<Self>, AppError> {
        let result = sqlx::query_as(FIND_ALL).fetch_all(db_pool).await?;
        Ok(result)
    }

    pub async fn find_by_id(db_pool: &SqlitePool, user_id: i64) -> Result<Option<Self>, AppError> {
        let result = sqlx::query_as(FIND_BY_ID)
            .bind(user_id)
            .fetch_optional(db_pool)
            .await?;

        Ok(result)
    }

    pub fn to_logged(self, token: u64) -> LoggedUser {
        LoggedUser {
            id: self.id,
            username: self.username,
            password: self.password,
            token,
        }
    }
}

impl LoginUser {
    pub async fn login(self, db_pool: &SqlitePool) -> Result<Option<User>, AppError> {
        let result = sqlx::query_as(LOGIN)
            .bind(self.username)
            .bind(self.password)
            .fetch_optional(db_pool)
            .await?;

        Ok(result)
    }
}

impl LoggedUser {
    pub fn get_logged_user(identity: Identity) -> Result<LoggedUser, AppError> {
        let logged_user = identity
            .identity()
            .ok_or(UserError::NotLoggedIn.into())
            .and_then(|user_string| LoggedUser::try_from(user_string))?;

        Ok(logged_user)
    }
}

impl TryFrom<String> for LoggedUser {
    type Error = AppError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let logged_user = serde_json::from_str(&value)?;
        Ok(logged_user)
    }
}

impl Responder for User {
    fn respond_to(self, _req: &HttpRequest) -> HttpResponse {
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
    type Config = ();

    type Error = AppError;

    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        JsonBody::new(req, payload, None)
            .limit(4056)
            .map(|res: Result<InsertUser, _>| match res {
                Ok(insert_user) => insert_user.validate().map_err(|fail| AppError::from(fail)),
                Err(fail) => Err(AppError::from(fail)),
            })
            .boxed_local()
    }
}

impl FromRequest for UpdateUser {
    type Config = ();

    type Error = AppError;

    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        JsonBody::new(req, payload, None)
            .limit(4056)
            .map(|res: Result<UpdateUser, _>| match res {
                Ok(update_user) => update_user.validate().map_err(|fail| AppError::from(fail)),
                Err(fail) => Err(AppError::from(fail)),
            })
            .boxed_local()
    }
}
