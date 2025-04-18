#[derive(Debug, Clone, PartialEq)]
pub struct Password(String);

#[derive(Debug)]
pub struct ParsePasswordError;

impl Password {
    pub fn parse(password: &str) -> Result<Self, ParsePasswordError> {
        if password.len() > 7 {
            Ok(Password(password.to_string()))
        } else {
            Err(ParsePasswordError)
        }
    }
}

impl std::convert::AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_password_succeeds() {
        let password = "password123";
        let password = Password::parse(password);

        assert!(password.is_ok());
        assert_eq!(password.unwrap().as_ref(), "password123");
    }

    #[test]
    fn parse_password_fails() {
        let password = "pass";
        let password = Password::parse(password);

        assert!(password.is_err());
    }
}
