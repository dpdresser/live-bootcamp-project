#[derive(Debug)]
pub enum AuthAPIError {
    UserAlreadyExists,
    IncorrectCredentials,
    InvalidCredentials,
    MissingToken,
    InvalidToken,
    UnexpectedError,
}
