use thiserror::Error;

use super::models::MIN_USERNAME_LENGTH;

#[derive(Debug, Error)]
pub(crate) enum UserError {
    #[error("`username` field of `User` cannot be empty!")]
    EmptyUsername,

    #[error(
        "`username` field of `User` must be at least {} characters!",
        MIN_USERNAME_LENGTH
    )]
    UsernameLength,

    #[error("`username` field of `User` cannot contain whitespaces!")]
    UsernameInvalidCharacter,

    #[error("`password` field of `User` cannot be empty!")]
    EmptyPassword,

    #[error(
        "`password` field of `User` must be at least {} characters!",
        MIN_USERNAME_LENGTH
    )]
    PasswordLength,

    #[error("`password` field of `User` cannot contain whitespaces!")]
    PasswordInvalidCharacter,

    #[error("Could not find any `User` for id: `{0}`!")]
    NotFound(i64),

    #[error("Failed to login user!")]
    LoginFailed,

    #[error("User is not logged in!")]
    NotLoggedIn,

    #[error("Invalid authorization token!")]
    InvalidToken,

    #[error("Could not find any `User`!")]
    Empty,
}
