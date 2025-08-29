use color_eyre::eyre::{eyre, Result};
use validator::Validate;

#[derive(Debug, Clone, Validate, Hash, Eq, PartialEq)]
pub struct Email {
    #[validate(email)]
    email: String,
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.email
    }
}

impl Email {
    pub fn parse(email: &str) -> Result<Self> {
        let email_struct = Self {
            email: email.to_string(),
        };

        match email_struct.validate() {
            Ok(_) => Ok(email_struct),
            Err(errors) => Err(eyre!("Invalid email format: {errors:?}")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_email() {
        let valid_emails = vec![
            "test@example.com",
            "user.name@domain.co.uk",
            "user+tag@example.org",
            "123@example.com",
            "a@b.co",
        ];

        for email in valid_emails {
            let result = Email::parse(email);
            assert!(
                result.is_ok(),
                "Expected {} to be valid, but got error: {:?}",
                email,
                result
            );

            if let Ok(parsed_email) = result {
                assert_eq!(parsed_email.as_ref(), email);
            }
        }
    }

    #[test]
    fn test_invalid_email() {
        let invalid_emails = vec![
            "invalid-email",
            "@example.com",
            "user@",
            "user@@example.com",
            "",
            "user@.com",
            "user@example.",
            "user name@example.com", // space in local part
        ];

        for email in invalid_emails {
            let result = Email::parse(email);
            assert!(
                result.is_err(),
                "Expected {} to be invalid, but it was accepted",
                email
            );
        }
    }

    #[test]
    fn test_as_ref_implementation() {
        let email_str = "test@example.com";
        let email = Email::parse(email_str).unwrap();
        assert_eq!(email.as_ref(), email_str);
    }
}
