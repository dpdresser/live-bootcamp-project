#[derive(Debug, Clone, PartialEq)]
pub struct Email(String);

#[derive(Debug)]
pub struct ParseEmailError;

impl Email {
    pub fn parse(email: &str) -> Result<Self, ParseEmailError> {
        if email.contains("@") && !email.is_empty() {
            Ok(Email(email.to_string()))
        } else {
            Err(ParseEmailError)
        }
    }
}

impl std::convert::AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_email_succeeds() {
        let email = "test@example.com";
        let email = Email::parse(email);

        assert!(email.is_ok());
        assert_eq!(email.unwrap().as_ref(), "test@example.com");
    }

    #[test]
    fn parse_email_fails() {
        let email = "testexample.com";
        let email = Email::parse(email);

        assert!(email.is_err());
    }

    #[test]
    fn parse_empty_email_fails() {
        let email = "";
        let email = Email::parse(email);

        assert!(email.is_err());
    }
}
