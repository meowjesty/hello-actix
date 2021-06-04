use serde::{Deserialize, Serialize};
use sqlx::{FromRow, SqlitePool};

use crate::errors::TaskError;

#[derive(Clone, Debug, Serialize, Deserialize, FromRow)]
pub(crate) struct Task {
    id: u64,
    title: String,
    details: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct InsertTask {
    non_empty_title: String,
    details: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct UpdateTask {
    id: u64,
    new_title: String,
    details: String,
}

impl InsertTask {
    pub(crate) async fn insert(&self, db_pool: &SqlitePool) -> Result<Task, TaskError> {
        unimplemented!()
    }
}

impl UpdateTask {
    pub(crate) async fn update(&self, db_pool: &SqlitePool) -> Result<Task, TaskError> {
        unimplemented!()
    }
}
