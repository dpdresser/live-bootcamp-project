use secrecy::Secret;

use crate::domain::{Email, Password};

#[derive(Clone)]
pub struct User {
    email: Email,
    password: Password,
    requires_2fa: bool,
}

impl User {
    pub fn new(email: Email, password: Password, requires_2fa: bool) -> Self {
        Self {
            email,
            password,
            requires_2fa,
        }
    }

    pub fn email(&self) -> &Secret<String> {
        self.email.as_ref()
    }

    pub fn password(&self) -> &Secret<String> {
        self.password.as_ref()
    }

    pub fn requires_2fa(&self) -> bool {
        self.requires_2fa
    }
}
