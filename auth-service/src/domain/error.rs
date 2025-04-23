#[derive(Debug)]
pub enum AuthAPIError {
    UserAlreadyExists,
    IncorrectCredentials,
    InvalidCredentials,
    UnexpectedError,
}
