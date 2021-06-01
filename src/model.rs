use actix_web::{Error, HttpRequest, HttpResponse, Responder};
use futures::future::{ready, Ready};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use crate::errors::TodoError;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub(crate) struct Todo {
    id: i64,
    task: String,
    details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct InputTodo {
    task: String,
    details: String,
}

impl Todo {
    const CREATE_DATABASE: &'static str =
        include_str!("./../databases/queries/create_database.sql");
    const FIND_ONGOING: &'static str = include_str!("./../databases/queries/find_ongoing.sql");
    const FIND_ALL: &'static str = include_str!("./../databases/queries/find_all.sql");
    const FIND_BY_ID: &'static str = include_str!("./../databases/queries/find_by_id.sql");
    const INSERT: &'static str = include_str!("./../databases/queries/insert.sql");
    const UPDATE: &'static str = include_str!("./../databases/queries/update.sql");
    const DELETE: &'static str = include_str!("./../databases/queries/delete.sql");

    const DONE: &'static str = include_str!("./../databases/queries/done.sql");
    const UNDO: &'static str = include_str!("./../databases/queries/undo.sql");

    pub(crate) async fn create_database(pool: &SqlitePool) -> Result<u64, TodoError> {
        let mut connection = pool.acquire().await?;

        let result = sqlx::query(Self::CREATE_DATABASE)
            .execute(&mut connection)
            .await?;
        Ok(result.rows_affected())
    }

    pub(crate) async fn find_ongoing(pool: &SqlitePool) -> Result<Vec<Todo>, TodoError> {
        let result = sqlx::query_as(Self::FIND_ONGOING).fetch_all(pool).await?;

        Ok(result)
    }

    pub(crate) async fn find_all(pool: &SqlitePool) -> Result<Vec<Todo>, TodoError> {
        let result = sqlx::query_as(Self::FIND_ALL).fetch_all(pool).await?;

        Ok(result)
    }

    pub(crate) async fn find_by_id(pool: &SqlitePool, id: i64) -> Result<Option<Todo>, TodoError> {
        let result = sqlx::query_as(Self::FIND_BY_ID)
            .bind(id)
            .fetch_optional(pool)
            .await?;

        Ok(result)
    }

    pub(crate) async fn create(pool: &SqlitePool, input: &InputTodo) -> Result<i64, TodoError> {
        let mut connection = pool.acquire().await.unwrap();
        let result = sqlx::query(Self::INSERT)
            .bind(&input.task)
            .bind(&input.details)
            .execute(&mut connection)
            .await?;

        Ok(result.last_insert_rowid())
    }

    pub(crate) async fn update(
        pool: &SqlitePool,
        id: i64,
        input: &InputTodo,
    ) -> Result<u64, TodoError> {
        let mut connection = pool.acquire().await.unwrap();
        let result = sqlx::query(Self::UPDATE)
            .bind(&input.task)
            .bind(&input.details)
            .bind(id)
            .execute(&mut connection)
            .await?;

        Ok(result.rows_affected())
    }

    pub(crate) async fn delete(pool: &SqlitePool, id: i64) -> Result<u64, TodoError> {
        let mut connection = pool.acquire().await.unwrap();
        let result = sqlx::query(Self::DELETE)
            .bind(id)
            .execute(&mut connection)
            .await?;

        Ok(result.rows_affected())
    }

    pub(crate) async fn done(pool: &SqlitePool, id: i64) -> Result<i64, TodoError> {
        let mut connection = pool.acquire().await.unwrap();
        let result = sqlx::query(Self::DONE)
            .bind(id)
            .execute(&mut connection)
            .await?;

        Ok(result.last_insert_rowid())
    }

    pub(crate) async fn undo(pool: &SqlitePool, id: i64) -> Result<u64, TodoError> {
        let mut connection = pool.acquire().await.unwrap();
        let result = sqlx::query(Self::UNDO)
            .bind(id)
            .execute(&mut connection)
            .await?;

        Ok(result.rows_affected())
    }
}

impl Responder for Todo {
    fn respond_to(self, _req: &HttpRequest) -> HttpResponse {
        let response = match serde_json::to_string(&self) {
            Ok(body) => {
                // Create response and set content type
                HttpResponse::Ok()
                    .content_type("application/json")
                    .body(body)
            }
            Err(fail) => HttpResponse::from_error(TodoError::from(fail).into()),
        };

        response
    }
}
