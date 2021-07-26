use thiserror::Error;

#[derive(Debug, Error)]
pub enum TaskError {
    #[error("`title` field of `Task` cannot be empty!")]
    EmptyTitle,

    #[error("Could not find any `Task` for id: `{0}`!")]
    NotFound(i64),

    #[error("You have not favorited any `Task` yet!")]
    NoneFavorite,

    #[error("Could not find any `Task`!")]
    Empty,
}
