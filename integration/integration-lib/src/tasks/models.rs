use actix_web::{
    dev::{JsonBody, Payload},
    FromRequest, HttpRequest, HttpResponse, Responder,
};
use futures::{future::LocalBoxFuture, FutureExt};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use super::{errors::*, *};
use crate::errors::AppError;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub struct Task {
    pub id: i64,
    pub title: String,
    pub details: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct InsertTask {
    pub non_empty_title: String,
    pub details: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateTask {
    pub id: i64,
    pub new_title: String,
    pub details: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QueryTask {
    pub title: String,
    pub details: String,
}

impl InsertTask {
    pub async fn insert(self, db_pool: &SqlitePool) -> Result<Task, AppError> {
        let mut connection = db_pool.acquire().await?;
        let result = sqlx::query(INSERT)
            .bind(&self.non_empty_title)
            .bind(&self.details)
            .execute(&mut connection)
            .await?;

        let task = Task {
            id: result.last_insert_rowid(),
            title: self.non_empty_title,
            details: self.details,
        };

        Ok(task)
    }

    fn validate(self) -> Result<Self, TaskError> {
        if self.non_empty_title.trim().is_empty() {
            Err(TaskError::EmptyTitle)
        } else {
            Ok(self)
        }
    }
}

impl UpdateTask {
    pub async fn update(self, db_pool: &SqlitePool) -> Result<u64, AppError> {
        let mut connection = db_pool.acquire().await?;
        let result = sqlx::query(UPDATE)
            .bind(&self.new_title)
            .bind(&self.details)
            .bind(&self.id)
            .execute(&mut connection)
            .await?;

        Ok(result.rows_affected())
    }

    fn validate(self) -> Result<Self, TaskError> {
        if self.new_title.trim().is_empty() {
            Err(TaskError::EmptyTitle)
        } else {
            Ok(self)
        }
    }
}

impl Task {
    pub async fn delete(db_pool: &SqlitePool, task_id: i64) -> Result<u64, AppError> {
        let mut connection = db_pool.acquire().await?;
        let result = sqlx::query(DELETE)
            .bind(task_id)
            .execute(&mut connection)
            .await?;

        Ok(result.rows_affected())
    }

    pub async fn done(pool: &SqlitePool, task_id: i64) -> Result<i64, AppError> {
        let mut connection = pool.acquire().await?;
        let result = sqlx::query(DONE)
            .bind(task_id)
            .execute(&mut connection)
            .await?;

        Ok(result.last_insert_rowid())
    }

    pub async fn undo(db_pool: &SqlitePool, task_id: i64) -> Result<u64, AppError> {
        let mut connection = db_pool.acquire().await?;
        let result = sqlx::query(UNDO)
            .bind(task_id)
            .execute(&mut connection)
            .await;

        Ok(result?.rows_affected())
    }

    pub async fn find_all(db_pool: &SqlitePool) -> Result<Vec<Self>, AppError> {
        let result = sqlx::query_as(FIND_ALL).fetch_all(db_pool).await?;
        Ok(result)
    }

    pub async fn find_ongoing(db_pool: &SqlitePool) -> Result<Vec<Self>, AppError> {
        let result = sqlx::query_as(FIND_ONGOING).fetch_all(db_pool).await?;
        Ok(result)
    }

    pub async fn find_by_pattern(
        db_pool: &SqlitePool,
        search_pattern: &str,
    ) -> Result<Vec<Self>, AppError> {
        let result = sqlx::query_as(FIND_BY_PATTERN)
            .bind(search_pattern)
            .fetch_all(db_pool)
            .await?;

        Ok(result)
    }

    pub async fn find_by_id(db_pool: &SqlitePool, task_id: i64) -> Result<Option<Self>, AppError> {
        let result = sqlx::query_as(FIND_BY_ID)
            .bind(task_id)
            .fetch_optional(db_pool)
            .await?;

        Ok(result)
    }
}

impl Responder for Task {
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

impl FromRequest for InsertTask {
    type Config = ();

    type Error = AppError;

    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        JsonBody::new(req, payload, None)
            .limit(4056)
            .map(|res: Result<InsertTask, _>| match res {
                Ok(insert_task) => insert_task.validate().map_err(|fail| AppError::from(fail)),
                Err(fail) => Err(AppError::from(fail)),
            })
            .boxed_local()
    }
}

impl FromRequest for UpdateTask {
    type Config = ();

    type Error = AppError;

    type Future = LocalBoxFuture<'static, Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        JsonBody::new(req, payload, None)
            .limit(4056)
            .map(|res: Result<UpdateTask, _>| match res {
                Ok(update_task) => update_task.validate().map_err(|fail| AppError::from(fail)),
                Err(fail) => Err(AppError::from(fail)),
            })
            .boxed_local()
    }
}
