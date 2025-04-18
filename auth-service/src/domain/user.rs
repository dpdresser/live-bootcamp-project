use crate::domain::{Email, Password};

use super::AuthAPIError;

#[derive(Clone, Debug, PartialEq)]
pub struct User {
    pub email: Email,
    pub password: Password,
    pub requires_2fa: bool,
}

impl User {
    pub fn new(email: &str, password: &str, requires_2fa: bool) -> Result<Self, AuthAPIError> {
        if let (Ok(email), Ok(password)) = (Email::parse(email), Password::parse(password)) {
            Ok(Self {
                email,
                password,
                requires_2fa,
            })
        } else {
            Err(AuthAPIError::InvalidCredentials)
        }
    }
}
